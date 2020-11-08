mod expressions;
pub use expressions::{BinOp, ExpNode, ExpressionParser, UnOp, Value, ValueType};

mod instructions;
pub use instructions::{Definition, DefinitionTable, ExpressionEvaluator};
