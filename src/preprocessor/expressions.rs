use std::{iter::Peekable, slice::Iter};

use crate::{
    errors::DiagnosticBuilder,
    lexer::{Token, TokenKind},
    session::Session,
};

use super::parser::{
    parse_binary_literal, parse_float_literal, parse_hexadecimal_literal, parse_integer_literal,
};

pub type ExpResult<'a> = Result<Option<ExpNode>, DiagnosticBuilder<'a>>;
pub type TokenIter<'a> = Peekable<Iter<'a, Token>>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
    Int(i32),
    Double(f64),
    Bool(bool),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum UnOp {
    /// Arithmetic negation. Turns positive numbers negative, and negative numbers positive
    Negate,
    /// Flips all of the bits of the provided value
    Flip,
    /// Logical negation. !true = false, and !false = true
    Not,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BinOp {
    Add,
    Sub,
    Mult,
    Div,
    Mod,
    And,
    Or,
    Eq,
    Ne,
    Gt,
    Lt,
    Gte,
    Lte,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExpNode {
    BinOp(Box<ExpNode>, BinOp, Box<ExpNode>),
    UnOp(UnOp, Box<ExpNode>),
    Constant(Value),
}

// Generates binary operator parsing code, only suitable for extremely simple binary operators
macro_rules! gen_binop {
    ($tokens:ident, $session:ident, $func_name:ident, $token_kind:expr, $op_kind:expr) => {{
        Self::skip_whitespace($tokens);
        if let Some(mut lhs) = Self::$func_name($tokens, $session)? {
            Self::skip_whitespace($tokens);
            while let Some(&token) = $tokens.peek() {
                // See if there is the correct operator
                if token.kind == $token_kind {
                    // If it is, consume it
                    $tokens.next();

                    if let Some(rhs) = Self::$func_name($tokens, $session)? {
                        lhs = ExpNode::BinOp(Box::new(lhs), $op_kind, Box::new(rhs));
                    } else {
                        let db = $session
                            .struct_span_error(token.as_span(), "trailing operator".to_string());
                        return Err(db);
                    }
                }
                // If there isn't, break the loop
                else {
                    break;
                }
            }

            Ok(Some(lhs))
        } else {
            Ok(None)
        }
    }};
}

pub struct ExpressionParser {}

impl ExpressionParser {
    pub fn parse_expression<'a>(
        tokens: &mut TokenIter,
        session: &'a Session,
        nested: bool,
    ) -> ExpResult<'a> {
        let parsed = Self::parse_logical_or(tokens, session)?;

        if !nested {
            while let Some(token) = tokens.next() {
                if token.kind != TokenKind::Whitespace {
                    let db = session.struct_span_error(
                        token.as_span(),
                        "trailing token in expression".to_string(),
                    );

                    return Err(db);
                }
            }
        }

        Ok(parsed)
    }

    fn skip_whitespace(tokens: &mut TokenIter) {
        while let Some(token) = tokens.peek() {
            if token.kind != TokenKind::Whitespace {
                break;
            } else {
                tokens.next();
            }
        }
    }

    // Parses a logical or expression, or if none exists, parses the next lowest precidence
    fn parse_logical_or<'a>(tokens: &mut TokenIter, session: &'a Session) -> ExpResult<'a> {
        gen_binop!(
            tokens,
            session,
            parse_logical_and,
            TokenKind::OperatorOr,
            BinOp::Or
        )
    }

    // Parses a logical and expression, or if none exists, parses the next lowest precidence
    fn parse_logical_and<'a>(tokens: &mut TokenIter, session: &'a Session) -> ExpResult<'a> {
        gen_binop!(
            tokens,
            session,
            parse_equality_exp,
            TokenKind::OperatorAnd,
            BinOp::And
        )
    }

    // Parses an equality expression, or if none exists, parses the next lowest precidence
    fn parse_equality_exp<'a>(tokens: &mut TokenIter, session: &'a Session) -> ExpResult<'a> {
        Self::skip_whitespace(tokens);
        if let Some(mut lhs) = Self::parse_relational_exp(tokens, session)? {
            Self::skip_whitespace(tokens);
            while let Some(&&token) = tokens.peek() {
                // Check if it is an equality operator: ==, !=
                let op = match token.kind {
                    TokenKind::OperatorEquals => BinOp::Eq,
                    TokenKind::OperatorNotEquals => BinOp::Ne,
                    _ => {
                        break;
                    }
                };

                tokens.next();

                if let Some(rhs) = Self::parse_relational_exp(tokens, session)? {
                    lhs = ExpNode::BinOp(Box::new(lhs), op, Box::new(rhs));
                } else {
                    let db =
                        session.struct_span_error(token.as_span(), "trailing operator".to_string());
                    return Err(db);
                }
            }

            Ok(Some(lhs))
        } else {
            Ok(None)
        }
    }

    // Parses a relational expression, or if none exists, parses the next lowest precidence
    fn parse_relational_exp<'a>(tokens: &mut TokenIter, session: &'a Session) -> ExpResult<'a> {
        Self::skip_whitespace(tokens);
        if let Some(mut lhs) = Self::parse_additive_exp(tokens, session)? {
            Self::skip_whitespace(tokens);
            while let Some(&&token) = tokens.peek() {
                // Check if it is a relational operator: >, <, >=, or <=
                let op = match token.kind {
                    TokenKind::OperatorGreaterThan => BinOp::Gt,
                    TokenKind::OperatorLessThan => BinOp::Lt,
                    TokenKind::OperatorGreaterEquals => BinOp::Gte,
                    TokenKind::OperatorLessEquals => BinOp::Lte,
                    _ => {
                        break;
                    }
                };

                tokens.next();

                if let Some(rhs) = Self::parse_additive_exp(tokens, session)? {
                    lhs = ExpNode::BinOp(Box::new(lhs), op, Box::new(rhs));
                } else {
                    let db =
                        session.struct_span_error(token.as_span(), "trailing operator".to_string());
                    return Err(db);
                }
            }

            Ok(Some(lhs))
        } else {
            Ok(None)
        }
    }

    // Parses an additive expression, or if none exists, parses the next lowest precidence
    fn parse_additive_exp<'a>(tokens: &mut TokenIter, session: &'a Session) -> ExpResult<'a> {
        Self::skip_whitespace(tokens);
        if let Some(mut lhs) = Self::parse_term(tokens, session)? {
            Self::skip_whitespace(tokens);
            while let Some(&&token) = tokens.peek() {
                // Check if it is an additive operator: +/-
                let op = match token.kind {
                    TokenKind::OperatorPlus => BinOp::Add,
                    TokenKind::OperatorMinus => BinOp::Sub,
                    _ => {
                        break;
                    }
                };

                tokens.next();

                if let Some(rhs) = Self::parse_term(tokens, session)? {
                    lhs = ExpNode::BinOp(Box::new(lhs), op, Box::new(rhs));
                } else {
                    let db =
                        session.struct_span_error(token.as_span(), "trailing operator".to_string());
                    return Err(db);
                }
            }

            Ok(Some(lhs))
        } else {
            Ok(None)
        }
    }

    // Parses an expression term, or if none exists, parses the next lowest precidence
    fn parse_term<'a>(tokens: &mut TokenIter, session: &'a Session) -> ExpResult<'a> {
        Self::skip_whitespace(tokens);
        if let Some(mut lhs) = Self::parse_factor(tokens, session)? {
            Self::skip_whitespace(tokens);
            while let Some(&&token) = tokens.peek() {
                // Check if it is a multiplicative operator: * or /
                let op = match token.kind {
                    TokenKind::OperatorMultiply => BinOp::Mult,
                    TokenKind::OperatorDivide => BinOp::Div,
                    _ => {
                        break;
                    }
                };

                tokens.next();

                if let Some(rhs) = Self::parse_factor(tokens, session)? {
                    lhs = ExpNode::BinOp(Box::new(lhs), op, Box::new(rhs));
                } else {
                    let db =
                        session.struct_span_error(token.as_span(), "trailing operator".to_string());
                    return Err(db);
                }
            }

            Ok(Some(lhs))
        } else {
            Ok(None)
        }
    }

    // This function handles parsing the smallest unit of an expression. Either another expression
    // in parenthesis, or unary operations. It also parses constants.
    fn parse_factor<'a>(tokens: &mut TokenIter, session: &'a Session) -> ExpResult<'a> {
        Self::skip_whitespace(tokens);
        if let Some(&token) = tokens.next() {
            match token.kind {
                // (
                TokenKind::SymbolLeftParen => {
                    let inner_expression = Self::parse_expression(tokens, session, true)?;

                    Self::skip_whitespace(tokens);
                    if let Some(next) = tokens.next() {
                        if next.kind != TokenKind::SymbolRightParen {
                            println!("Token was: {:?}", next);
                            // Error
                            let db = session.struct_span_error(
                                next.as_span(),
                                "expected closing )".to_string(),
                            );

                            Err(db)
                        } else {
                            Ok(inner_expression)
                        }
                    } else {
                        // Error
                        let db = session
                            .struct_span_error(token.as_span(), "missing closing )".to_string());

                        Err(db)
                    }
                }
                // !, ~, -
                TokenKind::OperatorNegate
                | TokenKind::OperatorCompliment
                | TokenKind::OperatorMinus => {
                    let op = match token.kind {
                        TokenKind::OperatorNegate => UnOp::Not,
                        TokenKind::OperatorCompliment => UnOp::Flip,
                        TokenKind::OperatorMinus => UnOp::Negate,
                        _ => unreachable!(),
                    };

                    if let Some(factor) = Self::parse_factor(tokens, session)? {
                        Ok(Some(ExpNode::UnOp(op, Box::new(factor))))
                    } else {
                        let db = session.struct_span_error(
                            token.as_span(),
                            "operator with no expression".to_string(),
                        );

                        Err(db)
                    }
                }
                TokenKind::LiteralInteger | TokenKind::LiteralHex | TokenKind::LiteralBinary => {
                    let value_snippet = session.span_to_snippet(&token.as_span());
                    let value_str = value_snippet.as_slice();

                    if let Ok(value) = match token.kind {
                        TokenKind::LiteralInteger => parse_integer_literal(value_str),
                        TokenKind::LiteralHex => parse_hexadecimal_literal(value_str),
                        TokenKind::LiteralBinary => parse_binary_literal(value_str),
                        _ => unreachable!(),
                    } {
                        Ok(Some(ExpNode::Constant(Value::Int(value))))
                    } else {
                        let db = session.struct_span_error(
                            token.as_span(),
                            "literal too large to be stored".to_string(),
                        );

                        Err(db)
                    }
                }
                TokenKind::LiteralFloat => {
                    let value_snippet = session.span_to_snippet(&token.as_span());
                    let value_str = value_snippet.as_slice();

                    if let Ok(value) = parse_float_literal(value_str) {
                        Ok(Some(ExpNode::Constant(Value::Double(value))))
                    } else {
                        let db = session.struct_bug(format!("error parsing float {}", value_str));

                        Err(db)
                    }
                }
                TokenKind::LiteralTrue | TokenKind::LiteralFalse => Ok(Some(ExpNode::Constant(
                    Value::Bool(token.kind == TokenKind::LiteralTrue),
                ))),
                _ => {
                    let mut db = session
                        .struct_error("expected parenthesis, constant, or operator".to_string());

                    db.span_label(token.as_span(), "found invalid token".to_string());

                    Err(db)
                }
            }
        } else {
            Ok(None)
        }
    }
}
