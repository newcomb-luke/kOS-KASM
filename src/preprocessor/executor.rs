use crate::{lexer::Token, session::Session};

use super::past::{BenignTokens, IfClause, IfCondition, IfStatement, PASTNode};

pub type EResult<T> = Result<T, ()>;

pub struct Executor {
    nodes: Vec<PASTNode>,
    session: Session,
}

impl Executor {
    pub fn new(nodes: Vec<PASTNode>, session: Session) -> Self {
        Self { nodes, session }
    }

    /// Run the executor
    pub fn execute(self) -> EResult<(Vec<Token>, Session)> {
        for node in &self.nodes {
            println!("{:#?}", node);
            let new_tokens = match node {
                PASTNode::IfStatement(statement) => self.execute_if_statement(&statement)?,
                _ => unimplemented!(),
            };
        }

        unimplemented!()
    }

    // Executes an if statement
    fn execute_if_statement(&self, statement: &IfStatement) -> EResult<Option<Vec<Token>>> {
        for clause in &statement.clauses {
            if let Some(tokens) = self.execute_if_clause(clause)? {
                return Ok(Some(tokens));
            }
        }

        Ok(None)
    }

    fn execute_if_clause(&self, clause: &IfClause) -> EResult<Option<Vec<Token>>> {
        let inverse = clause.begin.inverse;

        println!("Inverse: {}", inverse);
        todo!();
    }

    fn evaluate_if_condition(&self, condition: IfCondition) -> EResult<bool> {
        todo!();
    }

    fn expand_macros(&self, nodes: &Vec<PASTNode>) -> EResult<BenignTokens> {
        todo!();
    }
}
