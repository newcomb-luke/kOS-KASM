mod expressions;
pub use expressions::{BinOp, ExpNode, ExpressionParser, UnOp, Value, ValueType};

mod processing;
pub use processing::{Definition, DefinitionTable, ExpressionEvaluator};

mod macros;

