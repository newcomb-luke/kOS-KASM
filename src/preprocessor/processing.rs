use std::{collections::HashMap, error::Error, slice::Iter, fs, iter::Peekable, path::Path};

use crate::{BinOp, ExpNode, UnOp, Value, ValueType, Token, Lexer, Instruction};

pub enum Definition {
    EMPTY,
    MACRO(Vec<Token>),
    MACROARGS(Vec<String>, Vec<Token>)
}

pub struct DefinitionTable {
    definitions: HashMap<String, Definition>,
}

pub enum DirectiveType {
    DEFINE,
    UNDEF,
    MACRO,
    ENDMACRO,
    UNMACRO,
    IF,
    IFN,
    ELIF,
    ELIFN,
    ELSE,
    ENDIF,
    IFDEF,
    IFNDEF,
    REP,
    ENDREP,
    INCLUDE,
    LINE
}

impl DirectiveType {
    pub fn from_str(s: &str) -> Result<DirectiveType, Box<dyn Error>> {
        Ok(match s {
            "define" => DirectiveType::DEFINE,
            "undef" => DirectiveType::UNDEF,
            "macro" => DirectiveType::MACRO,
            "unmacro" => DirectiveType::UNMACRO,
            "if" => DirectiveType::IF,
            "ifn" => DirectiveType::IFN,
            "elif" => DirectiveType::ELIF,
            "elifn" => DirectiveType::ELIFN,
            "else" => DirectiveType::ELSE,
            "endif" => DirectiveType::ENDIF,
            "ifdef" => DirectiveType::IFDEF,
            "ifndef" => DirectiveType::IFNDEF,
            "rep" => DirectiveType::REP,
            "endrep" => DirectiveType::ENDREP,
            "include" => DirectiveType::INCLUDE,
            "line" => DirectiveType::LINE,
            _ => {
                return Err(format!("Invalid directive found: {}", s).into());
            }
        })
    }
}

// pub struct Preprocessor {
//     definition_table: DefinitionTable,
//     include_path: String
// }

// impl Preprocessor {

//     pub fn new(include_path: String) -> Preprocessor {
//         Preprocessor {
//             definition_table: DefinitionTable::new(),
//             include_path,
//         }
//     }

//     pub fn process(&mut self, tokens: Vec<Token>) -> Result<Vec<Token>, Box<dyn Error>> {

//         // We will allocate a vector just as big as the tokens one, just in case it optimizes something
//         let mut new_tokens: Vec<Token> = Vec::with_capacity(tokens.len());

//         let mut token_iter = tokens.iter().peekable();

//         while token_iter.peek().is_some() {

//             match token_iter.peek().unwrap() {
//                 Token::DIRECTIVE(d) => {
//                     token_iter.next(); // Consume the directive token so it doesn't hold things up
//                     self.process_directive(d, &mut token_iter)?;
//                 },
//                 t => {
//                     new_tokens.push((*t).clone());
//                 }
//             }
//         }

//         Ok(new_tokens)
//     }

//     pub fn process_directive(&mut self, directive: &String, token_iter: &mut Peekable<Iter<Token>>) -> Result<(), Box<dyn Error>> {
//         let dtype = DirectiveType::from_str(directive)?;

//         match dtype {
//             DirectiveType::DEFINE => {
//                 let (id, definition) = self.parse_define(token_iter)?;
//                 self.definition_table.def(&id, definition);
//             },
//             _ => {
//                 return Err("Currently unsupported directive type.".into());
//             }
//         }

//         Ok(())
//     }

//     pub fn parse_define(&self, token_iter: &mut Peekable<Iter<Token>>) -> Result<(String, Definition), Box<dyn Error>> {
//         let id = self.expect_identifier(token_iter)?;
//         let mut contents = Vec::new();
//         let mut args = Vec::new();
//         let definition;

//         // Check to see if the id is valid, basically if it is an instruction, it is not valid
//         if Instruction::is_instruction(&id) {
//             return Err("Cannot define macro with same identifier as an instruction".into());
//         }

//         // If the define directive has a value
//         if token_iter.peek().is_some() {
//             match token_iter.peek().unwrap() {
//                 // If it has arguments
//                 Token::OPENPAREN => {
//                     // Get rid of the open parenthesis
//                     token_iter.next();

//                     // Consume all of the arguments
//                     loop {
//                         if token_iter.peek().is_none() {
//                             return Err(format!("Expected an argument in definition {}", id).into());
//                         }
//                         else {
//                             match token_iter.next().unwrap() {
//                                 Token::CLOSEPAREN => {
//                                     // This is for if the arguments were empty
//                                     break;
//                                 },
//                                 Token::IDENTIFIER(s) => {
//                                     // Push that identifier as the argument string
//                                     args.push(s.clone());
//                                     // After each argument there should be a comma if it isn't the last
//                                     match token_iter.next() {
//                                         Some(t) => {
//                                             match t {
//                                                 Token::COMMA => {},
//                                                 Token::CLOSEPAREN => {
//                                                     println!("lol");
//                                                     break;
//                                                 },
//                                                 _ => {
//                                                     return Err("Expected comma, found other token.".into());
//                                                 }
//                                             }
//                                         },
//                                         None => {
//                                             return Err("Incomplete define directive.".into());
//                                         }
//                                     }
//                                 },
//                                 _ => {
//                                     return Err(format!("Expected argument or close parenthesis.").into());
//                                 }
//                             }
//                         }
//                     }

                    
//                 },
//                 _ => {}
//             }

//             // If there is another token, and it isn't a newline, then loop
//             while token_iter.peek().is_some() && match token_iter.peek().unwrap() { Token::NEWLINE => false, _ => true } {
//                 // Append each token into the contents
//                 contents.push(token_iter.next().unwrap().clone());
//             }

//             // If the file doesn't end here, there is a newline
//             if token_iter.peek().is_some() {
//                 // So consume it
//                 token_iter.next();
//             }

//             // If there are any arguments
//             if args.len() > 0 {
//                 definition = Definition::MACROARGS(args, contents);
//             } else {
//                 definition = Definition::MACRO(contents);
//             }
//         }
//         else {
//             definition = Definition::EMPTY;
//         }

//         Ok((id, definition))
//     }

//     pub fn expect_identifier(&self, token_iter: &mut Peekable<Iter<Token>>) -> Result<String, Box<dyn Error>> {
//         if token_iter.peek().is_some() {
//             match token_iter.next().unwrap() {
//                 Token::IDENTIFIER(s) => {
//                     Ok(s.to_owned())
//                 },
//                 t => {
//                     Err(format!("Expected macro identifier, found {:?}.", t).into())
//                 }
//             }
//         }
//         else {
//             Err("Found empty directive.".into())
//         }
//     }

//     pub fn include_file(&self, file_path: &str) -> Result<Vec<Token>, Box<dyn Error>> {
        
//         let mut file_path = Path::new(file_path);
//         let path_buffer;
//         let contents;
//         let tokens;

//         // If the file does not exist, error out here
//         if !file_path.exists() {
//             return Err(format!("Could not include {}, file does not exist.", file_path.to_str().unwrap()).into());
//         }

//         // If the file isn't a file, there is a problem
//         if !file_path.is_file() {
//             return Err(format!("Could not include {}, directories cannot be included.", file_path.to_str().unwrap()).into());
//         }

//         // If it is an absolute path, we don't need to do anything. If it is not, make it one by adding the include path
//         if !file_path.is_absolute() {
//             let include_path = Path::new(&self.include_path);
//             path_buffer = include_path.join(file_path);
//             file_path = path_buffer.as_path();
//         }

//         // Attempt to read the file as text
//         contents = fs::read_to_string(file_path)?;

//         // Lex the contents
//         tokens = Lexer::lex(&contents)?;
        
//         Ok(tokens)

//     }

// }

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
}

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
