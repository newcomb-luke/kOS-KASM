use crate::{
    lexer::Token,
    preprocessor::{
        evaluator::{EvalError, ExpressionEvaluator, ToBool},
        expressions::ExpressionParser,
    },
    session::Session,
};

use super::{
    maps::{MLMacroMap, SLMacroMap},
    past::{IfClause, IfCondition, IfStatement, MLMacroDef, PASTNode, SLMacroDef},
};

pub type EResult<T> = Result<T, ()>;
pub type EMaybe = Result<Option<Vec<Token>>, ()>;

pub struct Executor {
    session: Session,
    sl_macros: SLMacroMap,
    ml_macros: MLMacroMap,
}

impl Executor {
    pub fn new(session: Session) -> Self {
        Self {
            session,
            sl_macros: SLMacroMap::new(),
            ml_macros: MLMacroMap::new(),
        }
    }

    /// Run the executor
    pub fn execute(mut self, nodes: Vec<PASTNode>) -> EResult<(Vec<Token>, Session)> {
        let new_tokens = self.execute_nodes(nodes)?;

        Ok((new_tokens, self.session))
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
                _ => unimplemented!(),
            } {
                new_tokens.append(&mut tokens);
            }
        }

        Ok(new_tokens)
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

    fn evaluate_if_condition(&mut self, condition: IfCondition) -> EResult<bool> {
        match condition {
            IfCondition::Exp(expression) => {
                let expanded_tokens = self.execute_nodes(expression.expression)?;
                let mut token_iter = expanded_tokens.iter().peekable();

                let root_node =
                    match ExpressionParser::parse_expression(&mut token_iter, &self.session) {
                        Ok(maybe_node) => {
                            if let Some(root_node) = maybe_node {
                                root_node
                            } else {
                                self.session
                                    .struct_span_error(
                                        expression.span,
                                        "expected expression".to_string(),
                                    )
                                    .emit();

                                return Err(());
                            }
                        }
                        Err(mut db) => {
                            db.emit();
                            todo!()
                        }
                    };

                println!("Parsed expression: {:?}", root_node);

                let evaluation = match ExpressionEvaluator::evaluate(&root_node) {
                    Ok(evaluation) => evaluation,
                    Err(e) => {
                        let error_message = match e {
                            EvalError::NegateBool => "`-` operator invalid for booleans",
                            EvalError::FlipDouble => "`~` operator invalid for doubles",
                            EvalError::ZeroDivide => "expression tried to divide by 0",
                        }
                        .to_string();

                        self.session
                            .struct_span_error(expression.span, error_message)
                            .emit();

                        return Err(());
                    }
                };

                println!("Result: {:?}", evaluation);

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
