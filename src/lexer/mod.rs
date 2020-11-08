use std::{error::Error, iter::Peekable, str::Chars, str::FromStr};

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    OPENPAREN,
    CLOSEPAREN,
    IDENTIFIER(String),
    INT(i32),
    DOUBLE(f64),
    STRING(String),
    MINUS,
    COMP,
    NEGATE,
    ADD,
    MULT,
    DIV,
    MOD,
    AND,
    OR,
    EQ,
    NE,
    LT,
    LTE,
    GT,
    GTE,
    QUESTION,
    COLON,
    COMMA,
    NEWLINE,
    AMPERSAND,
    LABEL(String),
    INNERLABEL(String),
    DIRECTIVE(String),
    EOF,
    LINECONTINUE,
}

#[derive(PartialEq, Eq)]
enum NumberType {
    DEC,
    HEX,
    FLOAT,
    BIN,
}

pub struct Lexer {}

impl<'source> Lexer {
    pub fn lex(input: &'source str) -> Result<Vec<Token>, Box<dyn Error>> {
        let mut tokens: Vec<Token> = Vec::new();

        let mut chars = input.chars().peekable();

        while !chars.peek().is_none() {
            tokens.push(Lexer::parse_token(&mut chars)?);

            // println!("---------------------------------------------------------------");

            // for token in &tokens {
            //     println!("{:?}", token);
            // }
        }

        while tokens.len() > 0
            && (*tokens.last().unwrap() == Token::EOF || *tokens.last().unwrap() == Token::NEWLINE)
        {
            tokens.remove(tokens.len() - 1);
        }

        Lexer::remove_line_continues(&mut tokens)?;

        Ok(tokens)
    }

    fn remove_line_continues(tokens: &mut Vec<Token>) -> Result<(), Box<dyn Error>> {
        
        // I was originally going to tokens.remove() all of the line continues and newlines
        // But it turns out that copies the entire vector every time, so why not just do that?

        // We will allocate the max, and then shrink it later
        let mut new_tokens: Vec<Token> = Vec::with_capacity(tokens.len());

        let mut skip_next = false;
        for index in 0..tokens.len() {

            if !skip_next {
                if tokens[index] == Token::LINECONTINUE {
                    if index < tokens.len() - 1 && tokens[index + 1] != Token::NEWLINE {
                        return Err(format!("Error parsing, \\ should only be followed by a newline. Found: {:?}", tokens[index + 1]).into());
                    } else {
                        // Skip the newline token
                        skip_next = true;
                    }
                } else {
                    new_tokens.push(tokens[index].to_owned());
                }
            } else {
                skip_next = false;
            }

        }

        tokens.clear();
        tokens.append(&mut new_tokens);

        tokens.shrink_to_fit();

        Ok(())
    }

    fn parse_token(chars: &mut Peekable<Chars>) -> Result<Token, Box<dyn Error>> {
        let mut next_char = chars.next().unwrap();

        if next_char == ' ' {
            while !chars.peek().is_none() {
                next_char = chars.next().unwrap();

                if next_char != ' ' {
                    break;
                }
            }
        }

        // If the last token is a space
        if next_char == ' ' {
            // It is the end of the file
            return Ok(Token::EOF);
        }

        Ok(match next_char {
            '(' => Token::OPENPAREN,
            ')' => Token::CLOSEPAREN,
            '\\' => Token::LINECONTINUE,
            '_' | 'A'..='z' => Lexer::parse_identifier(next_char, chars),
            '0'..='9' => Lexer::parse_number(next_char, chars)?,
            '"' => Lexer::parse_string(chars)?,
            '.' => {
                if !chars.peek().is_none() {
                    if chars.peek().unwrap().is_ascii_digit() {
                        Lexer::parse_number(next_char, chars)?
                    } else {
                        Lexer::parse_dotted(chars)?
                    }
                } else {
                    return Err("Found lone .".into());
                }
            }
            '-' => Token::MINUS,
            '~' => Token::COMP,
            '!' => {
                if !chars.peek().is_none() && *chars.peek().unwrap() == '=' {
                    chars.next();
                    Token::NE
                } else {
                    Token::NEGATE
                }
            }
            '+' => Token::ADD,
            '*' => Token::MULT,
            '/' => Token::DIV,
            '%' => Token::MOD,
            '<' => {
                if !chars.peek().is_none() && *chars.peek().unwrap() == '=' {
                    chars.next();
                    Token::LTE
                } else {
                    Token::LT
                }
            }
            '>' => {
                if !chars.peek().is_none() && *chars.peek().unwrap() == '=' {
                    chars.next();
                    Token::GTE
                } else {
                    Token::GT
                }
            }
            '=' => {
                if !chars.peek().is_none() {
                    if *chars.peek().unwrap() != '=' {
                        return Err(format!("Found ={} expected ==", chars.peek().unwrap()).into());
                    } else {
                        chars.next();
                        Token::EQ
                    }
                } else {
                    return Err("Found = expected ==".into());
                }
            }
            '&' => {
                if !chars.peek().is_none() && *chars.peek().unwrap() == '&' {
                    Token::AND
                } else {
                    Token::AMPERSAND
                }
            }
            '|' => {
                if !chars.peek().is_none() {
                    if *chars.peek().unwrap() != '|' {
                        return Err(format!("Found |{} expected ||", chars.peek().unwrap()).into());
                    } else {
                        chars.next();
                        Token::OR
                    }
                } else {
                    return Err("Found | expected ||".into());
                }
            }
            '?' => Token::QUESTION,
            ':' => Token::COLON,
            '\n' => Token::NEWLINE,
            ',' => Token::COMMA,
            _ => {
                return Err(
                    format!("Unexpected character {} while parsing token", next_char).into(),
                );
            }
        })
    }

    fn interpret_string(s: &str) -> Result<String, EscapeError> {
        (EscapedStringInterpreter { s: s.chars() }).collect()
    }

    fn parse_string(chars: &mut Peekable<Chars>) -> Result<Token, Box<dyn Error>> {
        // The first token was a " so we don't really need it

        let mut value = String::new();

        while !chars.peek().is_none() {

            // If we have encountered a "
            if *chars.peek().unwrap() == '"' {

                // Try to get the last character we parsed
                match value.get((value.len() - 1)..) {
                    // If there is one
                    Some(s) => {
                        let c = s.chars().next().unwrap();

                        // If it was not escaped, that is the end of the string
                        if c != '\\' {
                            break;
                        }
                    },
                    // This would actually be the string "", which is valid
                    None => {
                        break;
                    }
                }
            }

            value.push(chars.next().unwrap());
        }

        chars.next();

        let fully = match Lexer::interpret_string(&value) {
            Ok(s) => s,
            Err(e) => {
                match e {
                    EscapeError::TrailingEscape => {
                        return Err("Found trailing escape while parsing string".into());
                    },
                    EscapeError::InvalidEscapedChar(c) => {
                        return Err(format!("Invalid escape sequence found: {}", c).into());
                    }
                }
            }
        };

        Ok(Token::STRING(fully))
    }

    fn parse_dotted(chars: &mut Peekable<Chars>) -> Result<Token, Box<dyn Error>> {
        let mut value = String::new();

        value.push_str(&Lexer::parse_alphanumeric(chars));

        Ok(if Lexer::is_next_char(chars, ':') {
            chars.next();

            Token::INNERLABEL(value)
        } else {
            Token::DIRECTIVE(value)
        })
    }

    fn parse_number(
        first_char: char,
        chars: &mut Peekable<Chars>,
    ) -> Result<Token, Box<dyn Error>> {
        let mut parsable = String::from(first_char);

        while !chars.peek().is_none()
            && (chars.peek().unwrap().is_ascii_hexdigit()
                || chars.peek().unwrap().is_ascii_digit()
                || *chars.peek().unwrap() == '_'
                || *chars.peek().unwrap() == 'x'
                || *chars.peek().unwrap() == '.')
        {
            if *chars.peek().unwrap() != '_' {
                parsable.push(chars.next().unwrap());
            }
            else {
                chars.next();
            }
        }

        if parsable.starts_with("0x") {
            Lexer::parse_hex_number(&parsable)
        } else if parsable.starts_with("0b") {
            Lexer::parse_binary_number(&parsable)
        } else if parsable.contains('.') {
            Lexer::parse_double_number(&parsable)
        } else {
            Lexer::parse_decimal_number(&parsable)
        }
    }

    fn parse_decimal_number(input: &str) -> Result<Token, Box<dyn Error>> {
        match i32::from_str(input) {
            Result::Ok(value) => Ok(Token::INT(value)),
            Err(_) => {
                return match i64::from_str(input) {
                    Result::Ok(_) => Err(format!("Integer literal {} too large.", input).into()),
                    Err(_) => Err(format!("Invalid int literal: {}", input).into())
                }
            }
        }
    }

    fn parse_double_number(input: &str) -> Result<Token, Box<dyn Error>> {
        match f64::from_str(input) {
            Result::Ok(value) => Ok(Token::DOUBLE(value)),
            Err(_) => {
                return Err(format!("Invalid double literal: {}", input).into());
            }
        }
    }

    fn parse_binary_number(input: &str) -> Result<Token, Box<dyn Error>> {
        let number_str = &input[2..];

        if number_str.is_empty() {
            return Err("Error trying to parse token 0b, expected hex literal".into());
        } else {
            match i32::from_str_radix(number_str, 2) {
                Result::Ok(value) => Ok(Token::INT(value)),
                Err(_) => {
                    return match i64::from_str_radix(number_str, 16) {
                        Result::Ok(_) => Err(format!("Binary literal {} too large to fit into an integer", input).into()),
                        Err(_) => Err(format!("Invalid binary literal {}", input).into()),
                    }
                }
            }
        }
    }

    fn parse_hex_number(input: &str) -> Result<Token, Box<dyn Error>> {
        let number_str = &input[2..];

        if number_str.is_empty() {
            return Err("Error trying to parse token 0x, expected hex literal".into());
        } else {
            match i32::from_str_radix(number_str, 16) {
                Result::Ok(value) => Ok(Token::INT(value)),
                Err(_) => {
                    return match i64::from_str_radix(number_str, 16) {
                        Result::Ok(_) => Err(format!("Hex literal {} too large to fit into an integer", input).into()),
                        Err(_) => Err(format!("Invalid hex literal {}", input).into()),
                    }
                }
            }
        }
    }

    fn parse_alphanumeric(chars: &mut Peekable<Chars>) -> String {
        let mut value = String::new();

        while !chars.peek().is_none()
            && (chars.peek().unwrap().is_ascii_alphanumeric() || *chars.peek().unwrap() == '_')
        {
            value.push(chars.next().unwrap());
        }

        value
    }

    fn is_next_char(chars: &mut Peekable<Chars>, value: char) -> bool {
        !chars.peek().is_none() && *chars.peek().unwrap() == value
    }

    fn parse_identifier(first_char: char, chars: &mut Peekable<Chars>) -> Token {
        let mut id = String::from(first_char);

        id.push_str(&Lexer::parse_alphanumeric(chars));

        if Lexer::is_next_char(chars, ':') {
            chars.next();

            Token::LABEL(id)
        } else {
            Token::IDENTIFIER(id)
        }
    }
}

// Source: https://stackoverflow.com/questions/58551211/how-do-i-interpret-escaped-characters-in-a-string

#[derive(Debug, PartialEq)]
enum EscapeError {
    TrailingEscape,
    InvalidEscapedChar(char)
}

struct EscapedStringInterpreter<'a> {
    s: std::str::Chars<'a>
}

impl<'a> Iterator for EscapedStringInterpreter<'a> {
    type Item = Result<char, EscapeError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.s.next().map(|c| match c {
            '\\' => match self.s.next() {
                None => Err(EscapeError::TrailingEscape),
                Some('n') => Ok('\n'),
                Some('\\') => Ok('\\'),
                Some('"') => Ok('"'),
                Some('t') => Ok('\t'),
                Some(c) => Err(EscapeError::InvalidEscapedChar(c)),
            },
            c => Ok(c),
        })
    }
}