use std::{error::Error, str::Chars, iter::Peekable, str::FromStr};

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
    NEWLINE,
    AMPERSAND,
    LABEL(String),
    INNERLABEL(String),
    DIRECTIVE(String),
    EOF
}

#[derive(PartialEq, Eq)]
enum NumberType {
    DEC,
    HEX,
    FLOAT,
    BIN
}

pub struct Lexer {

}

impl<'source> Lexer {

    pub fn lex(input: &'source str) -> Result<Vec<Token>, Box<dyn Error>> {

        let mut tokens: Vec<Token> = Vec::new();

        let mut chars = input.chars().peekable();

        while !chars.peek().is_none() {
            tokens.push( Lexer::parse_token(&mut chars)? );

            println!("---------------------------------------------------------------");

            for token in &tokens {
                println!("{:?}", token);
            }
        }

        while tokens.len() > 0 && (*tokens.last().unwrap() == Token::EOF || *tokens.last().unwrap() == Token::NEWLINE) {
            tokens.remove(tokens.len() - 1);
        }

        Ok(tokens)
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
            '_' | 'A'..='z' => {
                Lexer::parse_identifier(next_char, chars)
            },
            '0'..='9' => {
                Lexer::parse_number(next_char, chars)?
            },
            '"' => {
                Lexer::parse_string(chars)?
            },
            '.' => {
                if !chars.peek().is_none() {
                    if chars.peek().unwrap().is_ascii_digit() {
                        Lexer::parse_number(next_char, chars)?
                    } else {
                        Lexer::parse_dotted(chars)?
                    }
                } else {
                    return Err("Just . is not a token".into());
                }
            },
            '-' => Token::MINUS,
            '~' => Token::COMP,
            '!' => {
                if !chars.peek().is_none() && *chars.peek().unwrap() == '=' {
                    chars.next();
                    Token::NE
                }
                else {
                    Token::NEGATE
                }
            },
            '+' => Token::ADD,
            '*' => Token::MULT,
            '/' => Token::DIV,
            '%' => Token::MOD,
            '<' => {
                if !chars.peek().is_none() && *chars.peek().unwrap() == '=' {
                    chars.next();
                    Token::LTE
                }
                else {
                    Token::LT
                }
            },
            '>' => {
                if !chars.peek().is_none() && *chars.peek().unwrap() == '=' {
                    chars.next();
                    Token::GTE
                }
                else {
                    Token::GT
                }
            },
            '=' => {
                if !chars.peek().is_none() {
                    if *chars.peek().unwrap() != '=' {
                        return Err(format!("Found ={} expected ==", chars.peek().unwrap()).into());
                    }
                    else {
                        chars.next();
                        Token::EQ
                    }
                }
                else {
                    return Err("Found = expected ==".into());
                }
            },
            '&' => {
                if !chars.peek().is_none() && *chars.peek().unwrap() == '&' {
                    Token::AND
                }
                else {
                    Token::AMPERSAND
                }
            },
            '|' => {
                if !chars.peek().is_none() {
                    if *chars.peek().unwrap() != '|' {
                        return Err(format!("Found |{} expected ||", chars.peek().unwrap()).into());
                    }
                    else {
                        chars.next();
                        Token::OR
                    }
                }
                else {
                    return Err("Found | expected ||".into());
                }
            },
            '?' => Token::QUESTION,
            ':' => Token::COLON,
            '\n' => Token::NEWLINE,
            _ => {
                return Err(format!("Unexpected character {} while parsing token", next_char).into());
            }
        })

    }

    fn parse_string(chars: &mut Peekable<Chars>) -> Result<Token, Box<dyn Error>> {
        // The first token was a " so we don't really need it

        let mut value = String::new();

        while !chars.peek().is_none() && *chars.peek().unwrap() != '"' {
            value.push(chars.next().unwrap());
        }

        chars.next();

        Ok(Token::STRING(value))
    }

    fn parse_dotted(chars: &mut Peekable<Chars>) -> Result<Token, Box<dyn Error>> {

        let mut value = String::new();

        while !chars.peek().is_none() && (chars.peek().unwrap().is_ascii_alphanumeric() || *chars.peek().unwrap() == '_') {
            value.push(chars.next().unwrap());
        }

        Ok(if !chars.peek().is_none() && *chars.peek().unwrap() == ':' {
            chars.next();

            Token::INNERLABEL(value)
        } else {
            Token::DIRECTIVE(value)
        })
    }

    fn parse_number(first_char: char, chars: &mut Peekable<Chars>) -> Result<Token, Box<dyn Error>> {

        let mut number_type = NumberType::DEC;

        let mut parsable = String::with_capacity(1);

        let mut next_char;

        // It could be binary (0b) or hex (0x)
        if first_char == '0' {

            // If it is not the last character
            if !chars.peek().is_none() {
                next_char = *chars.peek().unwrap();

                // 0b is a binary integer
                if next_char == 'b' {
                    chars.next();
                    number_type = NumberType::BIN;
                }
                // 0x is a hex integer
                else if next_char == 'x' {
                    chars.next();
                    number_type = NumberType::HEX;
                }
                // If it is neither, then it might be nice to carry on
                else {
                    parsable.push(first_char);
                }
            }
            // If that is the last character, it is just a 0
            // That actually simplifies a lot if it is
            else {
                return Ok(Token::INT(0));
            }

        }
        else if first_char == '.' {
            number_type = NumberType::FLOAT;
            parsable.push('.');
        } else {
            parsable.push(first_char);
        }

        match number_type {
            NumberType::DEC => {

                next_char = *chars.peek().unwrap();

                while next_char.is_ascii_digit() || next_char == '_' {

                    if next_char != '_' {
                        parsable.push(next_char);
                    }

                    chars.next();

                    if chars.peek().is_none() {
                        break;
                    }
                    else {
                        next_char = *chars.peek().unwrap();
                    }
                }

                if !chars.peek().is_none() && *chars.peek().unwrap() == '.' {

                    number_type = NumberType::FLOAT;

                    parsable.push('.');

                    chars.next();

                    if !chars.peek().is_none() {
                        next_char = *chars.peek().unwrap();

                        while next_char.is_ascii_digit() {
    
                            parsable.push(next_char);

                            chars.next();
        
                            if chars.peek().is_none() {
                                break;
                            }
                            else {
                                next_char = *chars.peek().unwrap();
                            }
                        }
                    }
                }
            },
            NumberType::FLOAT => {
                next_char = *chars.peek().unwrap();

                while next_char.is_ascii_digit() || next_char == '_' {

                    if next_char != '_' {
                        parsable.push(next_char);
                    }

                    chars.next();

                    if chars.peek().is_none() {
                        break;
                    }
                    else {
                        next_char = *chars.peek().unwrap();
                    }
                }
            },
            NumberType::HEX => {

                if chars.peek().is_none() {
                    return Err("Error trying to parse token 0x, expected hex literal".into());
                }
                else {
                    next_char = *chars.peek().unwrap();
                }

                while next_char.is_ascii_hexdigit() {

                    parsable.push(next_char);

                    chars.next();

                    if chars.peek().is_none() {
                        break;
                    }
                    else {
                        next_char = *chars.peek().unwrap();
                    }
                }
            },
            NumberType::BIN => {

                if chars.peek().is_none() {
                    return Err("Error trying to parse token 0b, expected binary literal".into());
                }
                else {
                    next_char = *chars.peek().unwrap();
                }

                while next_char == '1' || next_char == '0' || next_char == '_' {

                    if next_char != '_' {
                        parsable.push(next_char);
                    }

                    chars.next();

                    if chars.peek().is_none() {
                        break;
                    }
                    else {
                        next_char = *chars.peek().unwrap();
                    }
                }
            }
        }

        Ok(match number_type {
            NumberType::DEC => Token::INT( i32::from_str(&parsable).expect(&format!("Somehow failed to parse the decimal number: {}", parsable)) ),
            NumberType::HEX => Token::INT( i32::from_str_radix(&parsable, 16).expect(&format!("Somehow failed to parse the hex number: {}", parsable)) ),
            NumberType::FLOAT => Token::DOUBLE( f64::from_str(&parsable).expect(&format!("Somehow failed to parse the double number: {}", parsable)) ),
            NumberType::BIN => Token::INT( i32::from_str_radix(&parsable, 2).expect(&format!("Somehow failed to parse the binary number: {}", parsable)))
        })
    }

    fn parse_identifier(first_char: char, chars: &mut Peekable<Chars>) -> Token {

        let mut id = String::from(first_char);

        while !chars.peek().is_none() && (chars.peek().unwrap().is_ascii_alphanumeric() || *chars.peek().unwrap() == '_') {
            id.push(chars.next().unwrap());
        }

        if !chars.peek().is_none() && *chars.peek().unwrap() == ':' {
            chars.next();

            Token::LABEL(id)
        } else {
            Token::IDENTIFIER(id)
        }
        

    }

}