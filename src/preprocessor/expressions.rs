use std::{error::Error, iter::Peekable, slice::Iter};

use crate::Token;

#[derive(Debug)]
pub enum Value {
    Int(i32),
    Double(f64),
    Bool(bool),
    Id(String),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ValueType {
    INT,
    DOUBLE,
    BOOL,
}

impl Value {
    pub fn to_bool(&self) -> Result<bool, Box<dyn Error>> {
        match self {
            Value::Int(i) => Ok(*i > 0),
            Value::Double(_) => Ok(true),
            Value::Bool(b) => Ok(*b),
            _ => Err("Cannot directly convert identifier to boolean".into()),
        }
    }

    pub fn to_double(&self) -> Result<f64, Box<dyn Error>> {
        match self {
            Value::Int(i) => Ok(*i as f64),
            Value::Double(d) => Ok(*d),
            Value::Bool(b) => Ok(if *b { 1.0 } else { 0.0 }),
            _ => Err("Cannot directly convert identifier to double".into()),
        }
    }

    pub fn to_int(&self) -> Result<i32, Box<dyn Error>> {
        match self {
            Value::Int(i) => Ok(*i),
            Value::Double(d) => Ok(*d as i32),
            Value::Bool(b) => Ok(if *b { 1 } else { 0 }),
            _ => Err("Cannot directly convert identifier to integer".into()),
        }
    }

    pub fn valtype(&self) -> ValueType {
        match self {
            Value::Int(_) => ValueType::INT,
            Value::Double(_) => ValueType::DOUBLE,
            Value::Bool(_) => ValueType::BOOL,
            _ => unreachable!(),
        }
    }

    pub fn equals(&self, other: &Value) -> bool {
        let ltype = self.valtype();

        let rtype = other.valtype();

        if ltype != rtype {
            false
        } else {
            if ltype == ValueType::INT {
                let v1 = match self {
                    Value::Int(i) => *i,
                    _ => unreachable!(),
                };
                let v2 = match other {
                    Value::Int(i) => *i,
                    _ => unreachable!(),
                };
                v1 == v2
            } else if ltype == ValueType::DOUBLE {
                let v1 = match self {
                    Value::Double(d) => *d,
                    _ => unreachable!(),
                };
                let v2 = match other {
                    Value::Double(d) => *d,
                    _ => unreachable!(),
                };
                v1 == v2
            } else {
                let v1 = match self {
                    Value::Bool(b) => *b,
                    _ => unreachable!(),
                };
                let v2 = match other {
                    Value::Bool(b) => *b,
                    _ => unreachable!(),
                };
                v1 == v2
            }
        }
    }

    pub fn greater_than(&self, other: &Value) -> bool {
        let ltype = self.valtype();

        let rtype = other.valtype();

        if ltype != rtype {
            false
        } else {
            if ltype == ValueType::INT {
                let v1 = match self {
                    Value::Int(i) => *i,
                    _ => unreachable!(),
                };
                let v2 = match other {
                    Value::Int(i) => *i,
                    _ => unreachable!(),
                };
                v1 > v2
            } else if ltype == ValueType::DOUBLE {
                let v1 = match self {
                    Value::Double(d) => *d,
                    _ => unreachable!(),
                };
                let v2 = match other {
                    Value::Double(d) => *d,
                    _ => unreachable!(),
                };
                v1 > v2
            } else {
                let v1 = match self {
                    Value::Bool(b) => *b,
                    _ => unreachable!(),
                };
                let v2 = match other {
                    Value::Bool(b) => *b,
                    _ => unreachable!(),
                };
                v1 > v2
            }
        }
    }

    pub fn less_than(&self, other: &Value) -> bool {
        let ltype = self.valtype();

        let rtype = other.valtype();

        if ltype != rtype {
            false
        } else {
            if ltype == ValueType::INT {
                let v1 = match self {
                    Value::Int(i) => *i,
                    _ => unreachable!(),
                };
                let v2 = match other {
                    Value::Int(i) => *i,
                    _ => unreachable!(),
                };
                v1 < v2
            } else if ltype == ValueType::DOUBLE {
                let v1 = match self {
                    Value::Double(d) => *d,
                    _ => unreachable!(),
                };
                let v2 = match other {
                    Value::Double(d) => *d,
                    _ => unreachable!(),
                };
                v1 < v2
            } else {
                let v1 = match self {
                    Value::Bool(b) => *b,
                    _ => unreachable!(),
                };
                let v2 = match other {
                    Value::Bool(b) => *b,
                    _ => unreachable!(),
                };
                v1 < v2
            }
        }
    }
}

impl Clone for Value {
    fn clone(&self) -> Value {
        match self {
            Value::Int(i) => Value::Int(*i),
            Value::Double(f) => Value::Double(*f),
            Value::Bool(b) => Value::Bool(*b),
            Value::Id(s) => Value::Id(s.to_owned()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum UnOp {
    NEGATE,
    FLIP,
    NOT,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BinOp {
    ADD,
    SUB,
    MULT,
    DIV,
    MOD,
    AND,
    OR,
    EQ,
    NE,
    GT,
    LT,
    GTE,
    LTE,
}

#[derive(Debug, Clone)]
pub enum ExpNode {
    BinOp(Box<ExpNode>, BinOp, Box<ExpNode>),
    UnOp(UnOp, Box<ExpNode>),
    Constant(Value),
}

pub struct ExpressionParser {}

impl ExpressionParser {
    pub fn parse_expression(
        token_iter: &mut Peekable<Iter<Token>>,
    ) -> Result<Option<ExpNode>, Box<dyn Error>> {
        if token_iter.peek().is_some() {
            Ok(Some(ExpressionParser::parse_logical_or(token_iter)?))
        } else {
            Ok(None)
        }
    }

    pub fn parse_logical_or(
        token_iter: &mut Peekable<Iter<Token>>,
    ) -> Result<ExpNode, Box<dyn Error>> {
        let mut lhs = ExpressionParser::parse_logical_and(token_iter)?;

        while token_iter.peek().is_some() && **token_iter.peek().unwrap() == Token::OR {
            token_iter.next();

            let rhs = ExpressionParser::parse_logical_and(token_iter)?;

            lhs = ExpNode::BinOp(lhs.into(), BinOp::OR, rhs.into());
        }

        Ok(lhs)
    }

    pub fn parse_logical_and(
        token_iter: &mut Peekable<Iter<Token>>,
    ) -> Result<ExpNode, Box<dyn Error>> {
        let mut lhs = ExpressionParser::parse_equality_exp(token_iter)?;

        while token_iter.peek().is_some() && **token_iter.peek().unwrap() == Token::AND {
            token_iter.next();

            let rhs = ExpressionParser::parse_equality_exp(token_iter)?;

            lhs = ExpNode::BinOp(lhs.into(), BinOp::AND, rhs.into());
        }

        Ok(lhs)
    }

    pub fn parse_equality_exp(
        token_iter: &mut Peekable<Iter<Token>>,
    ) -> Result<ExpNode, Box<dyn Error>> {
        let mut lhs = ExpressionParser::parse_relational_exp(token_iter)?;

        while token_iter.peek().is_some()
            && (match token_iter.peek().unwrap() {
                Token::EQ => true,
                Token::NE => true,
                _ => false,
            })
        {
            let op = match token_iter.next().unwrap() {
                Token::EQ => BinOp::EQ,
                Token::NE => BinOp::NE,
                _ => unreachable!(),
            };

            let rhs = ExpressionParser::parse_relational_exp(token_iter)?;

            lhs = ExpNode::BinOp(lhs.into(), op, rhs.into());
        }

        Ok(lhs)
    }

    pub fn parse_relational_exp(
        token_iter: &mut Peekable<Iter<Token>>,
    ) -> Result<ExpNode, Box<dyn Error>> {
        let mut lhs = ExpressionParser::parse_additive_exp(token_iter)?;

        while token_iter.peek().is_some()
            && (match token_iter.peek().unwrap() {
                Token::GT => true,
                Token::LT => true,
                Token::GTE => true,
                Token::LTE => true,
                _ => false,
            })
        {
            let op = match token_iter.next().unwrap() {
                Token::GT => BinOp::GT,
                Token::LT => BinOp::LT,
                Token::GTE => BinOp::GTE,
                Token::LTE => BinOp::LTE,
                _ => unreachable!(),
            };

            let rhs = ExpressionParser::parse_additive_exp(token_iter)?;

            lhs = ExpNode::BinOp(lhs.into(), op, rhs.into());
        }

        Ok(lhs)
    }

    pub fn parse_additive_exp(
        token_iter: &mut Peekable<Iter<Token>>,
    ) -> Result<ExpNode, Box<dyn Error>> {
        let mut lhs = ExpressionParser::parse_term(token_iter)?;

        while token_iter.peek().is_some()
            && (match token_iter.peek().unwrap() {
                Token::ADD => true,
                Token::MINUS => true,
                _ => false,
            })
        {
            let op = match token_iter.next().unwrap() {
                Token::ADD => BinOp::ADD,
                Token::MINUS => BinOp::SUB,
                _ => unreachable!(),
            };

            let rhs = ExpressionParser::parse_term(token_iter)?;

            lhs = ExpNode::BinOp(lhs.into(), op, rhs.into());
        }

        Ok(lhs)
    }

    pub fn parse_term(token_iter: &mut Peekable<Iter<Token>>) -> Result<ExpNode, Box<dyn Error>> {
        let mut lhs = ExpressionParser::parse_factor(token_iter)?;

        while token_iter.peek().is_some()
            && (match token_iter.peek().unwrap() {
                Token::MULT => true,
                Token::DIV => true,
                _ => false,
            })
        {
            let op = match token_iter.next().unwrap() {
                Token::MULT => BinOp::MULT,
                Token::DIV => BinOp::DIV,
                _ => unreachable!(),
            };

            let rhs = ExpressionParser::parse_factor(token_iter)?;

            lhs = ExpNode::BinOp(lhs.into(), op, rhs.into());
        }

        Ok(lhs)
    }

    pub fn parse_factor(token_iter: &mut Peekable<Iter<Token>>) -> Result<ExpNode, Box<dyn Error>> {
        if token_iter.peek().is_none() {
            return Err("Tried to parse empty expression".into());
        }

        match token_iter.peek().unwrap() {
            Token::OPENPAREN => {
                // Consume the (
                token_iter.next();

                let exp_option = ExpressionParser::parse_expression(token_iter)?;

                if exp_option.is_none() {
                    return Err("Expected expression, found (".into());
                }

                let exp = exp_option.unwrap();

                if token_iter.peek().is_some() && **token_iter.peek().unwrap() == Token::CLOSEPAREN
                {
                    // Consume the )
                    token_iter.next();

                    return Ok(exp);
                } else {
                    return Err("Error parsing expression, found (, expected closing )".into());
                }
            }
            Token::NEGATE | Token::COMP | Token::MINUS => {
                let op = match token_iter.next().unwrap() {
                    Token::NEGATE => UnOp::NOT,
                    Token::COMP => UnOp::FLIP,
                    Token::MINUS => UnOp::NEGATE,
                    _ => unreachable!(),
                };

                let factor = ExpressionParser::parse_factor(token_iter)?;

                return Ok(ExpNode::UnOp(op, factor.into()));
            }
            Token::INT(v) => {
                // Consume it
                token_iter.next();
                return Ok(ExpNode::Constant(Value::Int(*v)).into());
            }
            Token::DOUBLE(v) => {
                // Consume it
                token_iter.next();
                return Ok(ExpNode::Constant(Value::Double(*v)).into());
            }
            Token::IDENTIFIER(v) => {
                // Consume it
                token_iter.next();

                if v == "true" {
                    return Ok(ExpNode::Constant(Value::Bool(true)).into());
                } else if v == "false" {
                    return Ok(ExpNode::Constant(Value::Bool(false)).into());
                } else {
                    return Ok(ExpNode::Constant(Value::Id(v.to_owned())).into());
                }
            }
            t => {
                return Err(format!("Invalid token {:?}", t).into());
            }
        }
    }
}