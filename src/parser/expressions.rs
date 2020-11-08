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
            _ => Err("Cannot implicitly convert identifier to boolean".into()),
        }
    }

    pub fn to_double(&self) -> Result<f64, Box<dyn Error>> {
        match self {
            Value::Int(i) => Ok(*i as f64),
            Value::Double(d) => Ok(*d),
            Value::Bool(b) => Ok(if *b { 1.0 } else { 0.0 }),
            _ => Err("Cannot implicitly convert identifier to double".into()),
        }
    }

    pub fn to_int(&self) -> Result<i32, Box<dyn Error>> {
        match self {
            Value::Int(i) => Ok(*i),
            Value::Double(d) => Ok(*d as i32),
            Value::Bool(b) => Ok(if *b { 1 } else { 0 }),
            _ => Err("Cannot implicitly convert identifier to integer".into()),
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
        }
        else {
            if ltype == ValueType::INT {
                let v1 = match self  { Value::Int(i) => *i, _ => unreachable!()};
                let v2 = match other { Value::Int(i) => *i, _ => unreachable!()};
                v1 == v2
            } else if ltype == ValueType::DOUBLE {
                let v1 = match self  { Value::Double(d) => *d, _ => unreachable!()};
                let v2 = match other { Value::Double(d) => *d, _ => unreachable!()};
                v1 == v2
            }
            else {
                let v1 = match self  { Value::Bool(b) => *b, _ => unreachable!()};
                let v2 = match other { Value::Bool(b) => *b, _ => unreachable!()};
                v1 == v2
            }
        }
    }

    pub fn greater_than(&self, other: &Value) -> bool {
        let ltype = self.valtype();

        let rtype = other.valtype();

        if ltype != rtype {
            false
        }
        else {
            if ltype == ValueType::INT {
                let v1 = match self  { Value::Int(i) => *i, _ => unreachable!()};
                let v2 = match other { Value::Int(i) => *i, _ => unreachable!()};
                v1 > v2
            } else if ltype == ValueType::DOUBLE {
                let v1 = match self  { Value::Double(d) => *d, _ => unreachable!()};
                let v2 = match other { Value::Double(d) => *d, _ => unreachable!()};
                v1 > v2
            }
            else {
                let v1 = match self  { Value::Bool(b) => *b, _ => unreachable!()};
                let v2 = match other { Value::Bool(b) => *b, _ => unreachable!()};
                v1 > v2
            }
        }
    }

    pub fn less_than(&self, other: &Value) -> bool {
        let ltype = self.valtype();

        let rtype = other.valtype();

        if ltype != rtype {
            false
        }
        else {
            if ltype == ValueType::INT {
                let v1 = match self  { Value::Int(i) => *i, _ => unreachable!()};
                let v2 = match other { Value::Int(i) => *i, _ => unreachable!()};
                v1 < v2
            } else if ltype == ValueType::DOUBLE {
                let v1 = match self  { Value::Double(d) => *d, _ => unreachable!()};
                let v2 = match other { Value::Double(d) => *d, _ => unreachable!()};
                v1 < v2
            }
            else {
                let v1 = match self  { Value::Bool(b) => *b, _ => unreachable!()};
                let v2 = match other { Value::Bool(b) => *b, _ => unreachable!()};
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

// pub struct ExpressionParser<'a> {
//     or_ops: Vec<&'a str>,
//     and_ops: Vec<&'a str>,
//     equ_ops: Vec<&'a str>,
//     rel_ops: Vec<&'a str>,
//     add_ops: Vec<&'a str>,
//     term_ops: Vec<&'a str>,
//     factor_ops: Vec<&'a str>
// }

// impl<'b> ExpressionParser<'b> {

//     pub fn new() -> ExpressionParser<'b> {

//         ExpressionParser {
//             or_ops: vec![ OR ],
//             and_ops: vec![ AND ],
//             equ_ops: vec![ EQ, NEQ ],
//             rel_ops: vec![ LT, GT, LTE, GTE ],
//             add_ops: vec![ PLUS, MINUS ],
//             term_ops: vec![ MULT, DIV ],
//             factor_ops: vec![ NOT, FLIP, NEG ]
//         }

//     }

//     fn next_two<'a> (char_iter: &mut Peekable<Chars>, possible: &'a Vec<&str>) -> Result<&'a str, Box<dyn Error>> {

//         let mut matched = false;

//         for operation in possible.iter() {

//             if *char_iter.peek().unwrap() == operation.chars().next().unwrap() {
//                 matched = true;
//             }

//         }

//         if !matched {
//             return Ok("");
//         }

//         let char1 = char_iter.next().unwrap();

//         if char_iter.peek().is_none() {

//             return Err( format!("Found {}, expected {}", char1, possible_to_str(possible)).into() );

//         }

//         let char2 = char_iter.next().unwrap();

//         let combined = format!("{}{}", char1, char2);

//         for operation in possible {

//             if combined == *operation {
//                 return Ok(operation);
//             }

//         }

//         Err( format!("Found {}, expected {}", combined, possible_to_str(possible)).into() )
//     }

//     pub fn parse_expression<'a> (&self, raw: &'a str) -> Result<ExpNode<'a>, Box<dyn Error>> {

//         let char_iter = raw.chars().peekable();

//         while *char_iter.peek().ok_or("Expected expression")? == ' ' {
//             char_iter.next();
//         }

//         Ok(self.parse_logical_or(&mut char_iter)?)

//     }

//     fn parse_logical_or<'a> (&self, char_iter: &mut Peekable<Chars>) -> Result<ExpNode<'a>, Box<dyn Error>> {

//         let and_exp = self.parse_logical_and(char_iter)?;

//         while ' ' == match char_iter.peek() {
//             Some(v) => {
//                 *v
//             },
//             None => {
//                 return Ok(and_exp);
//             }
//         } {
//             char_iter.next();
//         }

//         let next = ExpressionParser::next_two(char_iter, &self.or_ops)?;

//         if next.is_empty() {
//             Ok( and_exp )
//         }
//         else {
//             let second_and_exp = self.parse_logical_and(char_iter)?;

//             Ok( ExpNode::BinOp(and_exp.into(), Operator::OR, second_and_exp.into()) )
//         }
//     }

//     fn parse_logical_and<'a> (&self, char_iter: &mut Peekable<Chars>) -> Result<ExpNode<'a>, Box<dyn Error>> {

//         let eq_exp = self.parse_equality_exp(char_iter)?;

//         while ' ' == match char_iter.peek() {
//             Some(v) => {
//                 *v
//             },
//             None => {
//                 return Ok(eq_exp);
//             }
//         } {
//             char_iter.next();
//         }

//         let next = ExpressionParser::next_two(char_iter, &self.and_ops)?;

//         if next.is_empty() {
//             Ok( eq_exp )
//         }
//         else {
//             let second_eq_exp = self.parse_equality_exp(char_iter)?;

//             Ok( ExpNode::BinOp(eq_exp.into(), Operator::AND, second_eq_exp.into()) )
//         }

//     }

//     fn parse_equality_exp<'a> (&self, char_iter: &mut Peekable<Chars>) -> Result<ExpNode<'a>, Box<dyn Error>> {

//         let rel_exp = self.parse_relational_exp(char_iter)?;

//         while ' ' == match char_iter.peek() {
//             Some(v) => {
//                 *v
//             },
//             None => {
//                 return Ok(rel_exp);
//             }
//         } {
//             char_iter.next();
//         }

//         let next = ExpressionParser::next_two(char_iter, &self.equ_ops)?;

//         if next.is_empty() {
//             Ok( rel_exp )
//         }
//         else {
//             let second_rel_exp = self.parse_relational_exp(char_iter)?;

//             let op = match next {
//                 "==" => Operator::EQ,
//                 "!=" => Operator::NEQ,
//                 _ => panic!("Unexpected operator {}", next)
//             };

//             Ok( ExpNode::BinOp(rel_exp.into(), op, second_rel_exp.into()) )
//         }

//     }

//     fn parse_relational_exp<'a> (&self, char_iter: &mut Peekable<Chars>) -> Result<ExpNode<'a>, Box<dyn Error>> {

//         let add_exp = self.parse_relational_exp(char_iter)?;

//         while ' ' == match char_iter.peek() {
//             Some(v) => {
//                 *v
//             },
//             None => {
//                 return Ok(add_exp);
//             }
//         } {
//             char_iter.next();
//         }

//         let next = ExpressionParser::next_two(char_iter, &self.add_ops)?;

//         if next.is_empty() {
//             Ok( add_exp )
//         }
//         else {
//             let second_add_exp = self.parse_relational_exp(char_iter)?;

//             let op = match next {
//                 "==" => Operator::EQ,
//                 "!=" => Operator::NEQ,
//                 _ => panic!("Unexpected operator {}", next)
//             };

//             Ok( ExpNode::BinOp(rel_exp.into(), op, second_rel_exp.into()) )
//         }

//     }

//     fn parse_additive_exp<'a> (&self, char_iter: &mut Peekable<Chars>) -> Result<ExpNode<'a>, Box<dyn Error>> {

//     }

//     fn parse_term<'a> (&self, char_iter: &mut Peekable<Chars>) -> Result<ExpNode<'a>, Box<dyn Error>> {

//     }

//     fn parse_factor<'a> (&self, char_iter: &mut Peekable<Chars>) -> Result<ExpNode<'a>, Box<dyn Error>> {

//     }

//     fn parse_constant<'a> (raw: &'a str) -> Result<ExpNode<'a>, Box<dyn Error>> {

//         let mut stop_index = 0;

//         for char in raw.chars() {
//             if char.is_ascii_digit() {
//                 stop_index += 1;
//             }
//             else {
//                 break;
//             }
//         }

//         Ok(ExpNode::Constant( Value::Int(i32::from_str_radix( &raw[..stop_index], 10 )?) ))

//     }

// }

// fn possible_to_str(possible: &Vec<&str>) -> String {

//     let mut str = String::new();

//     for (i, v) in possible.iter().enumerate() {

//         if i == possible.len() - 1 {

//             str.push_str(v);

//         }
//         else if i < possible.len() - 2 {

//             str.push_str(&format!("{} or ", v));

//         }
//         else {

//             str.push_str(&format!("{}, ", v));

//         }

//     }

//     str
// }
