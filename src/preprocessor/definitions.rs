use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use crate::{
    errors::{AssemblyError, ErrorKind, ErrorManager, SourceFile},
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
    ) -> Option<String> {
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

        Some(identifier)
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
    pub fn parse(
        token_iter: &mut TokenIter,
        source_files: &Vec<SourceFile>,
        errors: &mut ErrorManager,
    ) -> Option<(Self, String)> {
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
                            ErrorKind::InvalidTokenDirectiveArguments,
                            ErrorKind::UnexpectedEndOfDirectiveArguments,
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

        Some((
            Self {
                arguments,
                contents,
            },
            identifier,
        ))
    }
}

fn _parse_definition(source: &str) -> Result<Definition, ErrorKind> {
    use crate::errors::KASMError;
    use crate::lexer::{self, check_errors};
    use crate::{phase0, phase1};

    let mut error_manager = ErrorManager::new();

    let source_file = SourceFile::new("test".to_string(), source.to_string());

    let source_files = vec![source_file];

    let mut tokens: Vec<Token> = lexer::tokenize(source_files.get(0).unwrap().source()).collect();

    check_errors(&tokens, &mut error_manager);

    if error_manager.errors().len() > 0 {
        return Err(match error_manager.errors().get(0).unwrap() {
            KASMError::Assembly(err) => err.kind(),
            _ => unreachable!(),
        });
    }

    // Run phase 0 of the assembly
    phase0(&mut tokens, &mut error_manager);

    if error_manager.errors().len() > 0 {
        return Err(match error_manager.errors().get(0).unwrap() {
            KASMError::Assembly(err) => err.kind(),
            _ => unreachable!(),
        });
    }
    // Run phase 1 of the assembly
    tokens = phase1(tokens);

    let mut token_iter = TokenIter::new(tokens);

    let (definition, _) =
        match Definition::parse(&mut token_iter, &source_files, &mut error_manager) {
            Some(def) => def,
            None => {
                let err = match error_manager.errors().get(0).unwrap() {
                    KASMError::Assembly(e) => e.kind(),
                    _ => unreachable!(),
                };

                return Err(err);
            }
        };

    Ok(definition)
}

#[test]
fn no_identifier() {
    match _parse_definition(".define") {
        Ok(_) => {
            panic!("Definition parsed successfully!!! BAD!!!!");
        }
        Err(e) => {
            assert_eq!(e, ErrorKind::MissingDirectiveIdentifier);
        }
    }
}

#[test]
fn no_closing_paren() {
    match _parse_definition(".define TEST(") {
        Ok(_) => {
            panic!("Definition parsed successfully!!! BAD!!!!");
        }
        Err(e) => {
            assert_eq!(e, ErrorKind::UnexpectedEndOfDirectiveArguments);
        }
    }
}

#[test]
fn valid_no_expansion() {
    match _parse_definition(".define FLAG") {
        Ok(_) => {}
        Err(e) => {
            panic!("Error: {:?}", e);
        }
    }
}

#[test]
fn valid_with_expansion() {
    match _parse_definition(".define NUM 24") {
        Ok(_) => {}
        Err(e) => {
            panic!("Error: {:?}", e);
        }
    }
}

#[test]
fn valid_with_arg() {
    match _parse_definition(".define SQUARE(x) x * x") {
        Ok(_) => {}
        Err(e) => {
            panic!("Error: {:?}", e);
        }
    }
}

#[test]
fn valid_with_args() {
    match _parse_definition(".define MULT(x, y) x * y") {
        Ok(_) => {}
        Err(e) => {
            panic!("Error: {:?}", e);
        }
    }
}
