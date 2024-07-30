use std::path::PathBuf;

use kasm::{
    errors::SourceFile,
    lexer::{Lexer, Literal, Token, TokenKind},
    preprocessor::{expressions::{BinOp, ExpNode, ExpressionParser, UnOp, Value}, parser::{parse_binary_literal, parse_hexadecimal_literal, parse_integer_literal}, past::{IncludePath, PASTNode}},
    session::Session,
    Config,
};

use kasm::preprocessor::parser::Parser;

// Lexes a source string to a vector, but can panic
fn lex_from_text(source: &str) -> (Vec<Token>, Session) {
    let config = Config {
        emit_errors: false,
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
    let tokens = lexer.lex().expect("Lexing failed");

    (tokens, session)
}

fn parse_source(source: &str) -> Result<(Vec<PASTNode>, Session), ()> {
    let (tokens, session) = lex_from_text(source);

    let preprocessor_parser = Parser::new(tokens, &session);

    let nodes = preprocessor_parser.parse().map_err(|_| ())?;

    Ok((nodes, session))
}

#[test]
fn parse_int_literal() {
    let source = "23";

    let (nodes, session) = parse_source(source).unwrap();

    assert_eq!(nodes.len(), 1);

    if let PASTNode::InertTokens(inert_tokens) = nodes.first().unwrap() {
        let tokens = &inert_tokens.tokens;

        assert_eq!(tokens.len(), 1);

        if tokens.first().unwrap().kind == TokenKind::Literal(Literal::Integer) {
            let snippet = session.span_to_snippet(&tokens.first().unwrap().as_span());
            let s = snippet.as_slice();

            let num = parse_integer_literal(s)
                .unwrap_or_else(|_| panic!("Invalid integer literal: {}", s));

            assert_eq!(num, 23);
        } else {
            panic!("BenignTokens did not contain a literal integer");
        }
    } else {
        panic!("PASTNode was not InertTokens");
    }
}

#[test]
fn parse_hex_literal() {
    let source = "0x24 0x00_FF";

    let (nodes, session) = parse_source(source).unwrap();

    assert_eq!(nodes.len(), 1);

    if let PASTNode::InertTokens(inert_tokens) = nodes.first().unwrap() {
        let tokens = &inert_tokens.tokens;

        assert_eq!(tokens.len(), 3);

        let mut tokens = tokens.iter();

        let token = tokens.next().unwrap();
        if token.kind == TokenKind::Literal(Literal::Hex) {
            let snippet = session.span_to_snippet(&token.as_span());
            let s = snippet.as_slice();

            let num = parse_hexadecimal_literal(s)
                .unwrap_or_else(|_| panic!("Invalid hex literal: {}", s));

            assert_eq!(num, 0x24);
        }

        let token = tokens.next().unwrap();
        if token.kind != TokenKind::Whitespace {
            panic!("Token should have been whitespace");
        }

        let token = tokens.next().unwrap();
        if token.kind == TokenKind::Literal(Literal::Hex) {
            let snippet = session.span_to_snippet(&token.as_span());
            let s = snippet.as_slice();

            let num = parse_hexadecimal_literal(s)
                .unwrap_or_else(|_| panic!("Invalid hex literal: {}", s));

            assert_eq!(num, 0x00FF);
        }
    } else {
        panic!("PASTNode was not InertTokens");
    }
}

#[test]
fn parse_bin_literal() {
    let source = "0b1101 0b0000_1111";

    let (nodes, session) = parse_source(source).unwrap();

    assert_eq!(nodes.len(), 1);

    if let PASTNode::InertTokens(inert_tokens) = nodes.first().unwrap() {
        let tokens = &inert_tokens.tokens;

        assert_eq!(tokens.len(), 3);

        let mut tokens = tokens.iter();

        let token = tokens.next().unwrap();
        if token.kind == TokenKind::Literal(Literal::Binary) {
            let snippet = session.span_to_snippet(&token.as_span());
            let s = snippet.as_slice();

            let num =
                parse_binary_literal(s).unwrap_or_else(|_| panic!("Invalid binary literal: {}", s));

            assert_eq!(num, 0b1101);
        }

        let token = tokens.next().unwrap();
        if token.kind != TokenKind::Whitespace {
            panic!("Token should have been whitespace");
        }

        let token = tokens.next().unwrap();
        if token.kind == TokenKind::Literal(Literal::Binary) {
            let snippet = session.span_to_snippet(&token.as_span());
            let s = snippet.as_slice();

            let num =
                parse_binary_literal(s).unwrap_or_else(|_| panic!("Invalid binary literal: {}", s));

            assert_eq!(num, 0b0000_1111);
        }
    } else {
        panic!("PASTNode was not InertTokens");
    }
}

#[test]
fn parse_expression() {
    let source = "!(2 == -(4 * 4))";

    let (nodes, session) = parse_source(source).unwrap();

    assert_eq!(nodes.len(), 1);

    if let PASTNode::InertTokens(inert_tokens) = nodes.first().unwrap() {
        let tokens = &inert_tokens.tokens;

        assert_eq!(tokens.len(), 15);

        let mut tokens = tokens.iter().peekable();

        match ExpressionParser::parse_expression(&mut tokens, &session, false) {
            Ok(expression) => match expression {
                Some(expression) => {
                    let correct = ExpNode::UnOp(
                        UnOp::Not,
                        Box::new(ExpNode::BinOp(
                            Box::new(ExpNode::Constant(Value::Int(2))),
                            BinOp::Eq,
                            Box::new(ExpNode::UnOp(
                                UnOp::Negate,
                                Box::new(ExpNode::BinOp(
                                    Box::new(ExpNode::Constant(Value::Int(4))),
                                    BinOp::Mult,
                                    Box::new(ExpNode::Constant(Value::Int(4))),
                                )),
                            )),
                        )),
                    );

                    println!("Expression: {:#?}", expression);
                    assert_eq!(correct, expression);
                }
                None => {
                    panic!("No expression parsed");
                }
            },
            Err(mut e) => {
                e.emit();

                panic!("Failed to parse expression");
            }
        }
    } else {
        panic!("PASTNode was not InertTokens");
    }
}

fn test_parse_include(source: &str, path: &str) {
    let (nodes, _) = parse_source(source).unwrap();

    assert_eq!(nodes.len(), 1);

    if let PASTNode::Include(include) = nodes.first().unwrap() {
        if let IncludePath::Literal(_, s) = &include.path {
            assert_eq!(s, path);
        } else {
            panic!("IncludePath was not Literal");
        }
    } else {
        panic!("PASTNode was not Include");
    }
}

#[test]
fn parse_include_literal() {
    let source = ".include \"test.kasm\"";
    test_parse_include(source, "test.kasm");
}

#[test]
fn parse_include_literal_with_trailing_whitespace() {
    let source = ".include \"test.kasm\"     ";
    test_parse_include(source, "test.kasm");
}

#[test]
fn parse_include_with_trailing_tokens() {
    let source = ".include \"test.kasm\" 2";
    assert!(parse_source(source).is_err());
}

#[test]
fn parse_include_without_path() {
    let source = ".include ";
    assert!(parse_source(source).is_err());
}

#[test]
fn parse_include_invalid_token() {
    let source = ".include 4";
    assert!(parse_source(source).is_err());
}