use crate::{BinOp, ExpNode, UnOp, Value, ValueType};

use super::errors::{ExpressionError, ExpressionResult};

pub struct ExpressionEvaluator {}

impl ExpressionEvaluator {
    pub fn evaluate(exp: &ExpNode) -> ExpressionResult<Value> {
        match exp {
            ExpNode::Constant(c) => match c {
                Value::Int(_) => Ok(c.clone()),
                Value::Double(_) => Ok(c.clone()),
                Value::Bool(_) => Ok(c.clone()),
            },
            ExpNode::UnOp(op, v) => match op {
                UnOp::FLIP => {
                    let c = ExpressionEvaluator::evaluate(v.as_ref())?;

                    match c {
                        Value::Int(i) => Ok(Value::Int(!i)),
                        _ => Err(ExpressionError::OperatorOnlyValid(String::from("~"), String::from("integer")))
                    }
                }
                UnOp::NEGATE => {
                    let c = ExpressionEvaluator::evaluate(v)?;

                    match c {
                        Value::Int(i) => Ok(Value::Int(-i)),
                        Value::Double(d) => Ok(Value::Double(-d)),
                        _ => Err(ExpressionError::OperatorNotValid(String::from("-"), String::from("bool")))
                    }
                }
                UnOp::NOT => {
                    let c = ExpressionEvaluator::evaluate(v)?;

                    match c {
                        v => Ok(Value::Bool(!v.to_bool())),
                    }
                }
            },
            ExpNode::BinOp(lhs, op, rhs) => {
                let lval = ExpressionEvaluator::evaluate(lhs)?;

                let rval = ExpressionEvaluator::evaluate(rhs)?;

                let ltype = lval.valtype();
                let rtype = rval.valtype();

                let math_return = if ltype == ValueType::INT && rtype == ValueType::INT {
                    ValueType::INT
                } else if ltype == ValueType::DOUBLE || rtype == ValueType::DOUBLE {
                    ValueType::DOUBLE
                } else {
                    ValueType::INT
                };

                match op {
                    BinOp::ADD => match math_return {
                        ValueType::INT => Ok(Value::Int(lval.to_int() + rval.to_int())),
                        ValueType::DOUBLE => {
                            Ok(Value::Double(lval.to_double() + rval.to_double()))
                        }
                        _ => unreachable!(),
                    },
                    BinOp::SUB => match math_return {
                        ValueType::INT => Ok(Value::Int(lval.to_int() - rval.to_int())),
                        ValueType::DOUBLE => {
                            Ok(Value::Double(lval.to_double() - rval.to_double()))
                        }
                        _ => unreachable!(),
                    },
                    BinOp::MULT => match math_return {
                        ValueType::INT => Ok(Value::Int(lval.to_int() * rval.to_int())),
                        ValueType::DOUBLE => {
                            Ok(Value::Double(lval.to_double() * rval.to_double()))
                        }
                        _ => unreachable!(),
                    },
                    BinOp::DIV => match math_return {
                        ValueType::INT => Ok(Value::Int(lval.to_int() / rval.to_int())),
                        ValueType::DOUBLE => {
                            Ok(Value::Double(lval.to_double() / rval.to_double()))
                        }
                        _ => unreachable!(),
                    },
                    BinOp::MOD => match math_return {
                        ValueType::INT => Ok(Value::Int(lval.to_int() % rval.to_int())),
                        ValueType::DOUBLE => {
                            Ok(Value::Double(lval.to_double() % rval.to_double()))
                        }
                        _ => unreachable!(),
                    },
                    BinOp::AND => Ok(Value::Bool(lval.to_bool() && rval.to_bool())),
                    BinOp::OR => Ok(Value::Bool(lval.to_bool() || rval.to_bool())),
                    BinOp::EQ => Ok(Value::Bool(lval.equals(&rval))),
                    BinOp::NE => Ok(Value::Bool(!lval.equals(&rval))),
                    BinOp::GT => Ok(Value::Bool(lval.greater_than(&rval))),
                    BinOp::LT => Ok(Value::Bool(lval.less_than(&rval))),
                    BinOp::GTE => Ok(Value::Bool(!lval.less_than(&rval))),
                    BinOp::LTE => Ok(Value::Bool(!lval.greater_than(&rval))),
                }
            }
        }
    }
}
