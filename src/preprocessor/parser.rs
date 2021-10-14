use std::{collections::hash_map::DefaultHasher, hash::Hasher};

type PResult<T> = Result<T, ()>;

use crate::{
    errors::{DiagnosticBuilder, Span},
    lexer::{Token, TokenKind},
    preprocessor::past::SLMacroDef,
    session::Session,
};

use super::past::{Ident, PASTNode, SLMacroDefArgs, SLMacroDefContents, PAST};

pub struct Parser {
    tokens: Vec<Token>,
    token_cursor: usize,
    session: Session,
    past: PAST,
    last_token: Option<Token>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>, session: Session) -> Self {
        let first_token = tokens.get(0).copied();

        Self {
            tokens,
            token_cursor: 0,
            session,
            past: PAST::new(),
            last_token: first_token,
        }
    }

    pub fn parse(&mut self) -> Result<(), ()> {
        todo!();

        Ok(())
    }

    fn parse_bit(&mut self) -> PResult<PASTNode> {
        todo!();
    }

    // Parse a single line macro definition
    //
    // See the SLMacroDef grammar
    //
    fn parse_sl_macro_def(&mut self) -> PResult<PASTNode> {
        let mut span = Span::new(0, 0, 0);
        // Consume the .define
        let define_span = self.assert_next(TokenKind::DirectiveDefine)?;

        // Copy the span values
        span.start = define_span.start;
        span.file = define_span.file;

        // As per the grammar, the next token MUST be an identifier
        let identifier = self.parse_ident()?;

        // Now we parse the optional arguments
        let args = self.parse_sl_macro_def_args()?;

        // Then the optional contents
        let contents = self.parse_sl_macro_def_contents()?;

        // Adjust this SLMacroDef's span
        if let Some(contents) = &contents {
            span.end = contents.span.end;
        } else if let Some(args) = &args {
            // This means we have arguments, but no tokens to expand
            // This is valid, but we should emit a warning

            self.session
                .struct_span_warn(args.span, "macro arguments but no expansion".to_string())
                .emit();

            span.end = args.span.end;
        } else {
            span.end = identifier.span.end;
        }

        Ok(PASTNode::SLMacroDef(SLMacroDef::new(
            span, identifier, args, contents,
        )))
    }

    // Parse a single line macro definition arguments
    fn parse_sl_macro_def_args(&mut self) -> PResult<Option<SLMacroDefArgs>> {
        if let Some(&token) = self.peek_next() {
            // ( is required
            if token.kind != TokenKind::SymbolLeftParen {
                // No (, no args
                Ok(None)
            } else {
                let mut arguments = Vec::new();
                let mut span = Span::new(0, 0, 0);

                // Consume the (
                let paren_span = self.assert_next(TokenKind::SymbolLeftParen)?;

                span.start = paren_span.start;
                span.file = paren_span.file;

                // We have (
                // Now we parse until we reach a )
                while let Some(&token) = self.peek_next() {
                    // Check if it is a )
                    if token.kind == TokenKind::SymbolRightParen {
                        break;
                    }

                    // It should now be an identifier
                    let ident = self.parse_ident()?;
                    arguments.push(ident);

                    // Now we should check if it is a comma, or a ). Anything else is not allowed
                    if let Some(&next) = self.peek_next() {
                        if next.kind == TokenKind::SymbolComma {
                            self.assert_next(TokenKind::SymbolComma)?;
                        } else if next.kind == TokenKind::SymbolRightParen {
                            continue;
                        } else {
                            // Emit an error, it wasn't either of them
                            self.session
                                .struct_span_error(next.as_span(), "`,` or `)`".to_string())
                                .emit();

                            return Err(());
                        }
                    }
                }

                // We need to check if this ended because we ran out of tokens, which isn't okay
                if self.peek_next().is_none() {
                    // Emit an error
                    self.struct_err_expected_eof(self.last_token.unwrap().as_span(), ")")
                        .emit();

                    Err(())
                } else {
                    // Consume the ) that caused us to stop
                    let right_span = self.assert_next(TokenKind::SymbolRightParen)?;

                    span.end = right_span.end;

                    Ok(Some(SLMacroDefArgs::new(span, arguments)))
                }
            }
        } else {
            Ok(None)
        }
    }

    // Parse a single line macro definition contents
    fn parse_sl_macro_def_contents(&mut self) -> PResult<Option<SLMacroDefContents>> {
        todo!();
    }

    // Peeks the next token from the Parser's tokens
    fn peek_next(&self) -> Option<&Token> {
        self.tokens.get(self.token_cursor)
    }

    // Consumes the next token from the Parser's tokens
    fn consume_next(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.token_cursor)?;

        self.token_cursor += 1;
        self.last_token = Some(*token);

        Some(token)
    }

    // Emits a bug diagnostic and Err(()) if the token was not what we assumed it to be
    fn assert_next(&mut self, token_kind: TokenKind) -> PResult<Span> {
        if let Some(&token) = self.consume_next() {
            if token.kind == token_kind {
                Ok(token.as_span())
            } else {
                self.session
                    .struct_bug(format!(
                        "token assertion failed: {:?}, found {:?}",
                        token_kind, token.kind
                    ))
                    .emit();
                Err(())
            }
        } else {
            // If it doesn't exist, that is also a problem
            self.session
                .struct_bug(format!(
                    "{:?} token assumed to exist, found None",
                    token_kind
                ))
                .emit();
            Err(())
        }
    }

    // Skips whitespace if there is any, not including newlines
    //
    // Returns true if there was any, false if not
    fn skip_whitespace(&mut self) -> bool {
        let mut was_whitespace = false;

        while self.peek_next().is_some() && self.peek_next().unwrap().kind == TokenKind::Whitespace
        {
            was_whitespace = true;

            // Increment the token cursor
            self.token_cursor += 1;
        }

        was_whitespace
    }

    // Parses an identifier, or returns an Err(())
    //
    // This will emit a diagnostic if the next token is not an identifier
    //
    fn parse_ident(&mut self) -> PResult<Ident> {
        if let Some(&token) = self.consume_next() {
            if token.kind == TokenKind::Identifier {
                let span = token.as_span();
                let snippet = self.session.span_to_snippet(&span);

                let mut hasher = DefaultHasher::new();
                hasher.write(snippet.as_slice().as_bytes());
                let hash = hasher.finish();

                Ok(Ident { span, hash })
            } else {
                self.struct_err_expected_found(token.as_span(), "identifier")
                    .emit();

                Err(())
            }
        } else {
            self.struct_err_expected_eof(self.last_token.unwrap().as_span(), "identifier")
                .emit();

            Err(())
        }
    }

    fn struct_err_expected_eof(&self, last: Span, expected: &str) -> DiagnosticBuilder<'_> {
        let message = format!("expected {}", expected);
        let mut db = self.session.struct_error(message);

        db.span_label(last, format!("found end of file"));

        db
    }

    fn struct_err_expected_found(&self, found: Span, expected: &str) -> DiagnosticBuilder<'_> {
        let message = format!("expected {}", expected);
        let mut db = self.session.struct_error(message);

        db.span_label(
            found,
            format!("found `{}`", self.session.span_to_snippet(&found)),
        );

        db
    }
}
