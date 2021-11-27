use crate::{
    errors::Span,
    lexer::{phase0, Lexer, Token, TokenKind},
    preprocessor::{
        evaluator::{EvalError, ExpressionEvaluator, ToBool},
        expressions::{ExpressionParser, Value},
    },
    session::Session,
};

use super::{
    maps::{MLMacroMap, SLMacroMap},
    parser::Parser,
    past::{
        IfClause, IfCondition, IfStatement, Include, MLMacroDef, MLMacroUndef, PASTNode, Repeat,
        SLMacroDef, SLMacroUndef,
    },
};

pub type EResult<T> = Result<T, ()>;
pub type EMaybe = Result<Option<Vec<Token>>, ()>;

pub struct Executor<'a> {
    session: &'a mut Session,
    sl_macros: SLMacroMap,
    ml_macros: MLMacroMap,
}

impl<'a> Executor<'a> {
    pub fn new(session: &'a mut Session) -> Self {
        Self {
            session,
            sl_macros: SLMacroMap::new(),
            ml_macros: MLMacroMap::new(),
        }
    }

    /// Run the executor
    pub fn execute(mut self, nodes: Vec<PASTNode>) -> EResult<Vec<Token>> {
        let new_tokens = self.execute_nodes(nodes)?;

        Ok(new_tokens)
    }

    fn execute_nodes(&mut self, nodes: Vec<PASTNode>) -> EResult<Vec<Token>> {
        let mut new_tokens = Vec::new();

        // println!("{:#?}", nodes);

        for node in nodes {
            if let Some(mut tokens) = match node {
                PASTNode::IfStatement(statement) => self.execute_if_statement(statement)?,
                PASTNode::SLMacroDef(sl_macro) => self.execute_sl_macro_def(sl_macro)?,
                PASTNode::MLMacroDef(ml_macro) => self.execute_ml_macro_def(ml_macro)?,
                PASTNode::BenignTokens(tokens) => Some(tokens.tokens),
                PASTNode::Repeat(repeat) => self.execute_rep(repeat)?,
                PASTNode::Include(include) => self.execute_include(include)?,
                PASTNode::SLMacroUndef(sl_macro_undef) => {
                    self.execute_sl_macro_undef(sl_macro_undef)?
                }
                PASTNode::MLMacroUndef(ml_macro_undef) => {
                    self.execute_ml_macro_undef(ml_macro_undef)?
                }
                _ => unimplemented!(),
            } {
                new_tokens.append(&mut tokens);
            }
        }

        Ok(new_tokens)
    }

    fn execute_ml_macro_undef(&mut self, ml_macro_undef: MLMacroUndef) -> EMaybe {
        self.ml_macros.undefine(ml_macro_undef);

        Ok(None)
    }

    fn execute_sl_macro_undef(&mut self, sl_macro_undef: SLMacroUndef) -> EMaybe {
        self.sl_macros.undefine(sl_macro_undef);

        Ok(None)
    }

    fn include_path(&mut self, span: &Span, path: &str) -> EResult<Vec<Token>> {
        // Check if we have been given a valid file
        if !self.session.is_file(&path) {
            self.session
                .struct_span_error(*span, format!("path provided `{}` is not a file", path))
                .emit();

            return Err(());
        }

        // Read it
        let file_id = match self.session.read_file(&path) {
            Ok(file_id) => file_id,
            Err(e) => {
                self.session
                    .struct_bug(format!("unable to read file `{}`: {}", &path, e))
                    .emit();

                return Err(());
            }
        };

        let file = self.session.get_file(file_id as usize).unwrap();

        // Create the lexer
        let lexer = Lexer::new(&file.source, file_id, &self.session);

        // Lex the tokens, if they are all valid
        let mut tokens = lexer.lex()?;

        // Replace comments and line continuations
        phase0(&mut tokens, &self.session)?;

        let preprocessor_parser = Parser::new(tokens, &self.session);

        let nodes = preprocessor_parser.parse()?;

        let tokens = self.execute_nodes(nodes)?;

        Ok(tokens)
    }

    fn execute_include(&mut self, include: Include) -> EMaybe {
        let path = self.execute_nodes(include.path.expression)?;

        if let Some(path_token) = path
            .iter()
            .find(|token| token.kind != TokenKind::Whitespace)
        {
            if path_token.kind == TokenKind::LiteralString {
                let path_span = path_token.as_span();
                let path_snippet = self.session.span_to_snippet(&path_span);

                let path_str = path_snippet.as_slice().trim_matches('\"');

                let included_tokens = self.include_path(&include.path.span, path_str)?;

                Ok(Some(included_tokens))
            } else {
                self.session
                    .struct_span_error(include.path.span, "expected path".to_string())
                    .emit();

                Err(())
            }
        } else {
            self.session
                .struct_span_error(include.path.span, ".include requires path".to_string())
                .help("macros may have expanded to nothing".to_string())
                .emit();

            Err(())
        }
    }

    fn execute_rep(&mut self, repeat: Repeat) -> EMaybe {
        let evaluation = self.evaluate_expression(&repeat.number.span, repeat.number.expression)?;

        let num = match evaluation {
            Value::Int(i) => i,
            Value::Bool(_) => {
                self.session
                    .struct_span_error(
                        repeat.number.span,
                        "expression resulted in boolean value".to_string(),
                    )
                    .help(".rep requires an integer value".to_string())
                    .emit();

                return Err(());
            }
            Value::Double(d) => d as i32,
        };

        if num < 0 {
            self.session
                .struct_span_error(
                    repeat.number.span,
                    "expression resulted in negative number".to_string(),
                )
                .help(".rep number must be positive".to_string())
                .emit();

            return Err(());
        }

        let mut repeat_tokens = self.execute_nodes(repeat.contents)?;

        repeat_tokens = repeat_tokens.repeat(num as usize);

        Ok(Some(repeat_tokens))
    }

    fn execute_sl_macro_def(&mut self, sl_macro: SLMacroDef) -> EMaybe {
        if let Some(ml_macro) = self.ml_macros.find_by_hash(sl_macro.identifier.hash) {
            self.session
                .struct_span_error(
                    sl_macro.identifier.span,
                    "Macro defined with same name".to_string(),
                )
                .span_label(
                    ml_macro.identifier.span,
                    "Previously defined here".to_string(),
                )
                .emit();

            return Err(());
        }

        self.sl_macros.define(sl_macro);

        Ok(None)
    }

    fn execute_ml_macro_def(&mut self, ml_macro: MLMacroDef) -> EMaybe {
        if let Some(sl_macro) = self.sl_macros.find_by_hash(ml_macro.identifier.hash) {
            self.session
                .struct_span_error(
                    ml_macro.identifier.span,
                    "Macro defined with same name".to_string(),
                )
                .span_label(
                    sl_macro.identifier.span,
                    "Previously defined here".to_string(),
                )
                .emit();

            return Err(());
        }

        self.ml_macros.define(ml_macro);

        Ok(None)
    }

    // Executes an if statement
    fn execute_if_statement(&mut self, statement: IfStatement) -> EMaybe {
        for clause in statement.clauses {
            if let Some(tokens) = self.execute_if_clause(clause)? {
                return Ok(Some(tokens));
            }
        }

        Ok(None)
    }

    fn execute_if_clause(&mut self, clause: IfClause) -> EMaybe {
        let inverse = clause.begin.inverse;

        let condition = self.evaluate_if_condition(clause.condition)? ^ inverse;

        println!("condition: {}", condition);

        Ok(if condition {
            let nodes = clause.contents;

            let tokens = self.execute_nodes(nodes)?;

            Some(tokens)
        } else {
            None
        })
    }

    fn evaluate_expression(&mut self, span: &Span, expression: Vec<PASTNode>) -> EResult<Value> {
        let expanded_tokens = self.execute_nodes(expression)?;
        let mut token_iter = expanded_tokens.iter().peekable();

        let root_node = match ExpressionParser::parse_expression(&mut token_iter, &self.session) {
            Ok(maybe_node) => {
                if let Some(root_node) = maybe_node {
                    root_node
                } else {
                    self.session
                        .struct_span_error(*span, "expected expression".to_string())
                        .emit();

                    return Err(());
                }
            }
            Err(mut db) => {
                db.emit();
                todo!()
            }
        };

        let evaluation = match ExpressionEvaluator::evaluate(&root_node) {
            Ok(evaluation) => evaluation,
            Err(e) => {
                let error_message = match e {
                    EvalError::NegateBool => "`-` operator invalid for booleans",
                    EvalError::FlipDouble => "`~` operator invalid for doubles",
                    EvalError::ZeroDivide => "expression tried to divide by 0",
                }
                .to_string();

                self.session.struct_span_error(*span, error_message).emit();

                return Err(());
            }
        };

        Ok(evaluation)
    }

    fn evaluate_if_condition(&mut self, condition: IfCondition) -> EResult<bool> {
        match condition {
            IfCondition::Exp(expression) => {
                let evaluation =
                    self.evaluate_expression(&expression.span, expression.expression)?;

                Ok(evaluation.to_bool())
            }
            IfCondition::Def(definition) => {
                let hash = definition.identifier.hash;

                let args = match &definition.args {
                    Some(args) => (args.required, args.maximum),
                    None => (0, None),
                };

                match args {
                    (_, Some(_)) => Ok(self.ml_macros.contains(hash, &definition.args)),
                    (num_args, None) => Ok({
                        self.sl_macros.contains(hash, num_args)
                            || self.ml_macros.contains(hash, &definition.args)
                    }),
                }
            }
            IfCondition::Else => Ok(true),
        }
    }
}
