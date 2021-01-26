use std::{iter::Peekable, slice::Iter};

use crate::{Token, TokenType};
use super::errors::{ParseError, ParseResult};

pub struct Function {
    name: String,
    tokens: Vec<Token>
}

impl Function {

    pub fn new(name: &str, tokens: Vec<Token>) -> Function {
        Function {
            name: name.to_owned(),
            tokens
        }
    }

    pub fn parse(token_iter: &mut Peekable<Iter<Token>>) -> ParseResult<Function> {
        
        // let func_name;

        while token_iter.peek().is_some() {
            // Check if it is a newline, which we ignore
            if token_iter.peek().unwrap().tt() == TokenType::NEWLINE {
                // Just consume it
                token_iter.next();
            }
            
        }

        Err(ParseError::Test)
    }

}