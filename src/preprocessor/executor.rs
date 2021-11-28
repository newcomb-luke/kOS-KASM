use std::{collections::hash_map::DefaultHasher, hash::Hasher};

use crate::{
    errors::Span,
    lexer::{phase0, Lexer, Token, TokenKind},
    preprocessor::{
        evaluator::{EvalError, ExpressionEvaluator, ToBool},
        expressions::{ExpressionParser, Value},
        parser::parse_integer_literal,
        past::{BenignTokens, Ident},
    },
    session::Session,
};

use super::{
    maps::{MLMacroMap, SLMacroMap},
    parser::Parser,
    past::{
        IfClause, IfCondition, IfStatement, Include, MLMacroDef, MLMacroUndef, MacroInvok,
        PASTNode, Repeat, SLMacroDef, SLMacroUndef,
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
                PASTNode::MacroInvok(macro_invok) => self.execute_macro_invokation(macro_invok)?,
            } {
                new_tokens.append(&mut tokens);
            }
        }

        Ok(new_tokens)
    }

    fn expand_sl_macro(
        &self,
        sl_macro: &SLMacroDef,
        arg_replacements: Vec<Vec<Token>>,
    ) -> EResult<Option<Vec<PASTNode>>> {
        if let Some(contents) = &sl_macro.contents {
            let new_contents = if let Some(macro_def_args) = &sl_macro.args {
                let arg_idents: &[Ident] = &macro_def_args.args;

                let mut cleaner_contents = Vec::new();

                for node in &contents.contents {
                    if let PASTNode::BenignTokens(benign_tokens) = node {
                        let mut new_benign_tokens = Vec::new();

                        for token in &benign_tokens.tokens {
                            if token.kind == TokenKind::Identifier {
                                let ident_snippet = self.session.span_to_snippet(&token.as_span());
                                let ident_str = ident_snippet.as_slice();

                                let mut hasher = DefaultHasher::new();
                                hasher.write(ident_str.as_bytes());
                                let ident_hash = hasher.finish();

                                if let Some(pos) =
                                    arg_idents.iter().position(|ident| ident.hash == ident_hash)
                                {
                                    let replacement = arg_replacements.get(pos).unwrap();

                                    for replacement_token in replacement {
                                        new_benign_tokens.push(*replacement_token);
                                    }
                                } else {
                                    new_benign_tokens.push(*token);
                                }
                            } else {
                                new_benign_tokens.push(*token);
                            }
                        }

                        cleaner_contents.push(PASTNode::BenignTokens(BenignTokens::from_vec(
                            new_benign_tokens,
                        )));
                    } else {
                        cleaner_contents.push(node.clone());
                    }
                }

                cleaner_contents
            } else {
                contents.contents.clone()
            };

            Ok(Some(new_contents))
        } else {
            Ok(None)
        }
    }

    fn expand_ml_macro(
        &self,
        ml_macro: &MLMacroDef,
        mut arg_replacements: Vec<Vec<Token>>,
        num_args_provided: usize,
    ) -> EResult<Option<Vec<PASTNode>>> {
        if let Some(ml_args) = &ml_macro.args {
            // If there are defaults that we might fill in
            let mut default_replacements = if let Some(arg_defaults) = &ml_macro.defaults {
                let num_needed_defaults =
                    ml_args.maximum.map(|val| val.get() as usize).unwrap_or(0) - num_args_provided;

                let replacement_defaults = arg_defaults
                    .values
                    .iter()
                    .rev()
                    .take(num_needed_defaults)
                    .rev();

                let mut replacement_tokens = Vec::new();

                for replacement_default in replacement_defaults {
                    let mut tokens = Vec::new();

                    for token in &replacement_default.tokens {
                        tokens.push(*token);
                    }

                    replacement_tokens.push(tokens);
                }

                replacement_tokens
            }
            // If there aren't
            else {
                Vec::new()
            };

            // Append the defaults to the replacements
            arg_replacements.append(&mut default_replacements);

            println!("All args: {:#?}", arg_replacements);

            let mut cleaner_contents = Vec::new();

            for node in &ml_macro.contents {
                if let PASTNode::BenignTokens(benign_tokens) = node {
                    let mut new_benign_tokens = Vec::new();
                    let mut was_arg_ref = false;

                    for token in &benign_tokens.tokens {
                        if token.kind == TokenKind::SymbolAnd {
                            println!("yes");
                            was_arg_ref = true;
                        } else if was_arg_ref {
                            was_arg_ref = false;

                            if token.kind != TokenKind::LiteralInteger {
                                println!("token kind: {:?}", token.kind);
                                self.session.struct_bug("didn't properly check for multi-line macro argument references".to_string()).emit();
                                return Err(());
                            }

                            let arg_ref_snippet = self.session.span_to_snippet(&token.as_span());
                            let arg_ref_str = arg_ref_snippet.as_slice();
                            let arg_ref = match parse_integer_literal(arg_ref_str) {
                                Ok(num) => num,
                                Err(_) => {
                                    self.session
                                        .struct_span_error(
                                            token.as_span(),
                                            "integer value out of bounds for signed 32 bit"
                                                .to_string(),
                                        )
                                        .emit();
                                    return Err(());
                                }
                            };

                            if arg_ref == 0 {
                                self.session
                                    .struct_span_error(
                                        token.as_span(),
                                        "macro argument indexes start at 1".to_string(),
                                    )
                                    .emit();
                                return Err(());
                            }

                            // We offset by 1 here, because macro arguments are 1-indexed
                            if let Some(replacement) = arg_replacements.get((arg_ref as usize) - 1)
                            {
                                for token in replacement {
                                    new_benign_tokens.push(*token);
                                }
                            } else {
                                self.session
                                    .struct_span_error(
                                        token.as_span(),
                                        "argument index out of bounds".to_string(),
                                    )
                                    .emit();
                                return Err(());
                            }
                        } else {
                            new_benign_tokens.push(*token);
                        }
                    }

                    cleaner_contents.push(PASTNode::BenignTokens(BenignTokens::from_vec(
                        new_benign_tokens,
                    )));
                } else {
                    cleaner_contents.push(node.clone());
                }
            }

            Ok(Some(cleaner_contents))
        } else {
            Ok(Some(ml_macro.contents.clone()))
        }
    }

    fn execute_macro_invokation(&mut self, macro_invok: MacroInvok) -> EMaybe {
        let invok_args = if let Some(args) = &macro_invok.args {
            args.args.clone()
        } else {
            Vec::new()
        };

        let num_args_provided = invok_args.len();

        // Now we can expand any macros that are in any of the arguments
        let mut arg_replacements = Vec::with_capacity(num_args_provided);

        for node in invok_args {
            let tokens = self.execute_nodes(node.contents)?;

            arg_replacements.push(tokens);
        }

        if let Some(sl_macro) = self.sl_macros.get(&macro_invok) {
            let new_contents = self.expand_sl_macro(sl_macro, arg_replacements)?;

            if let Some(new_contents) = new_contents {
                self.execute_nodes(new_contents).map(Some)
            } else {
                Ok(None)
            }
        } else if let Some(ml_macro) = self.ml_macros.get(&macro_invok) {
            let new_contents =
                self.expand_ml_macro(ml_macro, arg_replacements, num_args_provided)?;

            if let Some(new_contents) = new_contents {
                self.execute_nodes(new_contents).map(Some)
            } else {
                Ok(None)
            }
        } else {
            let macro_name_snippet = self.session.span_to_snippet(&macro_invok.identifier.span);

            let macro_name = macro_name_snippet.as_slice();

            // If there were arguments provided (we know this was an attempt at invoking a
            // macro)
            if num_args_provided != 0 {
                let mut db = self.session.struct_span_error(
                    macro_invok.identifier.span,
                    format!(
                        "use of undeclared macro `{}` with {} argument{}",
                        macro_name,
                        num_args_provided,
                        if num_args_provided == 1 { "" } else { "s" }
                    ),
                );

                // Note for if it exists as a single-line macro
                if let Some(accepted_num_args) = self
                    .sl_macros
                    .get_accepted_num_args(macro_invok.identifier.hash)
                {
                    db.note(format!(
                        "macro `{}` takes {} argument(s)",
                        macro_name, accepted_num_args
                    ));
                }

                db.emit();

                Err(())
            } else {
                // If it exists as a single-line macro
                if let Some(accepted_num_args) = self
                    .sl_macros
                    .get_accepted_num_args(macro_invok.identifier.hash)
                {
                    self.session
                        .struct_span_error(
                            macro_invok.identifier.span,
                            format!(
                                "macro `{}` exists, takes {} argument(s)",
                                macro_name, accepted_num_args
                            ),
                        )
                        .emit();
                } else {
                    // We will give a slightly more vague error message
                    self.session
                        .struct_span_error(
                            macro_invok.identifier.span,
                            "unknown macro or instruction".to_string(),
                        )
                        .emit();
                }

                Err(())
            }
        }
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
        if !self.session.is_file(path) {
            self.session
                .struct_span_error(*span, format!("path provided `{}` is not a file", path))
                .emit();

            return Err(());
        }

        // Read it
        let file_id = match self.session.read_file(path) {
            Ok(file_id) => file_id,
            Err(e) => {
                self.session
                    .struct_bug(format!("unable to read file `{}`: {}", path, e))
                    .emit();

                return Err(());
            }
        };

        let file = self.session.get_file(file_id as usize).unwrap();

        // Create the lexer
        let lexer = Lexer::new(&file.source, file_id, self.session);

        // Lex the tokens, if they are all valid
        let mut tokens = lexer.lex()?;

        // Replace comments and line continuations
        phase0(&mut tokens, self.session)?;

        let preprocessor_parser = Parser::new(tokens, self.session);

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

        let root_node = match ExpressionParser::parse_expression(&mut token_iter, self.session) {
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
