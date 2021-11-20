use crate::{
    errors::{ErrorKind, ErrorManager, SourceFile},
    lexer::{
        parse_token, test_next_is,
        token::{Token, TokenKind},
        TokenIter,
    },
};

#[derive(Debug)]
pub struct Repeat {
    times: usize,
    contents: Vec<Token>,
}

impl Repeat {
    /// Parses a .rep directive from a provided TokenIter
    pub fn parse(
        token_iter: &mut TokenIter,
        source_files: &Vec<SourceFile>,
        errors: &mut ErrorManager,
    ) -> Option<Self> {
        // .rep will always be there
        token_iter.next();

        // The number of times to repeat
        // TODO: Binary and hex literal support
        let times_token = parse_token(
            token_iter,
            errors,
            TokenKind::LiteralInteger,
            ErrorKind::ExpectedRepeatNumber,
            ErrorKind::MissingRepeatNumber,
        )?;

        // Parse it as an integer
        let times = times_token
            .slice(source_files)
            .unwrap()
            .parse::<usize>()
            .unwrap();

        // Assert and consume newline
        parse_token(
            token_iter,
            errors,
            TokenKind::Newline,
            ErrorKind::ExpectedRepeatNewline,
            ErrorKind::MissingRepeatNewline,
        )?;

        // Now begins the contents, so continue collecting the contents until a .endrep is
        // encountered

        let mut contents = Vec::new();

        // While we haven't reached an .endrep
        while !test_next_is(
            token_iter,
            errors,
            TokenKind::DirectiveEndRepeat,
            ErrorKind::ExpectedEndRepeat,
        )? {
            // Add it to the contents
            contents.push(*token_iter.next().unwrap());
        }

        // Now consume the .endrep
        token_iter.next();

        Some(Self { times, contents })
    }

    pub fn invoke(&self, preprocessed_tokens: &mut Vec<Token>) {
        for _ in 0..self.times {
            for token in self.contents.iter() {
                preprocessed_tokens.push(*token);
            }
        }
    }
}
