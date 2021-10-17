use std::{collections::hash_map::DefaultHasher, hash::Hasher, num::NonZeroU8};

type PResult<T> = Result<T, ()>;

use kerbalobjects::Opcode;

use crate::{
    errors::{DiagnosticBuilder, Span},
    lexer::{Token, TokenKind},
    preprocessor::past::{BenignTokens, SLMacroDef},
    session::Session,
};

use super::past::{
    Ident, MLMacroArgs, MLMacroUndef, MacroInvok, MacroInvokArgs, PASTNode, SLMacroDefArgs,
    SLMacroDefContents, SLMacroUndef, SLMacroUndefArgs,
};

pub struct Parser {
    tokens: Vec<Token>,
    token_cursor: usize,
    session: Session,
    last_token: Option<Token>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>, session: Session) -> Self {
        let first_token = tokens.get(0).copied();

        Self {
            tokens,
            token_cursor: 0,
            session,
            last_token: first_token,
        }
    }

    pub fn parse(&mut self) -> PResult<Vec<PASTNode>> {
        let mut nodes = Vec::new();

        while let Some(&next) = self.peek_next() {
            match next.kind {
                TokenKind::Whitespace => {
                    self.skip_whitespace();
                }
                TokenKind::Newline => {
                    self.consume_next();
                }
                _ => {
                    // Parse only one for now
                    let node = self.parse_bit()?;
                    nodes.push(node);
                }
            }
        }

        Ok(nodes)
    }

    // This usually parses a line, but in the case of any multi-line construct, this parses more
    // than that
    //
    // This assumes that there is a next token to parse, so keep that in mind
    //
    fn parse_bit(&mut self) -> PResult<PASTNode> {
        let next = *self.peek_next().unwrap();

        match next.kind {
            TokenKind::DirectiveDefine => self.parse_sl_macro_def(),
            TokenKind::DirectiveUndef => self.parse_sl_macro_undef(),
            TokenKind::DirectiveUnmacro => self.parse_ml_macro_undef(),
            _ => {
                println!("Token was: {:?}", next.kind);
                return Err(());
            }
        }
    }

    // Parse a multi line macro undefinition
    //
    // See the MLMacroUndef grammar
    //
    fn parse_ml_macro_undef(&mut self) -> PResult<PASTNode> {
        let mut span = Span::new(0, 0, 0);

        // Consume the .unmacro
        let unmacro_span = self.assert_next(TokenKind::DirectiveUnmacro)?;

        // Copy the span values
        span.start = unmacro_span.start;
        span.file = unmacro_span.file;

        // Skip any whitespace
        self.skip_whitespace();

        // As per the grammar, the next token MUST be an identifier
        let identifier = self.parse_ident()?;

        // Now parse the optional arguments/range
        let args = match self.parse_ml_macro_args()? {
            Some(args) => args,
            None => MLMacroArgs::new(identifier.span, 0, None),
        };

        // Adjust the span
        span.end = args.span.end;

        Ok(PASTNode::MLMacroUndef(MLMacroUndef::new(
            span, identifier, args,
        )))
    }

    // Parse a multi line macro's number of arguments
    fn parse_ml_macro_args(&mut self) -> PResult<Option<MLMacroArgs>> {
        // Skip any whitespace
        self.skip_whitespace();

        // If we have any tokens at all
        let required = if let Some(&next) = self.peek_next() {
            // If it is not a newline, then it _should_ be the number of arguments
            if next.kind == TokenKind::Newline {
                self.assert_next(TokenKind::Newline)?;

                // No number of arguments
                None
            } else {
                // Parse the number
                Some(self.parse_num_arguments()?)
            }
        } else {
            None
        };

        if let Some((required_span, required_num)) = required {
            // If we had the first number, there might be `-` and then another.
            let maximum = if let Some(&next) = self.peek_next() {
                // First test for a newline
                if next.kind == TokenKind::Newline {
                    self.assert_next(TokenKind::Newline)?;

                    None
                }
                // If it isn't a `-`
                else if next.kind != TokenKind::OperatorMinus {
                    // This is an error. With only the number of required arguments specified, we don't
                    // have any default arguments to take in, so there should be nothing there
                    self.struct_err_expected_found(next.as_span(), "`-` or a newline")
                        .emit();

                    return Err(());
                } else {
                    // We have a `-`
                    self.assert_next(TokenKind::OperatorMinus)?;

                    // Now parse the next number then
                    let (span, num) = self.parse_num_arguments()?;

                    // This has the additional bound that it must be > # of required arguments
                    if num <= required_num {
                        self.session
                            .struct_span_error(
                                span,
                                format!(
                                    "maximum must be greater than number of required ({})",
                                    required_num
                                ),
                            )
                            .emit();

                        return Err(());
                    }

                    // SAFETY: We just checked if this was greater the number of required
                    // arguments, which has to be at least 0, so this has to be >= 1
                    Some((span, unsafe { NonZeroU8::new_unchecked(num) }))
                }
            } else {
                None
            };

            let mut span = Span::new(required_span.start, 0, required_span.file);

            if let Some((max_span, max_num)) = maximum {
                span.end = max_span.end;

                Ok(Some(MLMacroArgs::new(span, required_num, Some(max_num))))
            } else {
                Ok(Some(MLMacroArgs::new(span, required_num, None)))
            }
        }
        // If we didn't get a number of arguments at all, give the default of 0
        else {
            Ok(Some(MLMacroArgs::new(
                self.last_token.unwrap().as_span(),
                0,
                None,
            )))
        }
    }

    // Parse a single line macro undefinition
    //
    // See the SLMacroUndef grammar
    //
    fn parse_sl_macro_undef(&mut self) -> PResult<PASTNode> {
        let mut span = Span::new(0, 0, 0);

        // Consume the .undef
        let undef_span = self.assert_next(TokenKind::DirectiveUndef)?;

        // Copy the span values
        span.start = undef_span.start;
        span.file = undef_span.file;

        // Skip any whitespace
        self.skip_whitespace();

        // As per the grammar, the next token MUST be an identifier
        let identifier = self.parse_ident()?;

        // Now parse the optional number of arguments
        let args = match self.parse_sl_macro_undef_args()? {
            Some(args) => args,
            None => SLMacroUndefArgs::new(identifier.span, 0),
        };

        // Adjust the span
        span.end = args.span.end;

        Ok(PASTNode::SLMacroUndef(SLMacroUndef::new(
            span, identifier, args,
        )))
    }

    // Parses the number of arguments in a single line macro undefinition
    fn parse_sl_macro_undef_args(&mut self) -> PResult<Option<SLMacroUndefArgs>> {
        // Skip any whitespace
        self.skip_whitespace();

        // If we have any tokens at all
        if let Some(&next) = self.peek_next() {
            // If it is not a newline, then it _should_ be the number of arguments
            if next.kind == TokenKind::Newline {
                self.assert_next(TokenKind::Newline)?;

                // No number of arguments
                Ok(None)
            } else {
                // Parse the number
                let (span, num) = self.parse_num_arguments()?;

                Ok(Some(SLMacroUndefArgs::new(span, num)))
            }
        } else {
            Ok(None)
        }
    }

    // Parses a number of arguments, which is basically just a number that has the additional
    // requirement of being less than 255. This will also emit a special diagnostic that says that
    // macro expansions are not allowed in this place.
    fn parse_num_arguments(&mut self) -> PResult<(Span, u8)> {
        let (span, num) = match self.parse_number() {
            Ok(data) => Ok(data),
            Err((mut db, data)) => {
                if let Some((string, token)) = data {
                    // If we actually got an identifier
                    if token.kind == TokenKind::Identifier {
                        // If it isn't an instruction
                        if Opcode::from(string.as_str()) == Opcode::Bogus {
                            db.help("macros expansions are not allowed here".to_string());
                        }
                    }
                }

                db.emit();

                Err(())
            }
        }?;

        if num > u8::MAX as i32 {
            self.session
                .struct_span_error(
                    span,
                    "number greater than the maximum number of arguments (255)".to_string(),
                )
                .emit();

            Err(())
        } else {
            Ok((span, num as u8))
        }
    }

    // Parses a number from the source. This could be a hexadecimal, binary, or decimal number
    //
    // i32 is the return type because that is the maximum value that any kOS value can have, and it
    // works for our purposes as well
    //
    fn parse_number(
        &mut self,
    ) -> Result<(Span, i32), (DiagnosticBuilder<'_>, Option<(String, Token)>)> {
        if let Some(&token) = self.consume_next() {
            let span = token.as_span();
            let snippet = self.session.span_to_snippet(&span);
            let string = snippet.as_slice().to_string();

            match token.kind {
                TokenKind::LiteralInteger => {
                    if let Ok(num) = parse_integer_literal(&string) {
                        Ok((span, num))
                    } else {
                        Err((
                            self.session.struct_span_error(
                                span,
                                format!("number too large to be stored {}", string),
                            ),
                            Some((string, token)),
                        ))
                    }
                }
                TokenKind::LiteralHex => {
                    if let Ok(num) = parse_hexadecimal_literal(&string) {
                        Ok((span, num))
                    } else {
                        Err((
                            self.session.struct_span_error(
                                span,
                                format!("number too large to be stored {}", string),
                            ),
                            Some((string, token)),
                        ))
                    }
                }
                TokenKind::LiteralBinary => {
                    if let Ok(num) = parse_binary_literal(&string) {
                        Ok((span, num))
                    } else {
                        Err((
                            self.session.struct_span_error(
                                span,
                                format!("number too large to be stored {}", string),
                            ),
                            Some((string, token)),
                        ))
                    }
                }
                _ => Err((
                    self.struct_err_expected_found(token.as_span(), "number"),
                    Some((string, token)),
                )),
            }
        } else {
            Err((
                self.struct_err_expected_eof(self.last_token.unwrap().as_span(), "number"),
                None,
            ))
        }
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

        // Skip any whitespace
        self.skip_whitespace();

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

                    // We could have whitespace before this which shouldn't matter
                    self.skip_whitespace();

                    // It should now be an identifier
                    let ident = self.parse_ident()?;
                    arguments.push(ident);

                    // We could also have whitespace after it
                    self.skip_whitespace();

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
        // Skip any whitespace
        self.skip_whitespace();

        // We basically let anything in except for preprocessor directives
        // We do let in:
        //  Macro invokations
        //  Other directives

        // We will tell if an identifier is an instruction or a macro invokation by checking it
        // against the names of instructions

        // If we do have any token
        if let Some(&next) = self.peek_next() {
            // If it is a newline, this doesn't count as contents
            if next.kind == TokenKind::Newline {
                self.assert_next(TokenKind::Newline)?;
                return Ok(None);
            }

            let mut span = Span::new(0, 0, 0);
            let first_span = next.as_span();
            span.file = first_span.file;
            span.start = first_span.start;

            let mut contents = Vec::new();
            let mut benign_tokens = Vec::new();

            // Now that we know there is other stuff, loop until a newline/EOF
            while let Some(&next) = self.consume_next() {
                match next.kind {
                    TokenKind::DirectiveIf
                    | TokenKind::DirectiveIfNot
                    | TokenKind::DirectiveIfDef
                    | TokenKind::DirectiveIfNotDef
                    | TokenKind::DirectiveEndIf
                    | TokenKind::DirectiveElse
                    | TokenKind::DirectiveElseIf
                    | TokenKind::DirectiveElseIfNot
                    | TokenKind::DirectiveElseIfDef
                    | TokenKind::DirectiveElseIfNotDef
                    | TokenKind::DirectiveMacro
                    | TokenKind::DirectiveEndmacro
                    | TokenKind::DirectiveRepeat
                    | TokenKind::DirectiveEndRepeat
                    | TokenKind::DirectiveDefine
                    | TokenKind::DirectiveUndef
                    | TokenKind::DirectiveUnmacro
                    | TokenKind::DirectiveInclude => {
                        self.session
                            .struct_span_error(
                                next.as_span(),
                                "preprocessor directives not allowed in single line macros"
                                    .to_string(),
                            )
                            .emit();

                        return Err(());
                    }
                    TokenKind::Newline => break,
                    TokenKind::Identifier => {
                        let snippet = self.session.span_to_snippet(&next.as_span());
                        let ident_str = snippet.as_slice();

                        // Tests if this is an instruction or not
                        if Opcode::from(ident_str) != Opcode::Bogus {
                            // If it is
                            // Just push it
                            benign_tokens.push(next);

                            span.end = next.as_span().end;
                        } else {
                            // If it isn't, it is going to be parsed as a macro invokation
                            let macro_invok = self.parse_macro_invok(next.as_span(), ident_str)?;

                            // If we have captured any tokens before this
                            if benign_tokens.len() > 0 {
                                let benign_tokens_node = BenignTokens::from_vec(benign_tokens);
                                contents.push(PASTNode::BenignTokens(benign_tokens_node));

                                benign_tokens = Vec::new();
                            }

                            // Update this just in case it is the last part of the contents
                            span.end = macro_invok.span.end;

                            contents.push(PASTNode::MacroInvok(macro_invok));
                        }
                    }
                    _ => {
                        // Just push this, it is allowed and not special
                        benign_tokens.push(next);

                        // Just in case this is the last one
                        span.end = next.as_span().end;
                    }
                }
            }

            // Check if benign_tokens didn't end empty
            if benign_tokens.len() > 0 {
                contents.push(PASTNode::BenignTokens(BenignTokens::from_vec(
                    benign_tokens,
                )));
            }
            Ok(Some(SLMacroDefContents::new(span, contents)))
        } else {
            Ok(None)
        }
    }

    // Parses a macro invokation
    fn parse_macro_invok(&mut self, ident_span: Span, ident_str: &str) -> PResult<MacroInvok> {
        let mut hasher = DefaultHasher::new();
        hasher.write(ident_str.as_bytes());
        let hash = hasher.finish();
        let mut span = Span::new(ident_span.start, 0, ident_span.file);

        let identifier = Ident::new(ident_span, hash);

        // After the identifier, there could be arguments, or not
        let was_whitespace = self.skip_whitespace();

        if was_whitespace {
            span.end = ident_span.end;

            Ok(MacroInvok::new(span, identifier, None))
        } else {
            let args = self.parse_macro_invok_args()?;

            Ok(MacroInvok::new(span, identifier, Some(args)))
        }
    }

    // Parses a macro invokation's arguments
    fn parse_macro_invok_args(&mut self) -> PResult<MacroInvokArgs> {
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

/// Parses an integer literal from the given &str
///
/// This differs from the normal &str::parse() because it supports random `_` characters in the
/// integer. They allow for more easily readable constants
///
pub fn parse_integer_literal(string: &str) -> Result<i32, ()> {
    // This makes sure we only have to allocate once
    let mut no_separators = String::with_capacity(string.len());

    for c in string.chars() {
        if c.is_digit(10) {
            no_separators.push(c);
        } else if c != '_' {
            return Err(());
        }
    }

    Ok(no_separators.parse().unwrap())
}

/// Parses a hexadecimal literal from the given &str
pub fn parse_hexadecimal_literal(string: &str) -> Result<i32, ()> {
    let mut no_separators = String::with_capacity(string.len());

    for c in string.chars() {
        if c.is_digit(16) {
            no_separators.push(c);
        } else if c != '_' {
            return Err(());
        }
    }

    Ok(no_separators.parse().unwrap())
}

/// Parses a binary literal from the given &str
pub fn parse_binary_literal(string: &str) -> Result<i32, ()> {
    let mut no_separators = String::with_capacity(string.len());

    for c in string.chars() {
        if c == '0' || c == '1' {
            no_separators.push(c);
        } else if c != '_' {
            return Err(());
        }
    }

    Ok(no_separators.parse().unwrap())
}
