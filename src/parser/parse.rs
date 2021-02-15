use crate::{LabelManager, ParseError, Token, TokenType};

use super::{functions::Function, instructions::Operand, ParseResult};
pub struct Parser {}

impl Parser {
    /// This function parses the token vector with the help of the label manager
    /// This performs the first pass of a two-pass assembler:
    /// Collecting the functions into separate lists of instructions, while gathering labels
    /// This also then replaces those labels, and outputs functions in a form suitable for creating an object file from
    pub fn parse(
        tokens: Vec<Token>,
        label_manager: &mut LabelManager,
    ) -> ParseResult<Vec<Function>> {
        let mut functions = Vec::new();
        let mut location_counter = 0;

        let mut token_iter = tokens.iter().peekable();

        // Go until we find the first token that is a label
        while token_iter.peek().is_some() {
            // Skip all of the newlines
            while token_iter.peek().is_some()
                && token_iter.peek().unwrap().tt() == TokenType::NEWLINE
            {
                // Consume it
                token_iter.next();
            }

            // Now in theory we should have a function marker. If not, that is an issue.
            // Check if we are not at EOF
            if token_iter.peek().is_some() {
                // Now we need to test if this token is indeed a function
                if token_iter.peek().unwrap().tt() == TokenType::FUNCTION {
                    token_iter.next();
                    // Parse the function
                    let func =
                        Function::parse(&mut token_iter, &mut location_counter, label_manager)?;

                    // Add the function to the list
                    functions.push(func);
                } else {
                    return Err(ParseError::TokenOutsideFunctionError(
                        token_iter.peek().unwrap().line(),
                    ));
                }
            }
        }

        // Check for undefined labels
        Parser::check_labels(&functions, label_manager)?;

        Ok(functions)
    }

    /// This function checks if all LABELREFs in the instructions actually exist
    fn check_labels(functions: &Vec<Function>, label_manager: &LabelManager) -> ParseResult<()> {
        for function in functions {
            for instr in function.instructions() {
                for op in instr.operands() {
                    match op {
                        Operand::LABELREF(r) => {
                            if !label_manager.ifdef(r) {
                                return Err(ParseError::UndefinedLabelError(r.to_owned()));
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(())
    }
}
