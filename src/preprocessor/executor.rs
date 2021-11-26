use std::cell::RefCell;

use crate::{lexer::Token, session::Session};

use super::{
    maps::SLMacroMap,
    past::{BenignTokens, IfClause, IfCondition, IfStatement, PASTNode, SLMacroDef},
};

pub type EResult<T> = Result<T, ()>;

pub struct Executor {
    session: Session,
    sl_macros: SLMacroMap,
}

impl Executor {
    pub fn new(session: Session) -> Self {
        Self {
            session,
            sl_macros: SLMacroMap::new(),
        }
    }

    /// Run the executor
    pub fn execute(mut self, nodes: Vec<PASTNode>) -> EResult<(Vec<Token>, Session)> {
        let new_tokens = self.execute_nodes(nodes)?;

        Ok((new_tokens, self.session))
    }

    fn execute_nodes(&mut self, nodes: Vec<PASTNode>) -> EResult<Vec<Token>> {
        let mut new_tokens = Vec::new();

        for node in nodes {
            if let Some(mut tokens) = match node {
                PASTNode::IfStatement(statement) => self.execute_if_statement(statement)?,
                PASTNode::SLMacroDef(sl_macro) => self.execute_sl_macro_def(sl_macro)?,
                PASTNode::BenignTokens(tokens) => Some(tokens.tokens),
                _ => unimplemented!(),
            } {
                new_tokens.append(&mut tokens);
            }
        }

        Ok(new_tokens)
    }

    fn execute_sl_macro_def(&mut self, sl_macro: SLMacroDef) -> EResult<Option<Vec<Token>>> {
        self.sl_macros.define(sl_macro);

        Ok(None)
    }

    // Executes an if statement
    fn execute_if_statement(&mut self, statement: IfStatement) -> EResult<Option<Vec<Token>>> {
        for clause in statement.clauses {
            if let Some(tokens) = self.execute_if_clause(clause)? {
                return Ok(Some(tokens));
            }
        }

        a

        Ok(None)
    }

    fn execute_if_clause(&mut self, clause: IfClause) -> EResult<Option<Vec<Token>>> {
        let inverse = clause.begin.inverse;

        let condition = self.evaluate_if_condition(&clause.condition)? ^ inverse;

        println!("Condition: {}", condition);

        Ok(if condition {
            let nodes = clause.contents;

            let tokens = self.execute_nodes(nodes)?;

            Some(tokens)
        } else {
            None
        })
    }

    fn evaluate_if_condition(&self, condition: &IfCondition) -> EResult<bool> {
        match condition {
            IfCondition::Exp(expression) => {
                todo!()
            }
            IfCondition::Def(definition) => {
                let hash = definition.identifier.hash;

                let args = match &definition.args {
                    Some(args) => (args.required, args.maximum),
                    None => (0, None),
                };

                match args {
                    (required, Some(maximum)) => {
                        todo!()
                    }
                    (num_args, None) => Ok(self.sl_macros.contains(hash, num_args)),
                }
            }
        }
    }

    fn expand_macros(&self, nodes: &Vec<PASTNode>) -> EResult<BenignTokens> {
        todo!();
    }
}
