use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use crate::{
    errors::{AssemblyError, ErrorKind, ErrorManager, KASMResult, SourceFile},
    lexer::{
        parse_token, test_next_is,
        token::{Token, TokenKind},
        TokenIter,
    },
};

pub struct UnDefinition {}

impl UnDefinition {
    pub fn parse(
        token_iter: &mut TokenIter,
        source_files: &Vec<SourceFile>,
        errors: &mut ErrorManager,
    ) -> KASMResult<String> {
        // .undef will always be there
        token_iter.next();

        // The identifier
        let identifier_token = parse_token(
            token_iter,
            errors,
            TokenKind::Identifier,
            ErrorKind::ExpectedUndefIdentifier,
            ErrorKind::MissingUndefIdentifier,
        )?;

        // Get the actual identifier out of it
        let identifier = identifier_token.slice(source_files).unwrap().to_string();

        Ok(identifier)
    }
}

#[derive(Debug)]
pub struct Definition {
    arguments: Vec<u64>,
    contents: Option<Vec<Token>>,
}

impl Definition {
    /// Parses a single definition from KASM tokens
    ///
    /// There are various types of definitions and therefore a few different types of valid syntax:
    ///
    /// .define FLAG
    ///
    /// .define NUM         25
    ///
    /// .define A(x)        x * 2
    ///
    /// .define B(x, y)     x * y
    ///
    /// push A(x)
    ///
    /// <ident: push> <ident: A> <leftparen> <ident: x> <rightparen>
    ///
    /// call printFunc, #
    ///
    /// <ident: call> <ident: printFunc> <comma> <hash>
    ///
    /// .define printFunc wedseddawdaw
    ///
    /// .func
    /// printFunc:
    ///     store "$x"
    ///     push @
    ///     push "$x"
    ///     call #, "print()"
    ///     ret 0
    ///
    pub fn parse(
        token_iter: &mut TokenIter,
        source_files: &Vec<SourceFile>,
        errors: &mut ErrorManager,
    ) -> KASMResult<(Self, String)> {
        // .define will always be there
        token_iter.next();

        // The identifier
        let identifier_token = parse_token(
            token_iter,
            errors,
            TokenKind::Identifier,
            ErrorKind::ExpectedDirectiveIdentifier,
            ErrorKind::MissingDirectiveIdentifier,
        )?;

        // Get the actual identifier out of it
        let identifier = identifier_token.slice(source_files).unwrap().to_string();

        let mut arguments = Vec::new();

        let mut contents = Vec::new();

        // After this, come the optional arguments
        if let Some(token) = token_iter.peek() {
            // If it is a (
            if token.kind == TokenKind::SymbolLeftParen {
                // Consume it
                token_iter.next();

                // Check if the next token is already a )
                // This generates a warning, but isn't technically an error
                if test_next_is(
                    token_iter,
                    errors,
                    TokenKind::SymbolRightParen,
                    ErrorKind::UnexpectedEndOfDirectiveArguments,
                )? {
                    // Add the warning
                    // This also consumes the )
                    errors.add_assembly(AssemblyError::new(
                        ErrorKind::WarnEmptyDirectiveArguments,
                        *token_iter.next().unwrap(),
                    ))
                } else {
                    // While we have arguments
                    while !test_next_is(
                        token_iter,
                        errors,
                        TokenKind::SymbolRightParen,
                        ErrorKind::UnexpectedEndOfDirectiveArguments,
                    )? {
                        // Parse some arguments
                        let arg_token = parse_token(
                            token_iter,
                            errors,
                            TokenKind::Identifier,
                            ErrorKind::UnexpectedEndOfDirectiveArguments,
                            ErrorKind::InvalidTokenDirectiveArguments,
                        )?;

                        // Get the actual identifier out of it
                        let arg_str = arg_token.slice(source_files).unwrap();

                        // Hash it
                        let mut hasher = DefaultHasher::new();
                        arg_str.hash(&mut hasher);
                        let arg_hash = hasher.finish();

                        // Add it
                        arguments.push(arg_hash);

                        // Now we need to test if we have a comma or not
                        // If we do have a comma, we want to consume it, but if not, we want to
                        // leave it alone and let the rest of the logic deal with it
                        if test_next_is(
                            token_iter,
                            errors,
                            TokenKind::SymbolComma,
                            ErrorKind::UnexpectedEndOfDirectiveArguments,
                        )? {
                            token_iter.next();
                        }
                    }

                    // If we have reached here, then we have reached a ), so consume it
                    token_iter.next();
                }
            }
            // If it wasn't a (, or we are done parsing the arguments, then we have some tokens to
            // collect for invocation (maybe)

            // Just to make sure, check that we still have tokens
            if let Some(_) = token_iter.peek() {
                // If so, then collect all tokens until a newline or EOF
                while !test_next_is(
                    token_iter,
                    errors,
                    TokenKind::Newline,
                    ErrorKind::ShouldNotBeShown,
                )
                .unwrap_or(true)
                {
                    // Guaranteed to exist
                    let token = *token_iter.next().unwrap();

                    // Add it to the contents
                    contents.push(token);
                }

                // If we have arguments, but no contents
                if contents.len() == 0 && arguments.len() > 0 {
                    // Emit a warning
                    errors.add_assembly(AssemblyError::new(
                        ErrorKind::WarnEmptyDirectiveExpansionWithArgs,
                        identifier_token,
                    ));
                }
            } else {
                // If we don't have any more tokens, just for fun check if we actually captured any
                // arguments. If so, this is a weird definition
                if arguments.len() > 0 {
                    // Emit a warning
                    errors.add_assembly(AssemblyError::new(
                        ErrorKind::WarnEmptyDirectiveExpansionWithArgs,
                        identifier_token,
                    ));
                }
            }
        }

        let contents = if contents.len() > 0 {
            Some(contents)
        } else {
            None
        };

        Ok((
            Self {
                arguments,
                contents,
            },
            identifier,
        ))
    }
}
