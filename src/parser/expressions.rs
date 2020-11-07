use std::error::Error;

pub enum Value<'a> {
    Int(i32),
    Double(f64),
    Bool(bool),
    Id(&'a str),
}

pub enum UnOp {
    NEGATE,
    FLIP,
    NOT,
}

pub enum BinOp {
    ADD,
    SUB,
    MULT,
    DIV,
    MOD,
    AND,
    OR,
}

pub enum ExpNode<'a> {
    BinOp(Box<ExpNode<'a>>, BinOp, Box<ExpNode<'a>>),
    UnOp(UnOp, Box<ExpNode<'a>>),
    Constant(Value<'a>),
}

pub struct ExpressionParser {}

impl<'b> ExpressionParser {
    // pub fn parse_expression(&self) -> Result<ExpNode<'b>, Box<dyn Error>> {

    // }
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
