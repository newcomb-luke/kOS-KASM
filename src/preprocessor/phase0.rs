use crate::{
    errors::Error,
    lexer::token::{Token, TokenKind},
};

/// Phase 0 replaces comments and line continues with whitespace
pub fn phase0(tokens: &mut Vec<Token>) -> Result<(), Error> {
    let mut last_was_backslash = false;

    // Loop through all of the tokens
    for (token_index, token) in tokens.iter_mut().enumerate() {
        // If the last token was a backslash (line continue)
        if last_was_backslash {
            // If it was a newline as expected, then replace it with whitespace and reset
            if token.kind == TokenKind::Newline {
                token.kind = TokenKind::Whitespace;
                last_was_backslash = false;
            }
            // If it was whitespace that is fine
            else if token.kind != TokenKind::Whitespace {
                // If it wasn't though, that is an error
                return Err(Error::new(
                    crate::errors::ErrorKind::JunkAfterBackslash,
                    token_index as u32,
                ));
            }
        } else {
            match token.kind {
                // If it is a comment, replace it with whitespafce
                TokenKind::Comment => {
                    token.kind = TokenKind::Whitespace;
                }
                // If it is a backslash, replace it and prepare next iteration
                TokenKind::Backslash => {
                    token.kind = TokenKind::Whitespace;
                    last_was_backslash = true;
                }
                _ => {}
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        lexer::{
            token::{Token, TokenKind},
            tokenize,
        },
        preprocessor::phase0::phase0,
    };

    #[test]
    fn phase0_pass_test() {
        let source = "push \\\n\t2";

        let mut tokens: Vec<Token> = tokenize(source).collect();

        let phase0_result = phase0(&mut tokens);

        assert!(phase0_result.is_ok());

        for token in tokens {
            assert_ne!(token.kind, TokenKind::Backslash);
        }
    }

    #[test]
    fn phase0_junk_test() {
        let source = "push \\ unexpected \n\t2";

        let mut tokens: Vec<Token> = tokenize(source).collect();

        let phase0_result = phase0(&mut tokens);

        assert!(phase0_result.is_err());
    }
}
