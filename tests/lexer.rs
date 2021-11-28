use std::path::PathBuf;

use kasm::{
    errors::SourceFile,
    lexer::{Lexer, Token, TokenKind},
    session::Session,
    Config,
};

// Lexes a source string to a vector, but can panic
fn lex_from_text(source: &str) -> Vec<Token> {
    let config = Config {
        is_cli: true,
        emit_warnings: false,
        root_dir: PathBuf::new(),
        run_preprocessor: false,
        output_preprocessed: false,
    };

    let mut session = Session::new(config);

    // Create a SourceFile but with some dummy values
    let source_file = SourceFile::new("<input>".to_owned(), None, None, source.to_string(), 0);

    session.add_file(source_file);

    let primary_file = session.get_file(0).unwrap();

    // Create the lexer
    let lexer = Lexer::new(&primary_file.source, 0, &session);

    // Lex the tokens, if they are all valid
    let tokens = lexer.lex().expect("Lexing failed");

    tokens
}

#[test]
fn lex_operators() {
    let correct_kinds = vec![
        TokenKind::OperatorMinus,
        TokenKind::OperatorPlus,
        TokenKind::OperatorCompliment,
        TokenKind::OperatorMultiply,
        TokenKind::OperatorDivide,
        TokenKind::OperatorMod,
        TokenKind::OperatorAnd,
        TokenKind::OperatorOr,
        TokenKind::OperatorEquals,
        TokenKind::OperatorNotEquals,
        TokenKind::OperatorNegate,
        TokenKind::OperatorGreaterThan,
        TokenKind::OperatorLessThan,
        TokenKind::OperatorGreaterEquals,
        TokenKind::OperatorLessEquals,
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
        TokenKind::KeywordSection,
        TokenKind::KeywordText,
        TokenKind::KeywordData,
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
        TokenKind::DirectiveDefine,
        TokenKind::DirectiveMacro,
        TokenKind::DirectiveEndmacro,
        TokenKind::DirectiveRepeat,
        TokenKind::DirectiveEndRepeat,
        TokenKind::DirectiveInclude,
        TokenKind::DirectiveExtern,
        TokenKind::DirectiveGlobal,
        TokenKind::DirectiveLocal,
        TokenKind::DirectiveLine,
        TokenKind::DirectiveType,
        TokenKind::DirectiveValue,
        TokenKind::DirectiveUndef,
        TokenKind::DirectiveUnmacro,
        TokenKind::DirectiveFunc,
        TokenKind::DirectiveIf,
        TokenKind::DirectiveIfNot,
        TokenKind::DirectiveIfDef,
        TokenKind::DirectiveIfNotDef,
        TokenKind::DirectiveElseIf,
        TokenKind::DirectiveElseIfNot,
        TokenKind::DirectiveElseIfDef,
        TokenKind::DirectiveElseIfNotDef,
        TokenKind::DirectiveElse,
        TokenKind::DirectiveEndIf,
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
.undef
.unmacro
.func
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
        TokenKind::Identifier,
    ];

    let mut correct_iter = correct_kinds.iter();

    let source = "
_start:
.loopend:
loop_3231:
.endloop_3231:
.woohoo
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
        TokenKind::LiteralInteger,
        TokenKind::LiteralFloat,
        TokenKind::LiteralHex,
        TokenKind::LiteralHex,
        TokenKind::LiteralBinary,
        TokenKind::LiteralBinary,
        TokenKind::LiteralTrue,
        TokenKind::LiteralFalse,
        TokenKind::LiteralString,
        TokenKind::LiteralString,
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

    let mut token_iter = tokens.iter();

    while let Some(token) = token_iter.next() {
        let correct = *correct_iter.next().unwrap();

        assert_eq!(token.kind, correct);
    }
}

#[test]
fn lex_symbols() {
    let correct_kinds = vec![
        TokenKind::SymbolLeftParen,
        TokenKind::SymbolComma,
        TokenKind::SymbolHash,
        TokenKind::SymbolAt,
        TokenKind::SymbolAnd,
        TokenKind::SymbolRightParen,
        TokenKind::Comment,
    ];

    let mut correct_iter = correct_kinds.iter();

    let source = " ( , # @ & ) ; This is a comment";

    let tokens = lex_from_text(source);

    let mut token_iter = tokens.iter();

    while let Some(token) = token_iter.next() {
        assert_eq!(token.kind, TokenKind::Whitespace);

        let correct = *correct_iter.next().unwrap();
        let token = *token_iter.next().unwrap();

        assert_eq!(token.kind, correct);
    }
}
