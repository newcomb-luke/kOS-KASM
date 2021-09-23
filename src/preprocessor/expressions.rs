use crate::{
    errors::{ErrorKind, KASMError, KASMResult, SourceFile},
    lexer::{token::TokenKind, TokenIter},
};

#[derive(Debug, Clone, Copy)]
pub enum Value {
    Int(i32),
    Double(f64),
    Bool(bool),
}

impl From<Value> for bool {
    fn from(value: Value) -> Self {
        match value {
            Value::Int(i) => i != 0,
            Value::Double(d) => d != 0.0,
            Value::Bool(b) => b,
        }
    }
}

impl From<Value> for f64 {
    fn from(value: Value) -> Self {
        match value {
            Value::Int(i) => i as f64,
            Value::Double(d) => d,
            Value::Bool(b) => {
                if b {
                    1.0
                } else {
                    0.0
                }
            }
        }
    }
}

impl From<Value> for i32 {
    fn from(value: Value) -> Self {
        match value {
            Value::Int(i) => i,
            Value::Double(d) => d as i32,
            Value::Bool(b) => {
                if b {
                    1
                } else {
                    0
                }
            }
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match *self {
            // If this is an integer
            Value::Int(left) => {
                match *other {
                    // If the other is an integer
                    Value::Int(right) => {
                        // Compare them directly
                        left == right
                    }
                    // If the other is a double
                    Value::Double(right) => {
                        // Cast as an i32 then compare
                        left == right as i32
                    }
                    // If the other is a boolean, always return false
                    // The user must use a comparison operator to "turn" an integer into a boolean
                    Value::Bool(_) => false,
                }
            }
            Value::Double(left) => {
                match *other {
                    // If the other is an integer
                    Value::Int(right) => {
                        // Cast as an f64 then compare
                        left == right as f64
                    }
                    // If the other is a double
                    Value::Double(right) => {
                        // Compare them directly
                        left == right
                    }
                    // If the other is a boolean, always return false
                    // The user must use a comparison operator to "turn" a double into a boolean
                    Value::Bool(_) => false,
                }
            }
            Value::Bool(left) => {
                match *other {
                    // If the other is an integer or a double, return false. See above on
                    // Value::Bool
                    Value::Int(_) | Value::Double(_) => false,
                    Value::Bool(right) => {
                        // Compare them directly
                        left == right
                    }
                }
            }
        }
    }
}

impl Eq for Value {}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match *self {
            Value::Int(left) => match *other {
                Value::Int(right) => left.partial_cmp(&right),
                Value::Double(right) => left.partial_cmp(&(right as i32)),
                Value::Bool(_) => None,
            },
            Value::Double(left) => match *other {
                Value::Int(right) => left.partial_cmp(&(right as f64)),
                Value::Double(right) => left.partial_cmp(&right),
                Value::Bool(_) => None,
            },
            Value::Bool(left) => match *other {
                Value::Int(_) | Value::Double(_) => None,
                Value::Bool(right) => left.partial_cmp(&right),
            },
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum UnOp {
    Negate,
    Flip,
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

#[derive(Debug, Clone)]
pub enum ExpNode {
    BinOp(Box<ExpNode>, BinOp, Box<ExpNode>),
    UnOp(UnOp, Box<ExpNode>),
    Constant(Value),
}

pub struct ExpressionParser {}

impl ExpressionParser {
    /// Parses an expression in KASM source code
    pub fn parse_expression(
        token_iter: &mut TokenIter,
        source_files: &Vec<SourceFile>,
    ) -> KASMResult<ExpNode> {
        ExpressionParser::parse_logical_or(token_iter, source_files)
    }

    fn parse_logical_or(
        token_iter: &mut TokenIter,
        source_files: &Vec<SourceFile>,
    ) -> KASMResult<ExpNode> {
        let mut lhs = ExpressionParser::parse_logical_and(token_iter, source_files)?;

        while let Some(token) = token_iter.peek() {
            // If it is an Or operator
            if token.kind == TokenKind::OperatorOr {
                // If it is, consume it
                token_iter.next();

                let rhs = ExpressionParser::parse_logical_and(token_iter, source_files)?;

                lhs = ExpNode::BinOp(lhs.into(), BinOp::Or, rhs.into());
            }
            // If it isn't, break this loop
            else {
                break;
            }
        }

        Ok(lhs)
    }

    fn parse_logical_and(
        token_iter: &mut TokenIter,
        source_files: &Vec<SourceFile>,
    ) -> KASMResult<ExpNode> {
        let mut lhs = ExpressionParser::parse_equality_exp(token_iter, source_files)?;

        while let Some(token) = token_iter.peek() {
            // If it is an And operator
            if token.kind == TokenKind::OperatorAnd {
                // If it is, consume it
                token_iter.next();

                let rhs = ExpressionParser::parse_equality_exp(token_iter, source_files)?;

                lhs = ExpNode::BinOp(lhs.into(), BinOp::And, rhs.into());
            }
            // If it isn't, break this loop
            else {
                break;
            }
        }

        Ok(lhs)
    }

    fn parse_equality_exp(
        token_iter: &mut TokenIter,
        source_files: &Vec<SourceFile>,
    ) -> KASMResult<ExpNode> {
        let mut lhs = ExpressionParser::parse_relational_exp(token_iter, source_files)?;

        while let Some(token) = token_iter.peek() {
            // Check if it is an equality operator, == or !=
            let operator = match token.kind {
                TokenKind::OperatorEquals => BinOp::Eq,
                TokenKind::OperatorNotEquals => BinOp::Ne,
                _ => {
                    break;
                }
            };

            // Consume it
            token_iter.next();

            let rhs = ExpressionParser::parse_relational_exp(token_iter, source_files)?;

            lhs = ExpNode::BinOp(lhs.into(), operator, rhs.into());
        }

        Ok(lhs)
    }

    fn parse_relational_exp(
        token_iter: &mut TokenIter,
        source_files: &Vec<SourceFile>,
    ) -> KASMResult<ExpNode> {
        let mut lhs = ExpressionParser::parse_additive_exp(token_iter, source_files)?;

        while let Some(token) = token_iter.peek() {
            // Check if it is a relational operator, >, <, >=, or <=
            let operator = match token.kind {
                TokenKind::OperatorGreaterThan => BinOp::Gt,
                TokenKind::OperatorLessThan => BinOp::Lt,
                TokenKind::OperatorGreaterEquals => BinOp::Gte,
                TokenKind::OperatorLessEquals => BinOp::Lte,
                // If it isn't one
                _ => {
                    break;
                }
            };

            // Consume it
            token_iter.next();

            let rhs = ExpressionParser::parse_additive_exp(token_iter, source_files)?;

            lhs = ExpNode::BinOp(lhs.into(), operator, rhs.into());
        }

        Ok(lhs)
    }

    fn parse_additive_exp(
        token_iter: &mut TokenIter,
        source_files: &Vec<SourceFile>,
    ) -> KASMResult<ExpNode> {
        let mut lhs = ExpressionParser::parse_term(token_iter, source_files)?;

        while let Some(token) = token_iter.peek() {
            // Check if it is an additive operator: +/-
            let operator = match token.kind {
                TokenKind::OperatorPlus => BinOp::Add,
                TokenKind::OperatorMinus => BinOp::Sub,
                // If it isn't one
                _ => {
                    break;
                }
            };

            // Consume it
            token_iter.next();

            let rhs = ExpressionParser::parse_term(token_iter, source_files)?;

            lhs = ExpNode::BinOp(lhs.into(), operator, rhs.into());
        }

        Ok(lhs)
    }

    fn parse_term(
        token_iter: &mut TokenIter,
        source_files: &Vec<SourceFile>,
    ) -> KASMResult<ExpNode> {
        let mut lhs = ExpressionParser::parse_factor(token_iter, source_files)?;

        while let Some(token) = token_iter.peek() {
            // Check if it is either a * or / operator
            let operator = match token.kind {
                TokenKind::OperatorMultiply => BinOp::Mult,
                TokenKind::OperatorDivide => BinOp::Div,
                // If it isn't one
                _ => {
                    break;
                }
            };

            let rhs = ExpressionParser::parse_factor(token_iter, source_files)?;

            lhs = ExpNode::BinOp(lhs.into(), operator, rhs.into());
        }

        Ok(lhs)
    }

    fn parse_factor(
        token_iter: &mut TokenIter,
        source_files: &Vec<SourceFile>,
    ) -> KASMResult<ExpNode> {
        let token = match token_iter.peek() {
            Some(t) => t,
            None => {
                return Err(KASMError::new(
                    ErrorKind::UnexpectedEndOfExpression,
                    *token_iter.previous().unwrap(),
                ));
            }
        };

        match token.kind {
            // If the token is a (
            TokenKind::SymbolLeftParen => {
                // Consume it
                token_iter.next();

                let inner_expression = Self::parse_expression(token_iter, source_files)?;

                let token = match token_iter.peek() {
                    Some(t) => t,
                    None => {
                        return Err(KASMError::new(
                            ErrorKind::MissingClosingExpressionParen,
                            *token_iter.previous().unwrap(),
                        ));
                    }
                };

                // If it is a )
                if token.kind == TokenKind::SymbolRightParen {
                    // Consume it
                    token_iter.next();

                    Ok(inner_expression)
                } else {
                    Err(KASMError::new(
                        ErrorKind::MissingClosingExpressionParen,
                        *token,
                    ))
                }
            }
            TokenKind::OperatorNegate
            | TokenKind::OperatorCompliment
            | TokenKind::OperatorMinus => {
                // Get the token itself
                let token = token_iter.next().unwrap();

                // Convert the operator
                let operator = match token.kind {
                    TokenKind::OperatorNegate => UnOp::Not,
                    TokenKind::OperatorCompliment => UnOp::Flip,
                    TokenKind::OperatorMinus => UnOp::Negate,
                    _ => unreachable!(),
                };

                let factor = Self::parse_factor(token_iter, source_files)?;

                Ok(ExpNode::UnOp(operator, factor.into()))
            }
            TokenKind::LiteralInteger => {
                // Get the token itself
                let token = token_iter.next().unwrap();

                // Get the integer value out of it
                let value_str = token
                    .slice(source_files)
                    .ok_or(KASMError::new(ErrorKind::IntegerParse, *token))?;

                let value = i32::from_str_radix(value_str, 10)
                    .map_err(|_| KASMError::new(ErrorKind::IntegerParse, *token))?;

                Ok(ExpNode::Constant(Value::Int(value)).into())
            }
            TokenKind::LiteralFloat => {
                // Get the token itself
                let token = token_iter.next().unwrap();

                // Get the float value out of it
                let value_str = token
                    .slice(source_files)
                    .ok_or(KASMError::new(ErrorKind::FloatParse, *token))?;

                let value = value_str
                    .parse::<f64>()
                    .map_err(|_| KASMError::new(ErrorKind::FloatParse, *token))?;

                Ok(ExpNode::Constant(Value::Double(value)).into())
            }
            TokenKind::LiteralHex | TokenKind::LiteralString => {
                unimplemented!("Binary and hexadecimal literals will be supported")
            }
            TokenKind::LiteralTrue => Ok(ExpNode::Constant(Value::Bool(true))),
            TokenKind::LiteralFalse => Ok(ExpNode::Constant(Value::Bool(false))),
            _ => Err(KASMError::new(ErrorKind::InvalidTokenExpression, *token)),
        }
    }
}

#[test]
fn parse_addition() {
    use crate::lexer::token::Token;

    let source = "2 + 2";

    let source_file = SourceFile::new("test".to_string(), source.to_string());

    let source_files = vec![source_file];

    let tokens: Vec<Token> = crate::lexer::tokenize(source).collect();

    for token in tokens.iter() {
        println!("{:#?}", token);
    }

    let mut token_iter = TokenIter::new(tokens);

    match ExpressionParser::parse_expression(&mut token_iter, &source_files) {
        Ok(_) => {}
        Err(e) => {
            panic!("Parsing of 2 + 2 failed: {:#?}", e);
        }
    }
}

#[test]
fn parse_parens() {
    use crate::lexer::token::Token;

    let source = "(2+2) - 3";

    let source_file = SourceFile::new("test".to_string(), source.to_string());

    let source_files = vec![source_file];

    let tokens: Vec<Token> = crate::lexer::tokenize(source).collect();

    for token in tokens.iter() {
        println!("{:#?}", token);
    }

    let mut token_iter = TokenIter::new(tokens);

    match ExpressionParser::parse_expression(&mut token_iter, &source_files) {
        Ok(_) => {}
        Err(e) => {
            panic!("Parsing of (2 + 2) - 3 failed: {:#?}", e);
        }
    }
}
