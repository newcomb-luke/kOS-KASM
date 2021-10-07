use crate::{lexer::Token, session::Session};

pub struct Parser {
    tokens: Vec<Token>,
    token_cursor: usize,
    session: Session,
}

impl Parser {
    pub fn parse(&mut self, session: Session) -> () {}
}
