use std::{collections::HashMap, error::Error};

use crate::{BinOp, ExpNode, UnOp, Value, ValueType};

pub enum Definition {
    Empty,
    Constant(ExpNode),
}

pub struct DefinitionTable {
    definitions: HashMap<String, Definition>,
}

impl DefinitionTable {
    pub fn new() -> DefinitionTable {
        DefinitionTable {
            definitions: HashMap::new(),
        }
    }

    pub fn def(&mut self, identifier: &str, new_definition: Definition) {
        // This already does what it needs to do. If it exists, the value is updated, if not, the value is created.
        self.definitions
            .insert(String::from(identifier), new_definition);
    }

    pub fn ifdef(&mut self, identifier: &str) -> bool {
        self.definitions.contains_key(identifier)
    }

    pub fn ifndef(&mut self, identifier: &str) -> bool {
        !self.ifdef(identifier)
    }

    pub fn undef(&mut self, identifier: &str) {
        self.definitions.remove(identifier);
    }

    pub fn get(&mut self, identifier: &str) -> Result<&Definition, Box<dyn Error>> {
        if self.ifdef(identifier) {
            Ok(self.definitions.get(identifier).unwrap())
        } else {
            Err(format!("Constant {} referenced before definition", identifier).into())
        }
    }

    pub fn get_as_exp(&mut self, identifier: &str) -> Result<&ExpNode, Box<dyn Error>> {
        match self.get(identifier)? {
            Definition::Empty => Err(format!("Definition {} has no value", identifier).into()),
            Definition::Constant(exp) => Ok(exp)
        }
    }
}

pub struct ExpressionEvaluator<'a> {
    definition_table: &'a mut DefinitionTable,
}

impl<'a> ExpressionEvaluator<'a> {

    pub fn new(definition_table: &'a mut DefinitionTable) -> ExpressionEvaluator {

        ExpressionEvaluator { definition_table }

    }

    pub fn evaluate(&mut self, exp: &ExpNode) -> Result<Value, Box<dyn Error>> {
        match exp {
            ExpNode::Constant(c) => {
                match c {
                    Value::Int(_) => Ok(c.clone()),
                    Value::Double(_) => Ok(c.clone()),
                    Value::Bool(_) => Ok(c.clone()),
                    Value::Id(i) => {
                        match self.definition_table.get(&i)? {
                            Definition::Empty => {
                                Err(format!("Definition {} has no value", i).into())
                            }
                            Definition::Constant(exp) => {
                                let inner_exp = exp.clone();
                                self.evaluate(&inner_exp)
                            }
                        }
                    }
                }
            },
            ExpNode::UnOp(op, v) => match op {
                UnOp::FLIP => {
                    let c = self.evaluate(v.as_ref())?;

                    match c {
                        Value::Int(i) => Ok(Value::Int(!i)),
                        Value::Id(s) => {
                            let new_exp = self.definition_table.get_as_exp(&s)?.clone();
                            self.evaluate(&ExpNode::UnOp(UnOp::FLIP, new_exp.into()))
                        },
                        _ => Err("~ operator only valid on type of integer".into()),
                    }
                }
                UnOp::NEGATE => {
                    let c = self.evaluate(v)?;

                    match c {
                        Value::Int(i) => Ok(Value::Int(-i)),
                        Value::Double(d) => Ok(Value::Double(-d)),
                        Value::Id(s) => {
                            let new_exp = self.definition_table.get_as_exp(&s)?.clone();
                            self.evaluate(&ExpNode::UnOp(UnOp::NEGATE, new_exp.into()))
                        },
                        _ => Err("- operator not valid on type bool".into()),
                    }
                }
                UnOp::NOT => {
                    let c = self.evaluate(v)?;

                    match c {
                        Value::Id(s) => {
                            let new_exp = self.definition_table.get_as_exp(&s)?.clone();
                            self.evaluate(&ExpNode::UnOp(UnOp::NOT, new_exp.into()))
                        },
                        v => Ok(Value::Bool(!v.to_bool()?)),
                    }
                }
            },
            ExpNode::BinOp(lhs, op, rhs) => {
                let lval = self.evaluate(lhs)?;

                let rval = self.evaluate(rhs)?;

                let ltype = match &lval {
                    Value::Int(_) => ValueType::INT,
                    Value::Double(_) => ValueType::DOUBLE,
                    Value::Bool(_) => ValueType::BOOL,
                    _ => unreachable!(),
                };

                let rtype = match &rval {
                    Value::Int(_) => ValueType::INT,
                    Value::Double(_) => ValueType::DOUBLE,
                    Value::Bool(_) => ValueType::BOOL,
                    _ => unreachable!(),
                };

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
                        ValueType::DOUBLE => Ok(Value::Double(lval.to_double()? + rval.to_double()?)),
                        _ => unreachable!(),
                    },
                    BinOp::SUB => match math_return {
                        ValueType::INT => Ok(Value::Int(lval.to_int()? - rval.to_int()?)),
                        ValueType::DOUBLE => Ok(Value::Double(lval.to_double()? - rval.to_double()?)),
                        _ => unreachable!(),
                    },
                    BinOp::MULT => match math_return {
                        ValueType::INT => Ok(Value::Int(lval.to_int()? * rval.to_int()?)),
                        ValueType::DOUBLE => Ok(Value::Double(lval.to_double()? * rval.to_double()?)),
                        _ => unreachable!(),
                    },
                    BinOp::DIV => match math_return {
                        ValueType::INT => Ok(Value::Int(lval.to_int()? / rval.to_int()?)),
                        ValueType::DOUBLE => Ok(Value::Double(lval.to_double()? / rval.to_double()?)),
                        _ => unreachable!(),
                    },
                    BinOp::MOD => match math_return {
                        ValueType::INT => Ok(Value::Int(lval.to_int()? % rval.to_int()?)),
                        ValueType::DOUBLE => Ok(Value::Double(lval.to_double()? % rval.to_double()?)),
                        _ => unreachable!(),
                    },
                    BinOp::AND => Ok(Value::Bool( lval.to_bool()? && rval.to_bool()? )),
                    BinOp::OR  => Ok(Value::Bool( lval.to_bool()? || rval.to_bool()? )),
                    BinOp::EQ  => Ok(Value::Bool( lval.equals(&rval)                 )),
                    BinOp::NE  => Ok(Value::Bool( !lval.equals(&rval)                )),
                    BinOp::GT  => Ok(Value::Bool( lval.greater_than(&rval)           )),
                    BinOp::LT  => Ok(Value::Bool( lval.less_than(&rval)              )),
                    BinOp::GTE => Ok(Value::Bool( !lval.less_than(&rval)             )),
                    BinOp::LTE => Ok(Value::Bool( !lval.greater_than(&rval)          )),
                }
            }
        }
    }
}
