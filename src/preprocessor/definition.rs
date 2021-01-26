use std::{iter::Peekable, slice::Iter};

use crate::{Token, TokenData, TokenType};

use super::errors::*;

pub struct Definition {
    contents: Vec<Token>,
    num_args: usize,
    id: String,
}

impl Definition {
    /// Creates a new Definition
    pub fn new(id: &str, contents: Vec<Token>, num_args: usize) -> Definition {
        Definition {
            contents,
            num_args,
            id: id.to_owned(),
        }
    }

    /// Parses a definition from the provided token interator
    pub fn parse_definition(
        token_iter: &mut Peekable<Iter<Token>>,
    ) -> PreprocessResult<Definition> {
        // Check to see if we have a token, and it is an identifier
        if token_iter.peek().is_some() && token_iter.peek().unwrap().tt() == TokenType::IDENTIFIER {
            let id = match token_iter.next().unwrap().data() {
                TokenData::STRING(s) => s,
                _ => unreachable!(),
            };
            let mut contents = Vec::new();
            let mut num_args = 0;
            let mut arg_ids = Vec::new();

            // Check to see if it is an "empty" definition
            if token_iter.peek().is_some() && token_iter.peek().unwrap().tt() != TokenType::NEWLINE
            {
                // If the definition has tokens

                // Check if it has arguments
                if token_iter.peek().unwrap().tt() == TokenType::OPENPAREN {
                    // Consume the open parenthesis
                    token_iter.next();

                    // If there is a parenthesis, it is required to have at least on argument
                    loop {
                        // Read the argument

                        // If there is no token, throw an error
                        if token_iter.peek().is_none() {
                            return Err(PreprocessError::DefinitionParseError(
                                id.to_owned(),
                                DefinitionError::UnexpectedEOF.into(),
                            ));
                        }
                        // If it is not an identifier throw an error
                        else if token_iter.peek().unwrap().tt() != TokenType::IDENTIFIER {
                            return Err(PreprocessError::DefinitionParseError(
                                id.to_owned(),
                                DefinitionError::ExpectedArgument(
                                    token_iter.peek().unwrap().as_str(),
                                )
                                .into(),
                            ));
                        }
                        // If everything is fine
                        else {
                            // Consume the token, but also add it to the argument identifiers list
                            arg_ids.push(match token_iter.next().unwrap().data() {
                                TokenData::STRING(s) => s,
                                _ => unreachable!(),
                            });
                            // Increment the number of arguments
                            num_args += 1;
                        }

                        // If there are no tokens left
                        if token_iter.peek().is_none() {
                            // That is an error.
                            return Err(PreprocessError::DefinitionParseError(
                                id.to_owned(),
                                DefinitionError::ExpectedClosingParen.into(),
                            ));
                        }
                        // Or if there is a closing parenthesis
                        else if token_iter.peek().unwrap().tt() == TokenType::CLOSEPAREN {
                            // Consume it
                            token_iter.next();
                            // Then break the loop
                            break;
                        }
                        // If there are tokens, and it isn't a closing parenthesis, then it must be a comma
                        else if token_iter.peek().unwrap().tt() == TokenType::COMMA {
                            // Consume it
                            token_iter.next();
                        }
                        // If it isn't a comma, there is a problem
                        else {
                            return Err(PreprocessError::DefinitionParseError(
                                id.to_owned(),
                                Box::new(DefinitionError::ExpectedArgumentsEnd(
                                    token_iter.peek().unwrap().as_str(),
                                )),
                            ));
                        }
                    }
                }

                // Now we read the contents of the definition
                while token_iter.peek().is_some()
                    && token_iter.peek().unwrap().tt() != TokenType::NEWLINE
                {
                    // Get the token
                    let token = token_iter.next().unwrap();

                    // If the token is an identifer
                    if token.tt() == TokenType::IDENTIFIER {
                        let token_id = match token.data() {
                            TokenData::STRING(s) => s,
                            _ => unreachable!(),
                        };
                        let mut argument_index = 0;
                        let mut is_argument = false;
                        // Check to see if it is in the arg ids
                        for (index, id) in arg_ids.iter().enumerate() {
                            // If the id is an argument, say so
                            if *token_id == **id {
                                is_argument = true;
                                argument_index = index;
                            }
                        }

                        if is_argument {
                            // Replace the token with a placeholder token
                            contents.push(Token::new(
                                TokenType::PLACEHOLDER,
                                TokenData::INT(argument_index as i32),
                            ));
                        } else {
                            // Just push the token as it is
                            contents.push(token.clone());
                        }
                    }
                    // If the token is not an identifier, then just append it
                    else {
                        contents.push(token.clone());
                    }
                }

                // If there is another token, then it must be a newline
                if token_iter.peek().is_some() {
                    token_iter.next();
                }
            }
            // If it is empty, do nothing except consume the newline
            else if token_iter.peek().is_some() {
                token_iter.next();
            }

            Ok(Definition::new(id, contents, num_args))
        }
        // If we don't that is an error because it is required
        else {
            Err(PreprocessError::DefinitionParseError(
                String::new(),
                Box::new(DefinitionError::MissingIdentifier),
            ))
        }
    }

    /// Returns the string id of this definition
    pub fn id(&self) -> String {
        self.id.to_owned()
    }

    /// Returns the number of arguments that this definition takes
    pub fn num_args(&self) -> usize {
        self.num_args
    }

    /// Returns false of this definition contains any tokens, otherwise returns true
    pub fn is_empty(&self) -> bool {
        self.contents.len() == 0
    }

    /// Returns an iterator over the tokens contained in this definition
    pub fn get_contents_iter(&self) -> Peekable<Iter<Token>> {
        self.contents.iter().peekable()
    }
}
