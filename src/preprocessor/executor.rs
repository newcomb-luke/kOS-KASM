use crate::{lexer::Token, session::Session};

use super::past::{IfClause, IfCondition, IfStatement, PASTNode};

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
    pub fn execute(&mut self) -> EResult<Vec<Token>> {
        for node in self.nodes.iter() {
            println!("{:#?}", node);
        }

        todo!();
    }

    // Executes an if statement
    fn execute_if_statement(&mut self, statement: IfStatement) -> EResult<Option<Vec<Token>>> {
        todo!();
    }

    fn execute_if_clause(&mut self, clause: IfClause) -> EResult<Option<Vec<Token>>> {
        let inverse = clause.begin.inverse;
        todo!();
    }

    fn execute_if_condition(&mut self, condition: IfCondition) -> EResult<bool> {
        todo!();
    }
}
