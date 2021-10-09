use crate::{lexer::Token, session::Session};

use super::past::{PASTNode, PAST};

pub struct Parser {
    tokens: Vec<Token>,
    token_cursor: usize,
    session: Session,
    past: PAST,
}

impl Parser {
    pub fn new(tokens: Vec<Token>, session: Session) -> Self {
        Self {
            tokens,
            token_cursor: 0,
            session,
            past: PAST::new(),
        }
    }

    pub fn parse(&mut self) -> Result<(), ()> {
        todo!();

        Ok(())
    }

    fn parse_bit(&mut self) -> Result<PASTNode, ()> {
        todo!();
    }
}
