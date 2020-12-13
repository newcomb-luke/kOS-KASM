use std::{error::Error};

use crate::{BinOp, ExpNode, UnOp, Value, ValueType, DefinitionTable};

pub struct ExpressionEvaluator {}

impl ExpressionEvaluator {

    pub fn evaluate(definition_table: &mut DefinitionTable, exp: &ExpNode) -> Result<Value, Box<dyn Error>> {
        match exp {
            ExpNode::Constant(c) => match c {
                Value::Int(_) => Ok(c.clone()),
                Value::Double(_) => Ok(c.clone()),
                Value::Bool(_) => Ok(c.clone()),
            },
            ExpNode::UnOp(op, v) => match op {
                UnOp::FLIP => {
                    let c = ExpressionEvaluator::evaluate(definition_table, v.as_ref())?;

                    match c {
                        Value::Int(i) => Ok(Value::Int(!i)),
                        _ => Err("~ operator only valid on type of integer".into()),
                    }
                }
                UnOp::NEGATE => {
                    let c = ExpressionEvaluator::evaluate(definition_table, v)?;

                    match c {
                        Value::Int(i) => Ok(Value::Int(-i)),
                        Value::Double(d) => Ok(Value::Double(-d)),
                        _ => Err("- operator not valid on type bool".into()),
                    }
                }
                UnOp::NOT => {
                    let c = ExpressionEvaluator::evaluate(definition_table, v)?;

                    match c {
                        v => Ok(Value::Bool(!v.to_bool()?)),
                    }
                }
            },
            ExpNode::BinOp(lhs, op, rhs) => {
                let lval = ExpressionEvaluator::evaluate(definition_table, lhs)?;

                let rval = ExpressionEvaluator::evaluate(definition_table, rhs)?;

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
                        ValueType::INT => Ok(Value::Int(lval.to_int()? + rval.to_int()?)),
                        ValueType::DOUBLE => {
                            Ok(Value::Double(lval.to_double()? + rval.to_double()?))
                        }
                        _ => unreachable!(),
                    },
                    BinOp::SUB => match math_return {
                        ValueType::INT => Ok(Value::Int(lval.to_int()? - rval.to_int()?)),
                        ValueType::DOUBLE => {
                            Ok(Value::Double(lval.to_double()? - rval.to_double()?))
                        }
                        _ => unreachable!(),
                    },
                    BinOp::MULT => match math_return {
                        ValueType::INT => Ok(Value::Int(lval.to_int()? * rval.to_int()?)),
                        ValueType::DOUBLE => {
                            Ok(Value::Double(lval.to_double()? * rval.to_double()?))
                        }
                        _ => unreachable!(),
                    },
                    BinOp::DIV => match math_return {
                        ValueType::INT => Ok(Value::Int(lval.to_int()? / rval.to_int()?)),
                        ValueType::DOUBLE => {
                            Ok(Value::Double(lval.to_double()? / rval.to_double()?))
                        }
                        _ => unreachable!(),
                    },
                    BinOp::MOD => match math_return {
                        ValueType::INT => Ok(Value::Int(lval.to_int()? % rval.to_int()?)),
                        ValueType::DOUBLE => {
                            Ok(Value::Double(lval.to_double()? % rval.to_double()?))
                        }
                        _ => unreachable!(),
                    },
                    BinOp::AND => Ok(Value::Bool(lval.to_bool()? && rval.to_bool()?)),
                    BinOp::OR => Ok(Value::Bool(lval.to_bool()? || rval.to_bool()?)),
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