use crate::{LabelManager, Token};

use super::{functions::Function, ParseResult};
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

        println!("Tokens being passed to the parser:");

        for token in tokens {
            println!("{}", token.as_str());
        }

        unreachable!();
    }
}
