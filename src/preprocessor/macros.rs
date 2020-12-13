use std::{error::Error, slice::Iter, iter::Peekable};

use crate::{Token, TokenType, TokenData, Value};

#[derive(Debug, Clone)]
pub struct MacroArg {
    required: bool,
    default: Vec<Token>
}

impl MacroArg {
    /// Create a new macro argument.
    pub fn new(required: bool, default: Vec<Token>) -> MacroArg {
        MacroArg { required, default }
    }

    /// Returns true of this argument is required.
    pub fn required(&self) -> bool {
        self.required
    }

    /// Returns the default value of this argument.
    pub fn default_owned(&self) -> Vec<Token> {
        self.default.to_owned()
    }
}

pub struct Macro {
    id: String,
    contents: Vec<Token>,
    args: Vec<MacroArg>,
    num_required_args: usize
}

impl Macro {

    /// Creates a new macro
    pub fn new(id: &str, contents: Vec<Token>, args: Vec<MacroArg>, num_required_args: usize) -> Macro {
        Macro {
            id: id.to_owned(),
            contents,
            args,
            num_required_args
        }
    }

    /// Parses a macro from the provided token iterator
    pub fn parse_macro(start_line: usize, token_iter: &mut Peekable<Iter<Token>>) -> Result<Macro, Box<dyn Error>> {
        // Check to see if we have a token, and it is an identifier
        if token_iter.peek().is_some() && token_iter.peek().unwrap().tt() == TokenType::IDENTIFIER {
            let id = match token_iter.next().unwrap().data() { TokenData::STRING(s) => s, _ => unreachable!() };
            let mut contents = Vec::new();
            let mut args = Vec::new();
            let mut clean_exit = false;
            let mut min_args= 0;
            let mut max_args = 0;

            // There needs to be an .endmacro directive, so no more tokens is an error
            if token_iter.peek().is_none() {
                return Err(format!("Incomplete macro definition {}, expected macro body. Line {}", id, start_line).into())
            }
            // If there are arguments to this macro
            else if token_iter.peek().unwrap().tt() != TokenType::NEWLINE {
                
                // Now we either have a number of arguments, or a range.

                // It has to be an int
                if token_iter.peek().unwrap().tt() != TokenType::INT {
                    return Err(format!("Expected number of macro arguments, found: {}. Line: {}", token_iter.peek().unwrap().as_str(), start_line).into());
                }
                min_args = match token_iter.next().unwrap().data() { TokenData::INT(i) => *i, _ => unreachable!()};
                max_args = min_args;

                // The next token can either be a newline or a -
                if token_iter.peek().unwrap().tt() == TokenType::NEWLINE {
                    // If it is a newline that means that all arguments are required.
                    for _ in 0..min_args {
                        args.push(MacroArg::new(true, Vec::new()));
                    }
                }
                // If the next token was not a newline, it must be a minus, followed by a max numberof args
                else if token_iter.peek().unwrap().tt() == TokenType::MINUS {
                    let num_required_default_values;

                    // Consume the comma
                    token_iter.next();

                    // Next has to be an int
                    if token_iter.peek().unwrap().tt() != TokenType::INT {
                        return Err(format!("Expected max number of macro arguments, found: {}. Line: {}", token_iter.peek().unwrap().as_str(), start_line).into());
                    }
                    max_args = match token_iter.next().unwrap().data() { TokenData::INT(i) => *i, _ => unreachable!()};

                    // Test if the range makes sense...
                    if min_args >= max_args {
                        return Err(format!("Invalid macro definition {} argument numbers. Range {}-{} makes no sense. Line {}", id, min_args, max_args, start_line).into());
                    }

                    num_required_default_values = max_args - min_args;

                    // Populate all of the really required arguments
                    for _ in 0..min_args {
                        args.push(MacroArg::new(true, Vec::new()));
                    }

                    // If it is a range, now we have to deal with all of the default values
                    for _ in 0..num_required_default_values {
                        let mut argument_contents = Vec::new();

                        // We need to have all of the macro argument default values, if one is missing, there is a problem.
                        if token_iter.peek().is_none() || token_iter.peek().unwrap().tt() == TokenType::NEWLINE {
                            return Err(format!("Macro {} definition missing required default value. Line {}", id, start_line).into());
                        }

                        // Go until we run out or hit a comma or newline
                        while token_iter.peek().is_some() && token_iter.peek().unwrap().tt() != TokenType::COMMA && token_iter.peek().unwrap().tt() != TokenType::NEWLINE {
                            let token = token_iter.next().unwrap();

                            argument_contents.push(token.clone());
                        }

                        // If it was a comma that stopped us, consume it
                        if token_iter.peek().unwrap().tt() == TokenType::COMMA {
                            token_iter.next();
                        }

                        // Now that we have the argument contents, add it to the list
                        args.push(MacroArg::new(false, argument_contents));

                    }

                    // There MUST be a newline here
                    if token_iter.peek().is_none() || token_iter.peek().unwrap().tt() != TokenType::NEWLINE {
                        return Err(format!("Expected newline after macro arguments in definition {}. Line {}", id, start_line).into());
                    }

                }
                // If it isn't either, that is an error
                else {
                    return Err(format!("Expected newline or argument range. Found {}. Line {}", token_iter.peek().unwrap().as_str(), start_line).into());
                }

            }
            // If there are no arguments, we just move on, but consume the newline
            token_iter.next();

            // Now we can fill in the body of the macro
            while token_iter.peek().is_some() {
                // If the next token on the line is a directive, then test if it is the .endmacro directive
                if token_iter.peek().unwrap().tt() == TokenType::DIRECTIVE {
                    let directive = match token_iter.peek().unwrap().data() { TokenData::STRING(s) => s, _ => unreachable!()};

                    if *directive == "endmacro" {
                        // Consume the directive
                        token_iter.next();
                        // Set clean exit to true
                        clean_exit = true;
                        // Break from this loop
                        break;
                    }
                    // If it isn't the endmacro directive, then just treat it like any other token
                    else {
                        contents.push(token_iter.next().unwrap().clone());
                    }
                }
                // We should also test for argument placeholders
                else if token_iter.peek().unwrap().tt() == TokenType::AMPERSAND {
                    let argument_number;

                    // Consume it
                    token_iter.next();

                    // Now we MUST have an integer follow this
                    if token_iter.peek().is_none() || token_iter.peek().unwrap().tt() != TokenType::INT {
                        return Err(format!("Expected int value after & in macro definition").into());
                    }

                    // Get the number of the argument
                    argument_number = match token_iter.next().unwrap().data() { TokenData::INT(i) => *i, _ => unreachable!() };

                    // Make sure it isn't out of bounds
                    if argument_number > max_args {
                        return Err(format!("Argument number {} is out of bounds for macro definition", argument_number).into());
                    }

                    // Now replace it with a placeholder
                    contents.push(Token::new(TokenType::PLACEHOLDER, TokenData::INT(argument_number)));

                }
                // If it isn't either, just push the token
                else {
                    contents.push(token_iter.next().unwrap().clone());
                }
            }

            // If this loop ended because we ran out of tokens... that is bad
            if !clean_exit {
                Err(format!(".macro {} declaration requires closing .endmacro directive. Reached end of file before .endmacro. Macro line {}", id, start_line).into())
            } else {
                Ok(Macro::new(id, contents, args, min_args as usize))
            }
        }
        // If we don't that is an error because it is required
        else {
            Err(format!("Expected macro identifier. Line {}", start_line).into())
        }
    }

    pub fn id(&self) -> String {
        self.id.to_owned()
    }

    pub fn contents_cloned(&self) -> Vec<Token> {
        self.contents.to_owned()
    }

    pub fn args_cloned(&self) -> Vec<MacroArg> {
        self.args.to_owned()
    }

    pub fn get_contents_iter(&self) -> Peekable<Iter<Token>> {
        self.contents.iter().peekable()
    }

    pub fn args(&self) -> &Vec<MacroArg> {
        &self.args
    }

    pub fn num_required_args(&self) -> usize {
        self.num_required_args
    }

}