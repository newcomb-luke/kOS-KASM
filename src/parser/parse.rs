use std::convert::TryFrom;

use kerbalobjects::{kofile::symbols::SymBind, KOSValue, Opcode};

use crate::{
    errors::Span,
    lexer::{Token, TokenKind},
    parser::{DeclaredSymbol, SymbolType},
    preprocessor::{
        evaluator::ExpressionEvaluator,
        expressions::{ExpressionParser, Value},
    },
    session::Session,
};

use super::{Label, LabelManager, SymbolManager, SymbolValue};

#[derive(Debug)]
pub struct ParsedFunction {
    pub name: String,
    pub instructions: Vec<ParsedInstruction>,
}

impl ParsedFunction {
    pub fn new(name: String, instructions: Vec<ParsedInstruction>) -> Self {
        Self { name, instructions }
    }
}

#[derive(Debug, Clone)]
pub enum ParsedInstruction {
    ZeroOp {
        opcode: Opcode,
        span: Span,
    },
    OneOp {
        opcode: Opcode,
        span: Span,
        operand: InstructionOperand,
    },
    TwoOp {
        opcode: Opcode,
        span: Span,
        operand1: InstructionOperand,
        operand2: InstructionOperand,
    },
}

impl ParsedInstruction {
    pub fn opcode(&self) -> Opcode {
        *match self {
            ParsedInstruction::ZeroOp { opcode, span: _ } => opcode,
            ParsedInstruction::OneOp {
                opcode,
                span: _,
                operand: _,
            } => opcode,
            ParsedInstruction::TwoOp {
                opcode,
                span: _,
                operand1: _,
                operand2: _,
            } => opcode,
        }
    }
}

#[derive(Debug, Clone)]
pub enum InstructionOperand {
    Integer(i32),
    String(String),
    Float(f64),
    Label(String),
    Bool(bool),
    Symbol(String),
    ArgMarker,
    Null,
}

impl InstructionOperand {
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Integer(_) => "integer",
            Self::String(_) => "string",
            Self::Float(_) => "float",
            Self::Label(_) => "label",
            Self::Bool(_) => "bool",
            Self::Symbol(_) => "symbol",
            Self::ArgMarker => "arg marker",
            Self::Null => "null",
        }
    }
}

pub type PResult = Result<(), ()>;

/// The parser for KASM instructions, symbols, and labels
pub struct Parser<'a> {
    tokens: Vec<Token>,
    token_cursor: usize,
    session: &'a Session,
    last_token: Option<Token>,
    symbol_manager: SymbolManager,
    label_manager: LabelManager,
    latest_label: String,
    instruction_count: usize,
    mode: Mode,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Mode {
    Text,
    Data,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token>, session: &'a Session) -> Self {
        Self {
            tokens,
            token_cursor: 0,
            session,
            last_token: None,
            symbol_manager: SymbolManager::new(),
            label_manager: LabelManager::new(),
            latest_label: String::new(),
            instruction_count: 0,
            mode: Mode::Text,
        }
    }

    /// Parses the provided tokens as functions and instructions.
    /// This also happens to execute all remaining assembler directives such as declaring symbols
    /// and their bindings. It produces a list of functions, as well as the symbols and labels that
    /// were encountered
    pub fn parse(mut self) -> Result<(Vec<ParsedFunction>, LabelManager, SymbolManager), ()> {
        let mut functions = Vec::new();

        // Skip until we get to a non-whitespace token
        self.skip_empty_lines();

        // According to the rules of KASM, the first token has to be a .func directive, or .section
        // directive
        while let Some(&next) = self.consume_next() {
            match next.kind {
                TokenKind::KeywordSection => {
                    self.parse_section(next.as_span())?;

                    self.assert_nothing_before_newline()?;
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

                    self.assert_nothing_before_newline()?;
                }
                TokenKind::DirectiveType => {
                    self.parse_type(next.as_span())?;

                    self.assert_nothing_before_newline()?;
                }
                TokenKind::DirectiveValue => {}
                TokenKind::DirectiveFunc => {
                    if self.mode == Mode::Data {
                        self.session
                            .struct_span_error(
                                next.as_span(),
                                "functions must be in a .text section".to_string(),
                            )
                            .emit();

                        return Err(());
                    } else {
                        let func = self.parse_function(next.as_span())?;

                        functions.push(func);
                    }
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
                        self.parse_data_entry(next.as_span())?;
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

            // Skip until we get to a non-whitespace token
            self.skip_empty_lines();
        }

        println!("-------------------------------------------------");

        for label in self.label_manager.labels() {
            println!("Label {} has value {}", label.0, label.1.value);
        }

        for (ident, symbol) in self.symbol_manager.symbols() {
            if symbol.sym_type == SymbolType::Default && symbol.binding == SymBind::Extern {
                self.session
                    .struct_span_error(
                        symbol.declared_span,
                        "external symbols must have the type specified".to_string(),
                    )
                    .emit();

                return Err(());
            }

            if symbol.value == SymbolValue::Undefined && symbol.binding != SymBind::Extern {
                self.session
                    .struct_span_error(
                        symbol.declared_span,
                        "symbol declared but never given a value".to_string(),
                    )
                    .emit();

                return Err(());
            }

            println!("Symbol {} : {:?}", ident, symbol);
        }

        Ok((functions, self.label_manager, self.symbol_manager))
    }

    fn parse_data_entry(&mut self, ident_span: Span) -> PResult {
        let ident_snippet = self.session.span_to_snippet(&ident_span);
        let ident_str = ident_snippet.as_slice().to_string();

        self.skip_whitespace();

        // Now we try to parse the data type and value
        let value = if let Some(&type_token) = self.consume_next() {
            let type_span = type_token.as_span();
            let type_snippet = self.session.span_to_snippet(&type_span);
            let type_str = type_snippet.as_slice();

            match type_token.kind {
                TokenKind::SymbolHash => {
                    self.assert_nothing_before_newline()?;

                    // Just the null symbol, no type needed
                    KOSValue::Null
                }
                TokenKind::SymbolAt => {
                    self.assert_nothing_before_newline()?;

                    // Just the argument marker symbol, no type needed
                    KOSValue::ArgMarker
                }
                other => {
                    self.skip_whitespace();

                    // In all other cases, we require this to be a data type
                    if matches!(
                        other,
                        TokenKind::TypeI8
                            | TokenKind::TypeI16
                            | TokenKind::TypeI32
                            | TokenKind::TypeI32V
                            | TokenKind::TypeF64
                            | TokenKind::TypeF64V
                            | TokenKind::TypeB
                            | TokenKind::TypeBV
                    ) {
                        // Parse a value as an expression
                        let value = self.parse_symbol_expression(type_span)?;

                        // If it is supposed to be a boolean
                        if matches!(other, TokenKind::TypeB | TokenKind::TypeBV) {
                            if let Value::Bool(b) = value {
                                if other == TokenKind::TypeB {
                                    KOSValue::Bool(b)
                                } else {
                                    KOSValue::BoolValue(b)
                                }
                            } else {
                                self.session
                                    .struct_span_error(
                                        type_span,
                                        format!("expected boolean value after type `{}`", type_str),
                                    )
                                    .emit();

                                return Err(());
                            }
                        } else if matches!(other, TokenKind::TypeF64 | TokenKind::TypeF64V) {
                            // If it is a floating point number
                            if let Value::Double(d) = value {
                                if other == TokenKind::TypeF64 {
                                    KOSValue::Double(d)
                                } else {
                                    KOSValue::ScalarDouble(d)
                                }
                            } else {
                                self.session
                                    .struct_span_error(
                                        type_span,
                                        format!("expected float value after type `{}`", type_str),
                                    )
                                    .emit();

                                return Err(());
                            }
                        } else {
                            // If it is a supposed to be an integer of some kind
                            if let Value::Int(i) = value {
                                if other == TokenKind::TypeI8 {
                                    if let Ok(i) = i8::try_from(i) {
                                        KOSValue::Byte(i)
                                    } else {
                                        self.session.struct_span_error(type_span, format!("value provided {} is too large to fit into a byte", i)).emit();
                                        return Err(());
                                    }
                                } else if other == TokenKind::TypeI16 {
                                    if let Ok(i) = i16::try_from(i) {
                                        KOSValue::Int16(i)
                                    } else {
                                        self.session.struct_span_error(type_span, format!("value provided {} is too large to fit into a 16-bit integer", i)).emit();
                                        return Err(());
                                    }
                                }
                                // These values have already been checked to fit because they
                                // needed to be parsed
                                else if other == TokenKind::TypeI32 {
                                    KOSValue::Int32(i)
                                } else {
                                    KOSValue::ScalarInt(i)
                                }
                            } else {
                                self.session
                                    .struct_span_error(
                                        type_span,
                                        format!("expected integer value after type `{}`", type_str),
                                    )
                                    .emit();

                                return Err(());
                            }
                        }
                    } else if matches!(other, TokenKind::TypeS | TokenKind::TypeSV) {
                        // If it is supposed to be a string
                        let value = if let Some(&s) = self.consume_next() {
                            if s.kind == TokenKind::LiteralString {
                                let value_snippet = self.session.span_to_snippet(&s.as_span());
                                let value_str = value_snippet.as_slice();
                                let value_str = (&value_str[1..value_str.len() - 1]).to_string();

                                if other == TokenKind::TypeS {
                                    KOSValue::String(value_str)
                                } else {
                                    KOSValue::StringValue(value_str)
                                }
                            } else {
                                self.session
                                    .struct_span_error(
                                        type_span,
                                        format!(
                                            "expected string literal after type `{}`",
                                            type_str
                                        ),
                                    )
                                    .emit();

                                return Err(());
                            }
                        } else {
                            self.session
                                .struct_span_error(
                                    type_span,
                                    format!("expected string literal after `{}`", type_str),
                                )
                                .emit();

                            return Err(());
                        };

                        self.assert_nothing_before_newline()?;

                        value
                    } else {
                        self.session
                            .struct_span_error(type_span, "expected symbol data type".to_string())
                            .emit();

                        return Err(());
                    }
                }
            }
        } else {
            self.session
                .struct_span_error(ident_span, "expected data type or value".to_string())
                .emit();

            return Err(());
        };

        if let Some(existing_symbol) = self.symbol_manager.get_mut(&ident_str) {
            if existing_symbol.value == SymbolValue::Undefined {
                if existing_symbol.binding != SymBind::Extern {
                    existing_symbol.value = SymbolValue::Value(value);

                    println!(
                        "Updated symbol in data section: {}. New value: {:?}",
                        ident_str, existing_symbol.value
                    );
                } else {
                    self.session
                        .struct_span_error(
                            ident_span,
                            "symbol declared to be external but provided value".to_string(),
                        )
                        .span_label(
                            existing_symbol.declared_span,
                            "first declared as external".to_string(),
                        )
                        .emit();

                    return Err(());
                }
            } else {
                self.session
                    .struct_span_error(ident_span, format!("symbol `{}` declared twice", ident_str))
                    .span_label(
                        existing_symbol.declared_span,
                        "initially declared here".to_string(),
                    )
                    .emit();

                return Err(());
            }
        } else {
            let new_symbol = DeclaredSymbol::new(
                ident_span,
                SymBind::Unknown,
                SymbolType::Value,
                SymbolValue::Value(value),
            );

            println!("Symbol in data section: {}", ident_str);

            self.symbol_manager.insert(ident_str, new_symbol);
        }

        Ok(())
    }

    fn parse_symbol_expression(&mut self, type_span: Span) -> Result<Value, ()> {
        let mut expression_tokens = Vec::new();

        while let Some(&expression_token) = self.consume_next() {
            if expression_token.kind == TokenKind::Newline {
                break;
            } else {
                expression_tokens.push(expression_token);
            }
        }

        if expression_tokens.is_empty() {
            self.session
                .struct_span_error(type_span, "expected symbol value".to_string())
                .emit();

            Err(())
        } else {
            let mut exp_tokens = expression_tokens.iter().peekable();
            let parsed_exp =
                match ExpressionParser::parse_expression(&mut exp_tokens, self.session, false) {
                    Ok(exp) => exp,
                    Err(mut db) => {
                        db.emit();

                        return Err(());
                    }
                };

            if let Some(exp) = parsed_exp {
                let evaluated = match ExpressionEvaluator::evaluate(&exp) {
                    Ok(exp) => exp,
                    Err(e) => {
                        let message = match e {
                            crate::preprocessor::evaluator::EvalError::NegateBool => {
                                "tried to apply operator - to boolean value"
                            }
                            crate::preprocessor::evaluator::EvalError::FlipDouble => {
                                "tried to apply operator ~ to double value"
                            }
                            crate::preprocessor::evaluator::EvalError::ZeroDivide => {
                                "tried to divide by zero"
                            }
                        };

                        self.session
                            .struct_span_error(
                                type_span,
                                format!("expression following this {}", message),
                            )
                            .emit();

                        return Err(());
                    }
                };

                Ok(evaluated)
            } else {
                self.session
                    .struct_bug(
                        "parsed expression is None despite having a first value".to_string(),
                    )
                    .emit();

                Err(())
            }
        }
    }

    fn parse_type(&mut self, type_span: Span) -> PResult {
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
                if symbol.sym_type == SymbolType::Default {
                    symbol.sym_type = sym_type;
                } else if symbol.sym_type == sym_type {
                    self.session
                        .struct_span_warn(
                            ident_token.as_span(),
                            "redundant .type declaration".to_string(),
                        )
                        .span_label(
                            symbol.declared_span,
                            "symbol inferred from this".to_string(),
                        )
                        .emit();
                } else {
                    self.session
                        .struct_span_error(
                            ident_token.as_span(),
                            "conflicting symbol types".to_string(),
                        )
                        .span_label(symbol.declared_span, "first declared here".to_string())
                        .emit();

                    return Err(());
                }

                println!("Symbol {} type is {:?}", ident_str, sym_type);
            } else {
                let declared_symbol = DeclaredSymbol::new(
                    ident_token.as_span(),
                    SymBind::Unknown,
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

    fn assert_nothing_before_newline(&mut self) -> PResult {
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

    fn parse_section(&mut self, section_span: Span) -> PResult {
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

    fn parse_binding(&mut self, span: Span, binding: SymBind) -> PResult {
        self.skip_whitespace();

        // The next token must be either a type, or an identifer
        let mut next = self.expect_consume_token(span, "expected either type or identifier")?;

        self.skip_whitespace();

        let mut gave_type = false;

        let sym_type = if next.kind == TokenKind::DirectiveValue {
            gave_type = true;
            SymbolType::Value
        } else if next.kind == TokenKind::DirectiveFunc {
            gave_type = true;
            SymbolType::Func
        } else if next.kind == TokenKind::Identifier {
            SymbolType::Default
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
        if let Some(declared_symbol) = self.symbol_manager.get_mut(&ident_string) {
            if declared_symbol.binding == binding {
                self.session
                    .struct_span_warn(
                        next.as_span(),
                        "redundant declaration of symbol binding".to_string(),
                    )
                    .emit();
            } else if declared_symbol.binding == SymBind::Unknown {
                if sym_type != SymbolType::Default {
                    if declared_symbol.sym_type != SymbolType::Default {
                        if declared_symbol.sym_type != sym_type {
                            self.session
                                .struct_span_error(
                                    next.as_span(),
                                    "conflicting symbol types".to_string(),
                                )
                                .span_label(
                                    declared_symbol.declared_span,
                                    "first declared here".to_string(),
                                )
                                .emit();

                            return Err(());
                        }
                    } else {
                        declared_symbol.sym_type = sym_type;
                    }
                }

                if declared_symbol.value != SymbolValue::Undefined && binding == SymBind::Extern {
                    self.session
                        .struct_span_error(
                            next.as_span(),
                            "symbol declared to be external".to_string(),
                        )
                        .span_label(
                            declared_symbol.declared_span,
                            "found to be declared locally".to_string(),
                        )
                        .emit();

                    return Err(());
                }

                declared_symbol.binding = binding;
            } else {
                self.session
                    .struct_span_error(next.as_span(), "conflicting symbol bindings".to_string())
                    .emit();

                return Err(());
            }
        } else {
            println!("Symbol declared: {}", ident_string);

            let declared_symbol =
                DeclaredSymbol::new(next.as_span(), binding, sym_type, SymbolValue::Undefined);

            self.symbol_manager.insert(ident_string, declared_symbol);
        }

        Ok(())
    }

    fn parse_function(&mut self, span: Span) -> Result<ParsedFunction, ()> {
        let mut instructions = Vec::new();

        self.skip_whitespace();

        self.struct_expected("newline", TokenKind::Newline, Some(span))?;

        let label = self.struct_expected("function label", TokenKind::Label, Some(span))?;
        let label_snippet = self.session.span_to_snippet(&label.as_span());
        let label_str = label_snippet.as_slice();
        let label_str = label_str[..label_str.len() - 1].to_string();

        println!("Parsing function: {}", label_str);

        self.declare_label(label.as_span(), false)?;

        if let Some(existing_symbol) = self.symbol_manager.get_mut(&label_str) {
            // If the symbol doesn't have a previously provided value
            if existing_symbol.value == SymbolValue::Undefined {
                // If the symbol type was previously .value (which has to be manually specified)
                if existing_symbol.sym_type == SymbolType::Value {
                    // Emit an error
                    self.session
                        .struct_span_error(label.as_span(), "conflicting symbol types".to_string())
                        .span_label(
                            existing_symbol.declared_span,
                            "first declared here".to_string(),
                        )
                        .emit();

                    return Err(());
                }

                // If this was declared to have a binding of "extern"
                if existing_symbol.binding == SymBind::Extern {
                    self.session
                        .struct_span_error(
                            label.as_span(),
                            "declared to be external, but found defined here".to_string(),
                        )
                        .span_label(
                            existing_symbol.declared_span,
                            "first declared here".to_string(),
                        )
                        .emit();

                    return Err(());
                }

                existing_symbol.sym_type = SymbolType::Func;
                existing_symbol.value = SymbolValue::Function;
            }
            // If it does have a previously defined value
            else {
                self.session
                    .struct_span_error(
                        label.as_span(),
                        "function declaraed with same name as existing symbol".to_string(),
                    )
                    .span_label(
                        existing_symbol.declared_span,
                        "first declared here".to_string(),
                    )
                    .emit();
                return Err(());
            }
        }
        // If this symbol doesn't already exist
        else {
            let declared_symbol = DeclaredSymbol::new(
                label.as_span(),
                SymBind::Unknown,
                SymbolType::Func,
                SymbolValue::Function,
            );

            self.symbol_manager
                .insert(label_str.clone(), declared_symbol);
        }

        self.latest_label = label_str.clone();

        let mut is_first = true;

        self.skip_empty_lines();

        while let Some(&next) = self.peek_next() {
            if !matches!(
                next.kind,
                TokenKind::Identifier | TokenKind::Label | TokenKind::InnerLabel
            ) {
                break;
            } else {
                let instr = self.parse_instruction(is_first)?;
                instructions.push(instr);
                self.instruction_count += 1;
                is_first = false;

                self.skip_empty_lines();
            }
        }

        println!("Function had {} instructions", instructions.len());

        Ok(ParsedFunction::new(label_str, instructions))
    }

    fn declare_label(&mut self, span: Span, inner: bool) -> PResult {
        let label = Label::new(self.instruction_count, span);
        let snippet = self.session.span_to_snippet(&span);
        let label_str = snippet.as_slice();

        let label_str = if inner {
            format!(
                "{}.{}",
                self.latest_label,
                &label_str[1..label_str.len() - 1]
            )
        } else {
            (&label_str[..label_str.len() - 1]).to_string()
        };

        if let Some(existing_label) = self.label_manager.get(&label_str) {
            // A label already existed with that name
            self.session
                .struct_span_error(span, "label with duplicate name found".to_string())
                .span_label(existing_label.span, "first declared here".to_string())
                .emit();

            Err(())
        } else {
            println!("New label declared: {}", label_str);

            self.label_manager.insert(label_str, label);

            Ok(())
        }
    }

    fn parse_instruction(&mut self, is_first: bool) -> Result<ParsedInstruction, ()> {
        let (opcode, opcode_span) = if is_first {
            self.parse_opcode(None, None)?
        } else {
            let next = *self.consume_next().unwrap();
            let next_span = next.as_span();

            if next.kind == TokenKind::Label {
                self.declare_label(next_span, false)?;

                self.skip_empty_lines();

                self.parse_opcode(None, Some(next_span))?
            } else if next.kind == TokenKind::InnerLabel {
                self.declare_label(next_span, true)?;

                self.skip_empty_lines();

                self.parse_opcode(None, Some(next_span))?
            } else {
                self.parse_opcode(Some(next), None)?
            }
        };

        self.skip_whitespace();

        println!("    Instruction was: {:?}", opcode);

        let mut operands = self.parse_operands()?;
        let provided_num = operands.len();

        let wanted_num = opcode.num_operands();

        if provided_num != wanted_num {
            self.session
                .struct_span_error(
                    opcode_span,
                    format!(
                        "{:?} requires {} argument{}, {} provided",
                        opcode,
                        wanted_num,
                        if wanted_num == 1 { "" } else { "s" },
                        provided_num
                    ),
                )
                .emit();

            return Err(());
        }

        println!("        Operands: {:?}", operands);

        let mut operands = operands.drain(..);

        Ok(match provided_num {
            0 => ParsedInstruction::ZeroOp {
                opcode,
                span: opcode_span,
            },
            1 => ParsedInstruction::OneOp {
                opcode,
                span: opcode_span,
                operand: operands.next().unwrap(),
            },
            _ => ParsedInstruction::TwoOp {
                opcode,
                span: opcode_span,
                operand1: operands.next().unwrap(),
                operand2: operands.next().unwrap(),
            },
        })
    }

    fn parse_operands(&mut self) -> Result<Vec<InstructionOperand>, ()> {
        let mut raw_operands = Vec::new();
        let mut operand = Vec::new();

        while let Some(&next) = self.consume_next() {
            if next.kind == TokenKind::Newline {
                break;
            } else if next.kind == TokenKind::SymbolComma {
                if operand.is_empty() {
                    self.session
                        .struct_span_error(
                            next.as_span(),
                            "expected operand before `,`".to_string(),
                        )
                        .emit();

                    return Err(());
                } else {
                    raw_operands.push(operand);
                    operand = Vec::new();
                }
            } else {
                operand.push(next);
            }

            self.skip_whitespace();
        }

        if !operand.is_empty() {
            raw_operands.push(operand);
        }

        let mut converted_operands = Vec::new();

        for raw in raw_operands {
            converted_operands.push(self.convert_operand(raw)?);
        }

        Ok(converted_operands)
    }

    fn convert_operand(&self, raw: Vec<Token>) -> Result<InstructionOperand, ()> {
        let first_token = raw.first().unwrap();
        let mut one_token = true;

        let operand = match first_token.kind {
            TokenKind::Identifier => {
                let snippet = self.session.span_to_snippet(&first_token.as_span());
                let identifier_str = snippet.as_slice().to_string();

                InstructionOperand::Symbol(identifier_str)
            }
            TokenKind::LiteralInteger
            | TokenKind::LiteralHex
            | TokenKind::LiteralBinary
            | TokenKind::LiteralTrue
            | TokenKind::LiteralFalse
            | TokenKind::LiteralFloat => {
                let mut exp_tokens = raw.iter().peekable();
                let parsed_exp = match ExpressionParser::parse_expression(
                    &mut exp_tokens,
                    self.session,
                    false,
                ) {
                    Ok(exp) => exp,
                    Err(mut db) => {
                        db.emit();

                        return Err(());
                    }
                };

                if let Some(exp) = parsed_exp {
                    let evaluated = match ExpressionEvaluator::evaluate(&exp) {
                        Ok(exp) => exp,
                        Err(e) => {
                            let message = match e {
                                crate::preprocessor::evaluator::EvalError::NegateBool => {
                                    "tried to apply operator - to boolean value"
                                }
                                crate::preprocessor::evaluator::EvalError::FlipDouble => {
                                    "tried to apply operator ~ to double value"
                                }
                                crate::preprocessor::evaluator::EvalError::ZeroDivide => {
                                    "tried to divide by zero"
                                }
                            };

                            self.session
                                .struct_span_error(
                                    first_token.as_span(),
                                    format!("expression following this {}", message),
                                )
                                .emit();

                            return Err(());
                        }
                    };

                    let operand = match evaluated {
                        Value::Int(i) => InstructionOperand::Integer(i),
                        Value::Bool(b) => InstructionOperand::Bool(b),
                        Value::Double(d) => InstructionOperand::Float(d),
                    };

                    one_token = false;

                    operand
                } else {
                    self.session
                        .struct_bug(
                            "parsed expression is None despite having a first value".to_string(),
                        )
                        .emit();

                    return Err(());
                }
            }
            TokenKind::SymbolAt => InstructionOperand::ArgMarker,
            TokenKind::SymbolHash => InstructionOperand::Null,
            TokenKind::InnerLabelReference => {
                let snippet = self.session.span_to_snippet(&first_token.as_span());
                let label = &snippet.as_slice()[1..];
                let combined_label = format!("{}.{}", self.latest_label, label);

                InstructionOperand::Label(combined_label)
            }
            TokenKind::LiteralString => {
                let snippet = self.session.span_to_snippet(&first_token.as_span());
                let inner = snippet.as_slice();
                let inner = &inner[1..inner.len() - 1];

                InstructionOperand::String(inner.to_string())
            }
            _ => {
                self.session
                    .struct_span_error(
                        first_token.as_span(),
                        "invalid token in instruction operand".to_string(),
                    )
                    .emit();

                return Err(());
            }
        };

        if one_token {
            if raw.len() > 1 {
                let unexpected = raw.get(1).unwrap();

                self.session
                    .struct_span_error(
                        unexpected.as_span(),
                        "expected comma after operand, found token".to_string(),
                    )
                    .emit();

                return Err(());
            }
        }

        Ok(operand)
    }

    fn parse_opcode(
        &mut self,
        token: Option<Token>,
        before: Option<Span>,
    ) -> Result<(Opcode, Span), ()> {
        let identifier_token = if let Some(token) = token {
            token
        } else {
            self.struct_expected("identifier", TokenKind::Identifier, before)?
        };

        let snippet = self.session.span_to_snippet(&identifier_token.as_span());
        let identifier_str = snippet.as_slice();

        let opcode = Opcode::from(identifier_str);

        if opcode == Opcode::Bogus {
            self.session
                .struct_span_error(
                    identifier_token.as_span(),
                    format!("expected instruction, found `{}`", identifier_str),
                )
                .emit();

            return Err(());
        }

        Ok((opcode, identifier_token.as_span()))
    }

    fn struct_expected(
        &mut self,
        expected: &str,
        kind: TokenKind,
        before: Option<Span>,
    ) -> Result<Token, ()> {
        if let Some(&next) = self.consume_next() {
            if next.kind == kind {
                Ok(next)
            } else {
                let found = if next.kind == TokenKind::Newline {
                    String::from("newline")
                } else {
                    let snippet = self.session.span_to_snippet(&next.as_span());
                    snippet.as_slice().to_string()
                };

                self.session
                    .struct_span_error(
                        next.as_span(),
                        format!("expected {}, found `{}`", expected, found),
                    )
                    .emit();

                Err(())
            }
        } else if let Some(before) = before {
            self.session
                .struct_span_error(before, format!("expected {}, found end of file", expected))
                .emit();

            Err(())
        } else {
            self.session
                .struct_bug("assumed EOF not possible while parsing".to_string())
                .emit();

            Err(())
        }
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
