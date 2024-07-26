use std::path::PathBuf;

use kasm::{
    errors::SourceFile,
    lexer::{Directive, IfDirective, Keyword, Lexer, Literal, Operator, PreprocessorDirective, Symbol, Token, TokenKind},
    session::Session,
    Config,
};

// Lexes a source string to a vector, but can panic
fn lex_from_text(source: &str) -> Vec<Token> {
    let config = Config {
        emit_errors: true,
        emit_warnings: false,
        root_dir: PathBuf::new(),
        run_preprocessor: false,
        preprocess_only: false,
        include_path: None,
        file_sym_name: None,
        comment: String::new(),
    };

    let mut session = Session::new(config);

    // Create a SourceFile but with some dummy values
    let source_file = SourceFile::new("<input>".to_owned(), None, None, source.to_string(), 0);

    session.add_file(source_file);

    let primary_file = session.get_file(0).unwrap();

    // Create the lexer
    let lexer = Lexer::new(&primary_file.source, 0, &session);

    // Lex the tokens, if they are all valid
    lexer.lex().expect("Lexing failed")
}

#[test]
fn lex_operators() {
    let correct_kinds = vec![
        TokenKind::Operator(Operator::Minus),
        TokenKind::Operator(Operator::Plus),
        TokenKind::Operator(Operator::Compliment),
        TokenKind::Operator(Operator::Multiply),
        TokenKind::Operator(Operator::Divide),
        TokenKind::Operator(Operator::Mod),
        TokenKind::Operator(Operator::And),
        TokenKind::Operator(Operator::Or),
        TokenKind::Operator(Operator::Equals),
        TokenKind::Operator(Operator::NotEquals),
        TokenKind::Operator(Operator::Negate),
        TokenKind::Operator(Operator::GreaterThan),
        TokenKind::Operator(Operator::LessThan),
        TokenKind::Operator(Operator::GreaterEquals),
        TokenKind::Operator(Operator::LessEquals),
    ];

    let mut correct_iter = correct_kinds.iter();

    let source = " - + ~ * / % && || == != ! > < >= <=";

    let tokens = lex_from_text(source);

    let mut token_iter = tokens.iter();

    while let Some(token) = token_iter.next() {
        assert_eq!(token.kind, TokenKind::Whitespace);

        let correct = *correct_iter.next().unwrap();
        let token = *token_iter.next().unwrap();

        assert_eq!(token.kind, correct);
    }
}

#[test]
fn lex_keywords() {
    let correct_kinds = vec![
        TokenKind::Keyword(Keyword::Section),
        TokenKind::Keyword(Keyword::Text),
        TokenKind::Keyword(Keyword::Data),
    ];

    let mut correct_iter = correct_kinds.iter();

    let source = "\n.section\n.text\n.data";

    let tokens = lex_from_text(source);

    let mut token_iter = tokens.iter();

    while let Some(token) = token_iter.next() {
        assert_eq!(token.kind, TokenKind::Newline);

        let correct = *correct_iter.next().unwrap();
        let token = *token_iter.next().unwrap();

        assert_eq!(token.kind, correct);
    }
}

#[test]
fn lex_directives() {
    let correct_kinds = vec![
        TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::Define)),
        TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::Macro)),
        TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::EndMacro)),
        TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::Repeat)),
        TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::EndRepeat)),
        TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::Include)),
        TokenKind::Directive(Directive::Extern),
        TokenKind::Directive(Directive::Global),
        TokenKind::Directive(Directive::Local),
        TokenKind::Directive(Directive::Line),
        TokenKind::Directive(Directive::Type),
        TokenKind::Directive(Directive::Value),
        TokenKind::Directive(Directive::Func),
        TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::Undef)),
        TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::Unmacro)),
        TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::If(IfDirective::If))),
        TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::If(IfDirective::IfNot))),
        TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::If(IfDirective::IfDef))),
        TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::If(IfDirective::IfNotDef))),
        TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::If(IfDirective::ElseIf))),
        TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::If(IfDirective::ElseIfNot))),
        TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::If(IfDirective::ElseIfDef))),
        TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::If(IfDirective::ElseIfNotDef))),
        TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::If(IfDirective::Else))),
        TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::If(IfDirective::EndIf))),
    ];

    let mut correct_iter = correct_kinds.iter();

    let source = "
.define
.macro
.endmacro
.rep
.endrep
.include
.extern
.global
.local
.line
.type
.value
.func
.undef
.unmacro
.if
.ifn
.ifdef
.ifndef
.elif
.elifn
.elifdef
.elifndef
.else
.endif";

    let tokens = lex_from_text(source);

    let mut token_iter = tokens.iter();

    while let Some(token) = token_iter.next() {
        assert_eq!(token.kind, TokenKind::Newline);

        let correct = *correct_iter.next().unwrap();
        let token = *token_iter.next().unwrap();

        assert_eq!(token.kind, correct);
    }
}

#[test]
fn lex_labels() {
    let correct_kinds = vec![
        TokenKind::Label,
        TokenKind::InnerLabel,
        TokenKind::Label,
        TokenKind::InnerLabel,
        TokenKind::InnerLabelReference,
        TokenKind::MacroArgmentReference,
        TokenKind::Identifier,
    ];

    let mut correct_iter = correct_kinds.iter();

    let source = "
_start:
.loopend:
loop_3231:
.endloop_3231:
.woohoo
&21
loop_3231";

    let tokens = lex_from_text(source);

    let mut token_iter = tokens.iter();

    while let Some(token) = token_iter.next() {
        assert_eq!(token.kind, TokenKind::Newline);

        let correct = *correct_iter.next().unwrap();
        let token = *token_iter.next().unwrap();

        assert_eq!(token.kind, correct);
    }
}

#[test]
fn lex_literals() {
    let correct_kinds = vec![
        TokenKind::Literal(Literal::Integer),
        TokenKind::Literal(Literal::Float),
        TokenKind::Literal(Literal::Hex),
        TokenKind::Literal(Literal::Hex),
        TokenKind::Literal(Literal::Binary),
        TokenKind::Literal(Literal::Binary),
        TokenKind::Literal(Literal::True),
        TokenKind::Literal(Literal::False),
        TokenKind::Literal(Literal::String),
        TokenKind::Literal(Literal::String),
    ];

    let mut correct_iter = correct_kinds.iter();

    let source = "
244
9.81
0xfa
0x00_0a
0b00000001
0b0110_0001
true
false
\"Hello world\"
\"\\tThe man said \\\"Who goes there?\\\"\\n\"";

    let tokens = lex_from_text(source);

    let mut token_iter = tokens.iter();

    while let Some(token) = token_iter.next() {
        assert_eq!(token.kind, TokenKind::Newline);

        let correct = *correct_iter.next().unwrap();
        let token = *token_iter.next().unwrap();

        assert_eq!(token.kind, correct);
    }
}

#[test]
fn lex_delimiters() {
    let correct_kinds = vec![
        TokenKind::Whitespace,
        TokenKind::Newline,
        TokenKind::Backslash,
    ];

    let mut correct_iter = correct_kinds.iter();

    let source = " \n\\";

    let tokens = lex_from_text(source);

    let token_iter = tokens.iter();

    for token in token_iter {
        let correct = *correct_iter.next().unwrap();

        assert_eq!(token.kind, correct);
    }
}

#[test]
fn lex_symbols() {
    let correct_kinds = vec![
        TokenKind::Symbol(Symbol::LeftParen),
        TokenKind::Symbol(Symbol::Comma),
        TokenKind::Symbol(Symbol::Hash),
        TokenKind::Symbol(Symbol::At),
        TokenKind::Symbol(Symbol::RightParen),
        TokenKind::Comment,
    ];

    let mut correct_iter = correct_kinds.iter();

    let source = " ( , # @ ) ; This is a comment";

    let tokens = lex_from_text(source);

    let mut token_iter = tokens.iter();

    while let Some(token) = token_iter.next() {
        assert_eq!(token.kind, TokenKind::Whitespace);

        let correct = *correct_iter.next().unwrap();
        let token = *token_iter.next().unwrap();

        assert_eq!(token.kind, correct);
    }
}
