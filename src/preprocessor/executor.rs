use crate::lexer::Token;

use super::past::PASTNode;

pub type EResult<T> = Result<T, ()>;

pub struct Executor {
    nodes: Vec<PASTNode>,
}

impl Executor {
    pub fn new(nodes: Vec<PASTNode>) -> Self {
        Self { nodes }
    }

    /// Run the executor
    pub fn execute(&mut self) -> EResult<Vec<Token>> {
        for node in self.nodes.iter() {
            println!("{:#?}", node);
        }

        todo!();
    }
}
