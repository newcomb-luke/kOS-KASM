use std::path::PathBuf;

use kasm::{
    errors::SourceFile,
    lexer::{Lexer, Token, TokenKind},
    preprocessor::parser::parse_binary_literal,
    preprocessor::parser::parse_hexadecimal_literal,
    preprocessor::parser::parse_integer_literal,
    preprocessor::past::PASTNode,
    session::Session,
    Config,
};

use kasm::preprocessor::parser::Parser;

// Lexes a source string to a vector, but can panic
fn lex_from_text(source: &str) -> (Vec<Token>, Session) {
    let config = Config {
        is_cli: true,
        emit_warnings: false,
        root_dir: PathBuf::new(),
        run_preprocessor: false,
    };

    let mut session = Session::new(config);

    // Create a SourceFile but with some dummy values
    let source_file = SourceFile::new("<input>".to_owned(), None, None, source.to_string(), 0);

    session.add_file(source_file);

    let primary_file = session.get_file(0).unwrap();

    // Create the lexer
    let lexer = Lexer::new(&primary_file.source, session);

    // Lex the tokens, if they are all valid
    let (tokens, session) = lexer.lex().expect("Lexing failed");

    (tokens, session)
}

fn parse_source(source: &str) -> (Vec<PASTNode>, Session) {
    let (tokens, session) = lex_from_text(source);

    let preprocessor_parser = Parser::new(tokens, session);

    preprocessor_parser.parse().expect("Failed to parse")
}

#[test]
fn parse_int_literal() {
    let source = "23";

    let (nodes, session) = parse_source(source);

    assert_eq!(nodes.len(), 1);

    if let PASTNode::BenignTokens(benign_tokens) = nodes.first().unwrap() {
        let tokens = &benign_tokens.tokens;

        assert_eq!(tokens.len(), 1);

        if tokens.first().unwrap().kind == TokenKind::LiteralInteger {
            let snippet = session.span_to_snippet(&tokens.first().unwrap().as_span());
            let s = snippet.as_slice();

            let num = parse_integer_literal(s).expect(&format!("Invalid integer literal: {}", s));

            assert_eq!(num, 23);
        } else {
            panic!("BenignTokens did not contain a literal integer");
        }
    } else {
        panic!("PASTNode was not BenignTokens");
    }
}

#[test]
fn parse_hex_literal() {
    let source = "0x24 0x00_FF";

    let (nodes, session) = parse_source(source);

    assert_eq!(nodes.len(), 1);

    if let PASTNode::BenignTokens(benign_tokens) = nodes.first().unwrap() {
        let tokens = &benign_tokens.tokens;

        assert_eq!(tokens.len(), 3);

        let mut tokens = tokens.iter();

        let token = tokens.next().unwrap();
        if token.kind == TokenKind::LiteralHex {
            let snippet = session.span_to_snippet(&token.as_span());
            let s = snippet.as_slice();

            let num = parse_hexadecimal_literal(s).expect(&format!("Invalid hex literal: {}", s));

            assert_eq!(num, 0x24);
        }

        let token = tokens.next().unwrap();
        if token.kind != TokenKind::Whitespace {
            panic!("Token should have been whitespace");
        }

        let token = tokens.next().unwrap();
        if token.kind == TokenKind::LiteralHex {
            let snippet = session.span_to_snippet(&token.as_span());
            let s = snippet.as_slice();

            let num = parse_hexadecimal_literal(s).expect(&format!("Invalid hex literal: {}", s));

            assert_eq!(num, 0x00FF);
        }
    } else {
        panic!("PASTNode was not BenignTokens");
    }
}

#[test]
fn parse_bin_literal() {
    let source = "0b1101 0b0000_1111";

    let (nodes, session) = parse_source(source);

    assert_eq!(nodes.len(), 1);

    if let PASTNode::BenignTokens(benign_tokens) = nodes.first().unwrap() {
        let tokens = &benign_tokens.tokens;

        assert_eq!(tokens.len(), 3);

        let mut tokens = tokens.iter();

        let token = tokens.next().unwrap();
        if token.kind == TokenKind::LiteralBinary {
            let snippet = session.span_to_snippet(&token.as_span());
            let s = snippet.as_slice();

            let num = parse_binary_literal(s).expect(&format!("Invalid binary literal: {}", s));

            assert_eq!(num, 0b1101);
        }

        let token = tokens.next().unwrap();
        if token.kind != TokenKind::Whitespace {
            panic!("Token should have been whitespace");
        }

        let token = tokens.next().unwrap();
        if token.kind == TokenKind::LiteralBinary {
            let snippet = session.span_to_snippet(&token.as_span());
            let s = snippet.as_slice();

            let num = parse_binary_literal(s).expect(&format!("Invalid binary literal: {}", s));

            assert_eq!(num, 0b0000_1111);
        }
    } else {
        panic!("PASTNode was not BenignTokens");
    }
}