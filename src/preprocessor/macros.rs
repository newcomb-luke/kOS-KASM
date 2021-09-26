use crate::{
    errors::{AssemblyError, ErrorKind, ErrorManager, SourceFile},
    lexer::{
        parse_token, test_next_is,
        token::{Token, TokenKind},
        TokenIter,
    },
};

pub struct UnMacro {}

impl UnMacro {
    /// Parse an .unmacro directive
    pub fn parse(
        token_iter: &mut TokenIter,
        source_files: &mut Vec<SourceFile>,
        errors: &mut ErrorManager,
    ) -> Option<(String, Option<MacroArgs>)> {
        // .unmacro will always be there
        token_iter.next();

        // The identifier
        let identifier_token = parse_token(
            token_iter,
            errors,
            TokenKind::Identifier,
            ErrorKind::ExpectedUnmacroIdentifier,
            ErrorKind::MissingUnmacroIdentifier,
        )?;

        // Get the actual identifier out of it
        let identifier = identifier_token.slice(source_files).unwrap().to_string();

        // It is perfectly valid to end the file here, so we should test if there is even another
        // token
        if let Some(token) = token_iter.peek() {
            // Now there could either be a newline, or a number indicating a number of arguments
            if token.kind == TokenKind::Newline {
                // Consume it
                token_iter.next();

                Some((identifier, None))
            } else {
                // TODO: Support for binary and hex literals
                let args_min_token = parse_token(
                    token_iter,
                    errors,
                    TokenKind::LiteralInteger,
                    ErrorKind::ExpectedUnmacroNumArgs,
                    ErrorKind::ShouldNotBeShown,
                )?;

                let args_min = args_min_token
                    .slice(source_files)
                    .unwrap()
                    .parse::<usize>()
                    .unwrap();

                // See if we have another token at all
                if let Some(token) = token_iter.peek() {
                    // It could be a newline, or a '-', indicating a range
                    if token.kind == TokenKind::Newline {
                        let args = MacroArgs::Fixed(args_min);

                        Some((identifier, Some(args)))
                    } else {
                        // '-'?
                        parse_token(
                            token_iter,
                            errors,
                            TokenKind::OperatorMinus,
                            ErrorKind::ExpectedUnmacroRange,
                            ErrorKind::ShouldNotBeShown,
                        )?;

                        // Now there definitely should be a maximum number
                        let args_max_token = parse_token(
                            token_iter,
                            errors,
                            TokenKind::LiteralInteger,
                            ErrorKind::ExpectedUnmacroMaxArguments,
                            ErrorKind::MissingUnmacroMaxArguments,
                        )?;

                        let args_max = args_max_token
                            .slice(source_files)
                            .unwrap()
                            .parse::<usize>()
                            .unwrap();

                        let args = MacroArgs::Range(args_min, args_max, Vec::new());

                        Some((identifier, Some(args)))
                    }
                } else {
                    let args = MacroArgs::Fixed(args_min);

                    Some((identifier, Some(args)))
                }
            }
        } else {
            Some((identifier, None))
        }
    }
}

/// Represents possible numbers of macro arguments
///
/// Fixed represents a constant number of arguments
///
/// Range represents a range of amounts of arguments, the first field being the minimum, the second
/// being the maximum, and the third being a vector of token vectors that represent the defaults
/// for arguments not provided
#[derive(Debug)]
pub enum MacroArgs {
    Fixed(usize),
    Range(usize, usize, Vec<Vec<Token>>),
}

#[derive(Debug)]
pub struct Macro {
    args: Option<MacroArgs>,
    contents: Vec<Token>,
}

impl Macro {
    /// Parses a single macro from KASM tokens
    ///
    /// There are various types of macros and therefore a few different types of valid syntax:
    ///
    /// .macro HELP
    ///     push "HELP"
    /// .endmacro
    ///
    /// .macro ADD_CONST 1
    ///     push &1
    ///     add
    /// .endmacro
    ///
    /// .macro ADD 1-2 2
    ///     push &1
    ///     push &2
    ///     add
    /// .endmacro
    ///
    pub fn parse(
        token_iter: &mut TokenIter,
        source_files: &Vec<SourceFile>,
        errors: &mut ErrorManager,
    ) -> Option<(Self, String)> {
        // .macro will always be there
        token_iter.next();

        // The identifier
        let identifier_token = parse_token(
            token_iter,
            errors,
            TokenKind::Identifier,
            ErrorKind::ExpectedMacroIdentifier,
            ErrorKind::MissingMacroIdentifier,
        )?;

        // Get the actual identifier out of it
        let identifier = identifier_token.slice(source_files).unwrap().to_string();

        // If we have a newline, then this macro takes no arguments
        let args = if test_next_is(
            token_iter,
            errors,
            TokenKind::Newline,
            ErrorKind::MissingMacroContents,
        )? {
            // Consume the newline
            token_iter.next();

            None
        } else {
            // If we do have a token, and it isn't a newline, then it must be at least part of the
            // arguments
            // TODO: Binary and hex literal support
            let args_min_token = parse_token(
                token_iter,
                errors,
                TokenKind::LiteralInteger,
                ErrorKind::ExpectedMacroNumArguments,
                ErrorKind::ShouldNotBeShown,
            )?;

            let args_min = args_min_token
                .slice(source_files)
                .unwrap()
                .parse::<usize>()
                .unwrap();

            // Now we check if there is a -, because that means that this is in fact a token range
            if test_next_is(
                token_iter,
                errors,
                TokenKind::OperatorMinus,
                ErrorKind::MissingMacroContents,
            )? {
                // If so, collect that
                token_iter.next();

                // Now it should be another number that says the maximum number of arguments
                let args_max_token = parse_token(
                    token_iter,
                    errors,
                    TokenKind::LiteralInteger,
                    ErrorKind::ExpectedMacroMaxArguments,
                    ErrorKind::MissingMacroMaxArguments,
                )?;

                let args_max = args_max_token
                    .slice(source_files)
                    .unwrap()
                    .parse::<usize>()
                    .unwrap();

                // Check if max > min
                if args_max > args_min {
                    // Now we have to collect as many default arguments as there are optional
                    // arguments
                    let amount_optional = args_max - args_min;

                    let mut defaults = Vec::new();

                    // If the number of optional tokens is > 1, then run this loop. This loop
                    // checks for commas
                    for _ in 0..(amount_optional - 1) {
                        let mut default = Vec::new();

                        // While we haven't yet reached a comma
                        while !test_next_is(
                            token_iter,
                            errors,
                            TokenKind::SymbolComma,
                            ErrorKind::MissingMacroArgumentDefault,
                        )? {
                            default.push(*token_iter.next().unwrap());
                        }

                        // Consume the comma
                        token_iter.next();

                        defaults.push(default);
                    }

                    let mut last_default = Vec::new();

                    // At this point we are looking for one more default, but the end will be a
                    // newline
                    while !test_next_is(
                        token_iter,
                        errors,
                        TokenKind::Newline,
                        ErrorKind::MissingMacroArgumentDefault,
                    )? {
                        last_default.push(*token_iter.next().unwrap());
                    }

                    // Consume the newline
                    token_iter.next();

                    defaults.push(last_default);

                    Some(MacroArgs::Range(args_min, args_max, defaults))
                }
                // They could still be equal. If they are, we can just emit a warning and keep
                // going
                else if args_max == args_min {
                    errors.add_assembly(AssemblyError::new(
                        ErrorKind::WarnMacroMinMaxEqual,
                        args_max_token,
                    ));

                    Some(MacroArgs::Fixed(args_max))
                }
                // Definitely error
                else {
                    errors.add_assembly(AssemblyError::new(
                        ErrorKind::InvalidMacroArgumentsRange,
                        args_max_token,
                    ));

                    return None;
                }
            } else {
                // If not, then it should be a newline
                parse_token(
                    token_iter,
                    errors,
                    TokenKind::Newline,
                    ErrorKind::ExpectedMacroNewline,
                    ErrorKind::MissingMacroContents,
                )?;

                // Then, this is just the fixed number of arguments
                Some(MacroArgs::Fixed(args_min))
            }
        };

        let mut contents = Vec::new();

        // Now that we have parsed the arguments, we deal with the contents
        while !test_next_is(
            token_iter,
            errors,
            TokenKind::DirectiveEndmacro,
            ErrorKind::ExpectedEndMacro,
        )? {
            contents.push(*token_iter.next().unwrap());
        }

        // Consume the .endmacro
        token_iter.next();

        Some((Self { args, contents }, identifier))
    }

    pub fn args(&self) -> &Option<MacroArgs> {
        &self.args
    }
}
