use std::collections::HashMap;

use kerbalobjects::kofile::symbols::SymBind;

use crate::{
    errors::Span,
    lexer::{Token, TokenKind},
    preprocessor::parser::{
        parse_binary_literal, parse_float_literal, parse_hexadecimal_literal, parse_integer_literal,
    },
    session::Session,
};

pub struct DeclaredSymbol {
    pub declared_span: Span,
    pub binding: SymBind,
    pub sym_type: SymbolType,
    pub value: SymbolValue,
}

impl DeclaredSymbol {
    pub fn new(span: Span, binding: SymBind, sym_type: SymbolType, value: SymbolValue) -> Self {
        Self {
            declared_span: span,
            binding,
            sym_type,
            value,
        }
    }
}

const DEFAULT_TYPE: SymbolType = SymbolType::Func;
const DEFAULT_BINDING: SymBind = SymBind::Local;

pub struct SymbolManager {
    map: HashMap<String, DeclaredSymbol>,
}

impl SymbolManager {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn contains(&mut self, identifier: &String) -> bool {
        self.map.contains_key(identifier)
    }

    pub fn get(&self, identifier: &String) -> Option<&DeclaredSymbol> {
        self.map.get(identifier)
    }

    pub fn get_mut(&mut self, identifier: &String) -> Option<&mut DeclaredSymbol> {
        self.map.get_mut(identifier)
    }

    pub fn insert(&mut self, identifier: String, declared: DeclaredSymbol) {
        self.map.insert(identifier, declared);
    }
}

impl Default for SymbolManager {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Parser<'a> {
    tokens: Vec<Token>,
    token_cursor: usize,
    session: &'a Session,
    last_token: Option<Token>,
    symbol_manager: SymbolManager,
    mode: Mode,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Mode {
    Text,
    Data,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SymbolType {
    Func,
    Value,
}

#[derive(Debug, PartialEq, Clone)]
pub enum SymbolValue {
    Integer(i32),
    String(String),
    Float(f64),
    Location(usize),
    Bool(bool),
    Undefined,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token>, session: &'a Session) -> Self {
        Self {
            tokens,
            token_cursor: 0,
            session,
            last_token: None,
            symbol_manager: SymbolManager::new(),
            mode: Mode::Text,
        }
    }

    pub fn parse(&mut self) -> Result<(), ()> {
        // Skip until we get to a non-whitespace token
        self.skip_empty_lines();

        // According to the rules of KASM, the first token has to be a .func directive, or .section
        // directive
        while let Some(&next) = self.consume_next() {
            match next.kind {
                TokenKind::KeywordSection => {
                    self.parse_section(next.as_span())?;
                }
                TokenKind::DirectiveExtern
                | TokenKind::DirectiveGlobal
                | TokenKind::DirectiveLocal => {
                    let binding = match next.kind {
                        TokenKind::DirectiveExtern => SymBind::Extern,
                        TokenKind::DirectiveGlobal => SymBind::Global,
                        TokenKind::DirectiveLocal => SymBind::Local,
                        _ => unreachable!(),
                    };

                    self.parse_binding(next.as_span(), binding)?;
                }
                TokenKind::DirectiveType => {
                    self.parse_type(next.as_span())?;
                }
                TokenKind::DirectiveValue => {}
                TokenKind::DirectiveFunc => {
                    unimplemented!()
                }
                TokenKind::Identifier => {
                    if self.mode == Mode::Text {
                        self.session
                            .struct_span_error(
                                next.as_span(),
                                "instruction found outside of function".to_string(),
                            )
                            .help("try adding .func before your first label".to_string())
                            .emit();

                        return Err(());
                    } else {
                        let ident_snippet = self.session.span_to_snippet(&next.as_span());
                        let ident_str = ident_snippet.as_slice().to_string();

                        self.skip_whitespace();

                        let new_value = if let Some(&value_token) = self.consume_next() {
                            let value_snippet =
                                self.session.span_to_snippet(&value_token.as_span());

                            let value_str = value_snippet.as_slice();

                            if value_token.kind == TokenKind::LiteralTrue {
                                SymbolValue::Bool(true)
                            } else if value_token.kind == TokenKind::LiteralFalse {
                                SymbolValue::Bool(false)
                            } else if value_token.kind == TokenKind::LiteralInteger {
                                let int_val = parse_integer_literal(value_str)?;
                                SymbolValue::Integer(int_val)
                            } else if value_token.kind == TokenKind::LiteralHex {
                                let int_val = parse_hexadecimal_literal(value_str)?;
                                SymbolValue::Integer(int_val)
                            } else if value_token.kind == TokenKind::LiteralBinary {
                                let int_val = parse_binary_literal(value_str)?;
                                SymbolValue::Integer(int_val)
                            } else if value_token.kind == TokenKind::LiteralFloat {
                                let float_val = parse_float_literal(value_str)?;
                                SymbolValue::Float(float_val)
                            } else if value_token.kind == TokenKind::LiteralString {
                                SymbolValue::String(value_str.to_string())
                            } else {
                                self.session
                                    .struct_span_error(
                                        value_token.as_span(),
                                        format!("expected symbol value, found `{}`", value_str),
                                    )
                                    .emit();

                                return Err(());
                            }
                        } else {
                            self.session
                                .struct_span_error(
                                    next.as_span(),
                                    "expected symbol value".to_string(),
                                )
                                .emit();

                            return Err(());
                        };

                        if let Some(existing_symbol) = self.symbol_manager.get_mut(&ident_str) {
                            if existing_symbol.value == SymbolValue::Undefined {
                                existing_symbol.value = new_value;

                                println!("Updated symbol in data section: {}", ident_str);
                            } else {
                                self.session
                                    .struct_span_error(
                                        next.as_span(),
                                        format!("symbol `{}` declared twice", ident_str),
                                    )
                                    .span_label(
                                        existing_symbol.declared_span,
                                        "initially declared here".to_string(),
                                    )
                                    .emit();

                                return Err(());
                            }
                        } else {
                            let new_symbol = DeclaredSymbol::new(
                                next.as_span(),
                                DEFAULT_BINDING,
                                SymbolType::Value,
                                new_value,
                            );

                            println!("Symbol in data section: {}", ident_str);

                            self.symbol_manager.insert(ident_str, new_symbol);
                        }
                    }
                }
                _ => {
                    self.session
                        .struct_span_error(
                            next.as_span(),
                            "expected instruction, function, or label".to_string(),
                        )
                        .emit();

                    return Err(());
                }
            }

            self.assert_nothing_before_newline()?;

            // Skip until we get to a non-whitespace token
            self.skip_empty_lines();
        }

        todo!();
    }

    fn parse_type(&mut self, type_span: Span) -> Result<(), ()> {
        self.skip_whitespace();

        let type_token = self.expect_consume_token(type_span, "expected symbol type")?;

        let sym_type = if type_token.kind == TokenKind::DirectiveFunc {
            SymbolType::Func
        } else if type_token.kind == TokenKind::DirectiveValue {
            SymbolType::Value
        } else {
            let type_snippet = self.session.span_to_snippet(&type_token.as_span());
            let type_str = type_snippet.as_slice();

            self.session
                .struct_span_error(
                    type_token.as_span(),
                    format!("expected symbol type, found {}", type_str),
                )
                .emit();

            return Err(());
        };

        self.skip_whitespace();

        let ident_token = self.expect_consume_token(type_token.as_span(), "expected identifier")?;

        if ident_token.kind == TokenKind::Identifier {
            let ident_snippet = self.session.span_to_snippet(&ident_token.as_span());
            let ident_str = ident_snippet.as_slice().to_string();

            if let Some(symbol) = self.symbol_manager.get_mut(&ident_str) {
                symbol.sym_type = sym_type;

                println!("Symbol {} type is {:?}", ident_str, sym_type);
            } else {
                let declared_symbol = DeclaredSymbol::new(
                    ident_token.as_span(),
                    DEFAULT_BINDING,
                    sym_type,
                    SymbolValue::Undefined,
                );

                println!("Symbol {} type is {:?}", ident_str, sym_type);

                self.symbol_manager.insert(ident_str, declared_symbol);
            }

            Ok(())
        } else {
            let token_snippet = self.session.span_to_snippet(&ident_token.as_span());
            let token_str = token_snippet.as_slice();

            self.session
                .struct_span_error(
                    ident_token.as_span(),
                    format!("expected identifier, found {}", token_str),
                )
                .emit();

            Err(())
        }
    }

    fn assert_nothing_before_newline(&mut self) -> Result<(), ()> {
        while let Some(&token) = self.consume_next() {
            if token.kind == TokenKind::Newline {
                return Ok(());
            } else if token.kind != TokenKind::Whitespace {
                self.session
                    .struct_span_error(token.as_span(), "newline expected".to_string())
                    .emit();

                return Err(());
            }
        }

        // An EOF is actually okay
        Ok(())
    }

    fn expect_consume_token(&mut self, before_span: Span, message: &str) -> Result<Token, ()> {
        if let Some(&token) = self.consume_next() {
            Ok(token)
        } else {
            self.session
                .struct_span_error(before_span, message.to_string())
                .emit();

            Err(())
        }
    }

    fn parse_section(&mut self, section_span: Span) -> Result<(), ()> {
        self.skip_whitespace();

        let mode_token = self.expect_consume_token(section_span, "expected section type")?;

        if mode_token.kind == TokenKind::KeywordText {
            self.mode = Mode::Text;
        } else if mode_token.kind == TokenKind::KeywordData {
            self.mode = Mode::Data;
        } else {
            let mode_token_snippet = self.session.span_to_snippet(&mode_token.as_span());
            let mode_token_str = mode_token_snippet.as_slice();

            self.session
                .struct_span_error(
                    mode_token.as_span(),
                    format!("expected section type, found `{}`", mode_token_str),
                )
                .emit();

            return Err(());
        }

        Ok(())
    }

    fn parse_binding(&mut self, span: Span, binding: SymBind) -> Result<(), ()> {
        self.skip_whitespace();

        // The next token must be either a type, or an identifer
        let mut next = self.expect_consume_token(span, "expected either type or identifier")?;

        self.skip_whitespace();

        let mut gave_type = false;

        // The default for values is being a Func
        let sym_type = if next.kind == TokenKind::DirectiveValue {
            gave_type = true;
            SymbolType::Value
        } else if next.kind == TokenKind::DirectiveFunc {
            gave_type = true;
            SymbolType::Func
        } else if next.kind == TokenKind::Identifier {
            DEFAULT_TYPE
        } else {
            self.session
                .struct_span_error(
                    next.as_span(),
                    "expected either type or identifier".to_string(),
                )
                .emit();
            return Err(());
        };

        if gave_type {
            self.skip_whitespace();

            let mut error_span = None;

            next = match self.consume_next() {
                Some(t) => *t,
                None => {
                    error_span = Some(next.as_span());

                    next
                }
            };

            if error_span.is_none() && next.kind != TokenKind::Identifier {
                error_span = Some(next.as_span());
            }

            if let Some(span) = error_span {
                self.session
                    .struct_span_error(span, "expected identifier".to_string())
                    .emit();

                return Err(());
            }
        }

        let ident_snippet = self.session.span_to_snippet(&next.as_span());
        let ident_string = ident_snippet.as_slice().to_string();

        // Because this is a declaration of a symbol we should check if this symbol was
        // previously declared
        if self.symbol_manager.contains(&ident_string) {
            let declared_symbol = self.symbol_manager.get(&ident_string).unwrap();

            self.session
                .struct_span_error(next.as_span(), "duplicate symbol name".to_string())
                .span_label(
                    declared_symbol.declared_span,
                    "previously defined here".to_string(),
                )
                .emit();

            return Err(());
        }

        println!("Symbol declared: {}", ident_string);

        let declared_symbol =
            DeclaredSymbol::new(next.as_span(), binding, sym_type, SymbolValue::Undefined);

        self.symbol_manager.insert(ident_string, declared_symbol);

        Ok(())
    }

    fn parse_function(&mut self) -> () {
        todo!();
    }

    fn parse_instruction(&mut self) -> () {
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

    // Skips all whitespace if there is any, including newlines
    //
    // Returns true if there was any, false if not
    fn skip_empty_lines(&mut self) -> bool {
        let mut was_empty = false;

        while let Some(next) = self.peek_next() {
            if next.kind == TokenKind::Whitespace || next.kind == TokenKind::Newline {
                was_empty = true;

                // Increment the token cursor
                self.token_cursor += 1;
            } else {
                break;
            }
        }

        was_empty
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
}
