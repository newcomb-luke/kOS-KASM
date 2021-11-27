use super::expressions::{BinOp, ExpNode, UnOp, Value};

pub type EvalResult = Result<Value, EvalError>;
pub type OpResult<T> = Result<T, EvalError>;

pub enum EvalError {
    /// A scenario such as trying to evaluate -false
    NegateBool,
    /// A scenario such as trying to evaluate ~2.0
    FlipDouble,
    /// A scenario such as trying to evaluate 2 / 0
    ZeroDivide,
}

pub struct ExpressionEvaluator {}

impl ExpressionEvaluator {
    /// Evalutes a constant expression. Returns a Ok(Value) that represents the final result.
    /// Returns Err() when expression evaluation fails
    pub fn evaluate(expression: &ExpNode) -> EvalResult {
        match expression {
            ExpNode::Constant(constant) => Ok(*constant),
            ExpNode::UnOp(op, node) => Self::evaluate_unop(*op, &node),
            ExpNode::BinOp(lhs, op, rhs) => Self::evaluate_binop(&lhs, *op, &rhs),
        }
    }

    fn evaluate_unop(op: UnOp, node: &ExpNode) -> EvalResult {
        let value = Self::evaluate(node)?;

        Ok(match op {
            UnOp::Not => value.not()?,
            UnOp::Flip => value.flip()?,
            UnOp::Negate => value.negate()?,
        })
    }

    fn evaluate_binop(lhs: &ExpNode, op: BinOp, rhs: &ExpNode) -> EvalResult {
        let lhs_value = Self::evaluate(lhs)?;
        let rhs_value = Self::evaluate(rhs)?;

        match op {
            BinOp::Add => lhs_value.add(rhs_value),
            BinOp::Sub => lhs_value.sub(rhs_value),
            BinOp::Mult => lhs_value.mult(rhs_value),
            BinOp::Div => lhs_value.div(rhs_value),
            BinOp::Mod => lhs_value.modulus(rhs_value),
            BinOp::Eq => lhs_value.equal(rhs_value),
            BinOp::Ne => lhs_value.equal(rhs_value)?.not(),
            BinOp::Gt => lhs_value.greater(rhs_value),
            BinOp::Lte => lhs_value.greater(rhs_value)?.not(),
            BinOp::Lt => lhs_value.less(rhs_value),
            BinOp::Gte => lhs_value.less(rhs_value)?.not(),
            BinOp::Or => lhs_value.or(rhs_value),
            BinOp::And => lhs_value.and(rhs_value),
        }
    }
}

trait Not: Sized {
    fn not(self) -> OpResult<Self>;
}

trait Negate: Sized {
    fn negate(self) -> OpResult<Self>;
}

trait Flip: Sized {
    fn flip(self) -> OpResult<Self>;
}

trait Add: Sized {
    fn add(self, other: Self) -> OpResult<Self>;
}

trait Sub: Sized {
    fn sub(self, other: Self) -> OpResult<Self>;
}

trait Mult: Sized {
    fn mult(self, other: Self) -> OpResult<Self>;
}

trait Div: Sized {
    fn div(self, other: Self) -> OpResult<Self>;
}

trait Mod: Sized {
    fn modulus(self, other: Self) -> OpResult<Self>;
}

trait Equal: Sized {
    fn equal(self, other: Self) -> OpResult<Self>;
}

trait Greater: Sized {
    fn greater(self, other: Self) -> OpResult<Self>;
}

trait Less: Sized {
    fn less(self, other: Self) -> OpResult<Self>;
}

pub trait ToBool: Sized {
    fn to_bool(self) -> bool;
}

trait And: Sized {
    fn and(self, other: Self) -> OpResult<Self>;
}

trait Or: Sized {
    fn or(self, other: Self) -> OpResult<Self>;
}

impl Not for Value {
    fn not(self) -> OpResult<Self> {
        Ok(Value::Bool(match self {
            Value::Int(i) => i != 0,
            Value::Bool(b) => !b,
            Value::Double(d) => d != 0.0,
        }))
    }
}

impl Negate for Value {
    fn negate(self) -> OpResult<Self> {
        match self {
            Value::Int(i) => Ok(Value::Int(-i)),
            Value::Bool(_) => Err(EvalError::NegateBool),
            Value::Double(d) => Ok(Value::Double(-d)),
        }
    }
}

impl Flip for Value {
    fn flip(self) -> OpResult<Self> {
        match self {
            Value::Int(i) => Ok(Value::Int(!i)),
            Value::Bool(b) => Ok(Value::Bool(!b)),
            Value::Double(_) => Err(EvalError::FlipDouble),
        }
    }
}

impl Add for Value {
    fn add(self, other: Self) -> OpResult<Self> {
        Ok(match self {
            Value::Int(i) => match other {
                Value::Int(i2) => Value::Int(i + i2),
                Value::Bool(b) => Value::Int(i + if b { 1 } else { 0 }),
                Value::Double(d) => Value::Double(i as f64 + d),
            },
            Value::Bool(b) => match other {
                Value::Int(i) => Value::Int(i + if b { 1 } else { 0 }),
                Value::Bool(b1) => Value::Int(if b { 1 } else { 0 } + if b1 { 1 } else { 0 }),
                Value::Double(d) => Value::Double(d + if b { 1.0 } else { 0.0 }),
            },
            Value::Double(d) => match other {
                Value::Int(i) => Value::Double(i as f64 + d),
                Value::Bool(b) => Value::Double(d + if b { 1.0 } else { 0.0 }),
                Value::Double(d1) => Value::Double(d + d1),
            },
        })
    }
}

impl Sub for Value {
    fn sub(self, other: Self) -> OpResult<Self> {
        Ok(match self {
            Value::Int(i) => match other {
                Value::Int(i2) => Value::Int(i - i2),
                Value::Bool(b) => Value::Int(i - if b { 1 } else { 0 }),
                Value::Double(d) => Value::Double(i as f64 - d),
            },
            Value::Bool(b) => match other {
                Value::Int(i) => Value::Int(i - if b { 1 } else { 0 }),
                Value::Bool(b1) => Value::Int(if b { 1 } else { 0 } - if b1 { 1 } else { 0 }),
                Value::Double(d) => Value::Double(d - if b { 1.0 } else { 0.0 }),
            },
            Value::Double(d) => match other {
                Value::Int(i) => Value::Double(i as f64 - d),
                Value::Bool(b) => Value::Double(d - if b { 1.0 } else { 0.0 }),
                Value::Double(d1) => Value::Double(d - d1),
            },
        })
    }
}

impl Mult for Value {
    fn mult(self, other: Self) -> OpResult<Self> {
        Ok(match self {
            Value::Int(i) => match other {
                Value::Int(i2) => Value::Int(i * i2),
                Value::Bool(b) => Value::Int(i * if b { 1 } else { 0 }),
                Value::Double(d) => Value::Double(i as f64 * d),
            },
            Value::Bool(b) => match other {
                Value::Int(i) => Value::Int(i * if b { 1 } else { 0 }),
                Value::Bool(b1) => Value::Int(if b { 1 } else { 0 } * if b1 { 1 } else { 0 }),
                Value::Double(d) => Value::Double(d * if b { 1.0 } else { 0.0 }),
            },
            Value::Double(d) => match other {
                Value::Int(i) => Value::Double(i as f64 * d),
                Value::Bool(b) => Value::Double(d * if b { 1.0 } else { 0.0 }),
                Value::Double(d1) => Value::Double(d * d1),
            },
        })
    }
}

impl Div for Value {
    fn div(self, other: Self) -> OpResult<Self> {
        match self {
            Value::Int(i) => {
                let other_int = match other {
                    Value::Int(i2) => i2,
                    Value::Bool(b) => {
                        if b {
                            1
                        } else {
                            0
                        }
                    }
                    Value::Double(d) => {
                        return if d != 0.0 {
                            Ok(Value::Double(i as f64 / d))
                        } else {
                            Err(EvalError::ZeroDivide)
                        };
                    }
                };

                if other_int != 0 {
                    Ok(Value::Int(i / other_int))
                } else {
                    Err(EvalError::ZeroDivide)
                }
            }
            Value::Bool(b) => {
                let other_int = match other {
                    Value::Int(i) => i,
                    Value::Bool(b) => {
                        if b {
                            1
                        } else {
                            0
                        }
                    }
                    Value::Double(d) => {
                        return if d != 0.0 {
                            Ok(Value::Double(if b { 1.0 } else { 0.0 } / d))
                        } else {
                            Err(EvalError::ZeroDivide)
                        };
                    }
                };

                if other_int != 0 {
                    Ok(Value::Int(if b { 1 } else { 0 } / other_int))
                } else {
                    Err(EvalError::ZeroDivide)
                }
            }
            Value::Double(d) => {
                let other_double = match other {
                    Value::Int(i) => i as f64,
                    Value::Bool(b) => {
                        if b {
                            1.0
                        } else {
                            0.0
                        }
                    }
                    Value::Double(d1) => d1,
                };

                if other_double != 0.0 {
                    Ok(Value::Double(d / other_double))
                } else {
                    Err(EvalError::ZeroDivide)
                }
            }
        }
    }
}

impl Mod for Value {
    fn modulus(self, other: Self) -> OpResult<Self> {
        Ok(match self {
            Value::Int(i) => {
                let other_int = match other {
                    Value::Int(i2) => i2,
                    Value::Bool(b) => {
                        if b {
                            1
                        } else {
                            0
                        }
                    }
                    Value::Double(d) => {
                        return Ok(Value::Double(i as f64 % d));
                    }
                };

                Value::Int(i % other_int)
            }
            Value::Bool(b) => {
                let other_int = match other {
                    Value::Int(i) => i,
                    Value::Bool(b) => {
                        if b {
                            1
                        } else {
                            0
                        }
                    }
                    Value::Double(d) => {
                        return Ok(Value::Double(if b { 1.0 } else { 0.0 } % d));
                    }
                };

                Value::Int(if b { 1 } else { 0 } % other_int)
            }
            Value::Double(d) => {
                let other_double = match other {
                    Value::Int(i) => i as f64,
                    Value::Bool(b) => {
                        if b {
                            1.0
                        } else {
                            0.0
                        }
                    }
                    Value::Double(d1) => d1,
                };

                Value::Double(d % other_double)
            }
        })
    }
}

impl Equal for Value {
    fn equal(self, other: Self) -> OpResult<Self> {
        Ok(Value::Bool(match self {
            Value::Int(i) => match other {
                Value::Int(i2) => i == i2,
                Value::Bool(b) => i == if b { 1 } else { 0 },
                Value::Double(d) => i as f64 == d,
            },
            Value::Bool(b) => match other {
                Value::Int(i) => i == if b { 1 } else { 0 },
                Value::Bool(b1) => b == b1,
                Value::Double(d) => d == if b { 1.0 } else { 0.0 },
            },
            Value::Double(d) => match other {
                Value::Int(i) => i as f64 == d,
                Value::Bool(b) => d == if b { 1.0 } else { 0.0 },
                Value::Double(d1) => d == d1,
            },
        }))
    }
}

impl Greater for Value {
    fn greater(self, other: Self) -> OpResult<Self> {
        Ok(Value::Bool(match self {
            Value::Int(i) => match other {
                Value::Int(i2) => i > i2,
                Value::Bool(b) => i > if b { 1 } else { 0 },
                Value::Double(d) => i as f64 > d,
            },
            Value::Bool(b) => match other {
                Value::Int(i) => i > if b { 1 } else { 0 },
                Value::Bool(b1) => b > b1,
                Value::Double(d) => d > if b { 1.0 } else { 0.0 },
            },
            Value::Double(d) => match other {
                Value::Int(i) => i as f64 > d,
                Value::Bool(b) => d > if b { 1.0 } else { 0.0 },
                Value::Double(d1) => d > d1,
            },
        }))
    }
}

impl Less for Value {
    fn less(self, other: Self) -> OpResult<Self> {
        Ok(Value::Bool(match self {
            Value::Int(i) => match other {
                Value::Int(i2) => i < i2,
                Value::Bool(b) => i < if b { 1 } else { 0 },
                Value::Double(d) => (i as f64) < d,
            },
            Value::Bool(b) => match other {
                Value::Int(i) => i < if b { 1 } else { 0 },
                Value::Bool(b1) => b < b1,
                Value::Double(d) => d < if b { 1.0 } else { 0.0 },
            },
            Value::Double(d) => match other {
                Value::Int(i) => (i as f64) < d,
                Value::Bool(b) => d < if b { 1.0 } else { 0.0 },
                Value::Double(d1) => d < d1,
            },
        }))
    }
}

impl ToBool for Value {
    fn to_bool(self) -> bool {
        match self {
            Value::Int(i) => i != 0,
            Value::Bool(b) => b,
            Value::Double(d) => d != 0.0,
        }
    }
}

impl And for Value {
    fn and(self, other: Self) -> OpResult<Self> {
        let b1 = self.to_bool();
        let b2 = other.to_bool();

        Ok(Value::Bool(b1 && b2))
    }
}

impl Or for Value {
    fn or(self, other: Self) -> OpResult<Self> {
        let b1 = self.to_bool();
        let b2 = other.to_bool();

        Ok(Value::Bool(b1 || b2))
    }
}
