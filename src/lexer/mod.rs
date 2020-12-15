use std::{error::Error, iter::Peekable, str::Chars, str::FromStr};

/// Represents the type of any given token
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TokenType {
    OPENPAREN,
    CLOSEPAREN,
    IDENTIFIER,
    INT,
    DOUBLE,
    STRING,
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
    LABEL,
    INNERLABEL,
    DIRECTIVE,
    LINECONTINUE,
    PLACEHOLDER,
    COMMENT,
    DOLLAR,
    HASH,
    ATLabel
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenData {
    STRING(String),
    INT(i32),
    DOUBLE(f64),
    NONE
}

// Produced by the lexer, it is the smallest element that can be parsed, it contains the token's data and position in the source code
#[derive(Debug, Clone)]
pub struct Token {
    tt: TokenType,
    data: TokenData,
    col: usize,
    line: usize,
    file: usize
}

impl Token {
    pub fn new(tt: TokenType, data: TokenData) -> Token {
        Token {
            tt,
            data,
            col: 0,
            line: 0,
            file: 0
        }
    }

    pub fn set_col(&mut self, col: usize) {
        self.col = col;
    }

    pub fn set_line(&mut self, line: usize) {
        self.line = line
    }

    pub fn set_file(&mut self, file: usize) {
        self.file = file
    }

    pub fn col(&self) -> usize {
        self.col
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn file(&self) -> usize {
        self.file
    }

    pub fn tt(&self) -> TokenType {
        self.tt
    }

    pub fn data(&self) -> &TokenData {
        &self.data
    }

    pub fn as_str(&self) -> String {
        let mut s = format!("{:?}", self.tt);

        match &self.data {
            TokenData::DOUBLE(d) => {
                s.push_str(&format!(": {}", d));
            },
            TokenData::INT(i) => {
                s.push_str(&format!(": {}", i));
            },
            TokenData::STRING(v) => {
                s.push_str(&format!(": \"{}\"", v));
            },
            TokenData::NONE => {}
        }

        s
    }
}

pub struct Lexer {
    line_map: Vec<(usize, usize)>,
    column_count: usize,
    source_index: usize
}

impl Lexer {

    /// Creates a new lexer
    pub fn new() -> Lexer {
        Lexer {
            line_map: vec![(0, 0)],
            column_count: 0,
            source_index: 0
        }
    }

    /// Lexes the given source and returns a vector of token structs
    pub fn lex(&mut self, input: &str, file_name: &str, file_id: usize) -> Result<Vec<Token>, Box<dyn Error>> {
        // The vector that will contain all of the tokens
        let mut tokens: Vec<Token> = Vec::new();

        // This lexer goes through the source character by character
        let mut chars = input.chars().peekable();

        // While we have another character
        while chars.peek().is_some() {

            // Consume the whitespace, but if we have reached the end of the file, break from the loop too
            if self.consume_whitespace(&mut chars) {
                break;
            }

            // Lex another token if there are more characters
            let (mut token, length) = match self.lex_token(&mut chars) {
                Ok(ret) => ret,
                Err(e) => {
                    let msg = format!("Error lexing inputin file {}: {}, line {}.\n", file_name, e, self.line_map.len());
                    return Err( msg.into() )
                },
            };

            if token.tt() == TokenType::NEWLINE {
                // Reset to start with the next line of the source
                self.source_index += 1 + self.column_count;
                self.line_map.last_mut().unwrap().1 = self.source_index - 1;
                self.line_map.push((self.source_index, 0));
                self.column_count = 0;
            } else {
                self.column_count += length;
            }

            // Set this token's column and line numbers
            token.set_col(self.column_count);
            token.set_line(self.line_map.len());

            // Finally, add the token to the list
            tokens.push(token);
        }

        // As long as there is at least one token in the vector, and the last token is a newline
        while tokens.len() > 0 && tokens.last().unwrap().tt() == TokenType::NEWLINE
        {
            // Remove it so that there are no random trailing tokens
            tokens.remove(tokens.len() - 1);
        }

        // Also, just in case there was no trailing newline in the input at all, make sure the last line ends in the map
        self.line_map.last_mut().unwrap().1 = self.source_index - 1;

        // Remove all comment tokens from the list
        self.remove_comments(&mut tokens);

        // Set the file id for each token to the current file
        self.set_file(&mut tokens, file_id);

        // Finally, remove the line continue characters because they can get in the way later
        match self.remove_line_continues(&mut tokens) {
            Ok(_) => {},
            Err(e) => {
                let msg = format!("Error lexing input: {}, line {}.\n", e, self.line_map.len());
                return Err( msg.into() )
            },
        };

        // We are done with this step
        Ok(tokens)
    }

    /// Sets the file member of each token to the specified file id
    fn set_file(&self, tokens: &mut Vec<Token>, file: usize) {
        for token in tokens.iter_mut() {
            token.set_file(file);
        }
    }

    /// Consumes whitespace characters all except for newlines, returns true if the end of the iterator has been reached
    fn consume_whitespace(&mut self, chars: &mut Peekable<Chars>) -> bool {
        // This will run if the next character is whitespace
        while chars.peek().is_some() && match chars.peek().unwrap() { '\t' | ' ' => true, _ => false } {
            self.column_count += 1; // Add it to the count
            chars.next(); // Consume it and move on to the next
        }

        // Returns true if there is not another character
        chars.peek().is_none()
    }

    fn remove_line_continues(&self, tokens: &mut Vec<Token>) -> Result<(), Box<dyn Error>> {
        // I was originally going to tokens.remove() all of the line continues and newlines
        // But it turns out that copies the entire vector every time, so why not just do that?

        // We will allocate the max, and then shrink it later
        let mut new_tokens: Vec<Token> = Vec::with_capacity(tokens.len());

        let mut skip_next = false;
        for index in 0..tokens.len() {
            if !skip_next {
                if tokens[index].tt() == TokenType::LINECONTINUE {
                    if index < tokens.len() - 1 && tokens[index + 1].tt() != TokenType::NEWLINE {
                        return Err(format!(
                            "Error parsing, \\ should only be followed by a newline. Found: {}",
                            tokens[index + 1].as_str()
                        )
                        .into());
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

    /// Removes all comment tokens from the token list
    fn remove_comments(&mut self, tokens: &mut Vec<Token>) {
        // We will allocate the max, and then shrink it later
        let mut new_tokens: Vec<Token> = Vec::with_capacity(tokens.len());

        for index in 0..tokens.len() {
            if tokens[index].tt() != TokenType::COMMENT {
                new_tokens.push(tokens[index].to_owned());
            }
        }

        tokens.clear();
        tokens.append(&mut new_tokens);

        tokens.shrink_to_fit();
    }

    /// Lexes a single token from the character iterator. Returns a tuple containing the token, and the number of characters that the token spans in the source
    fn lex_token(&self, chars: &mut Peekable<Chars>) -> Result<(Token, usize), Box<dyn Error>> {

        // We are guaranteed to have another character, so just get the next one
        let c = chars.next().unwrap();

        Ok(match c {
            '(' => (Token::new(TokenType::OPENPAREN, TokenData::NONE), 1),
            ')' => (Token::new(TokenType::CLOSEPAREN, TokenData::NONE), 1),
            '\\' => (Token::new(TokenType::LINECONTINUE, TokenData::NONE), 1),
            '_' | 'A'..='z' => Lexer::lex_identifier(c, chars),
            '0'..='9' => Lexer::lex_number(c, chars)?,
            '"' => Lexer::lex_string(chars)?,
            '.' => {
                if !chars.peek().is_none() {
                    if chars.peek().unwrap().is_ascii_digit() {
                        Lexer::lex_number(c, chars)?
                    } else {
                        Lexer::lex_dotted(chars)?
                    }
                } else {
                    return Err("Found lone .".into());
                }
            }
            '-' => (Token::new(TokenType::MINUS, TokenData::NONE), 1),
            '~' => (Token::new(TokenType::COMP, TokenData::NONE), 1),
            '!' => {
                if !chars.peek().is_none() && *chars.peek().unwrap() == '=' {
                    chars.next();
                    (Token::new(TokenType::NE, TokenData::NONE), 2)
                } else {
                    (Token::new(TokenType::NEGATE, TokenData::NONE), 1)
                }
            }
            '+' => (Token::new(TokenType::ADD, TokenData::NONE), 1),
            '*' => (Token::new(TokenType::MULT, TokenData::NONE), 1),
            '/' => (Token::new(TokenType::DIV, TokenData::NONE), 1),
            '%' => (Token::new(TokenType::MOD, TokenData::NONE), 1),
            '<' => {
                if !chars.peek().is_none() && *chars.peek().unwrap() == '=' {
                    chars.next();
                    (Token::new(TokenType::LTE, TokenData::NONE), 2)
                } else {
                    (Token::new(TokenType::LT, TokenData::NONE), 1)
                }
            }
            '>' => {
                if !chars.peek().is_none() && *chars.peek().unwrap() == '=' {
                    chars.next();
                    (Token::new(TokenType::GTE, TokenData::NONE), 2)
                } else {
                    (Token::new(TokenType::GT, TokenData::NONE), 1)
                }
            }
            '=' => {
                if !chars.peek().is_none() {
                    if *chars.peek().unwrap() != '=' {
                        return Err(format!("Found ={} expected ==", chars.peek().unwrap()).into());
                    } else {
                        chars.next();
                        (Token::new(TokenType::EQ, TokenData::NONE), 2)
                    }
                } else {
                    return Err("Found = expected ==".into());
                }
            }
            '&' => {
                if !chars.peek().is_none() && *chars.peek().unwrap() == '&' {
                    (Token::new(TokenType::AND, TokenData::NONE), 2)
                } else {
                    (Token::new(TokenType::AMPERSAND, TokenData::NONE), 1)
                }
            }
            '|' => {
                if !chars.peek().is_none() {
                    if *chars.peek().unwrap() != '|' {
                        return Err(format!("Found |{} expected ||", chars.peek().unwrap()).into());
                    } else {
                        chars.next();
                        (Token::new(TokenType::OR, TokenData::NONE), 2)
                    }
                } else {
                    return Err("Found | expected ||".into());
                }
            }
            '?' => (Token::new(TokenType::QUESTION, TokenData::NONE), 1),
            ':' => (Token::new(TokenType::COLON, TokenData::NONE), 1),
            '\n' => (Token::new(TokenType::NEWLINE, TokenData::NONE), 1),
            ',' => (Token::new(TokenType::COMMA, TokenData::NONE), 1),
            ';' => Lexer::lex_comment(chars),
            '$' => (Token::new(TokenType::DOLLAR, TokenData::NONE), 1),
            '#' => (Token::new(TokenType::HASH, TokenData::NONE), 1),
            '@' => (Token::new(TokenType::ATLabel, TokenData::NONE), 1),
            '\r' => {
                // This is a carriage return which will always be followed by a newline.
                // Consume the newline
                chars.next();
                // Return a newline token
                (Token::new(TokenType::NEWLINE, TokenData::NONE), 2)
            },
            _ => {
                return Err(
                    format!("Unexpected character {} while parsing token", c).into()
                );
            }
        })
    }

    fn interpret_string(s: &str) -> Result<String, EscapeError> {
        (EscapedStringInterpreter { s: s.chars() }).collect()
    }

    fn lex_string(chars: &mut Peekable<Chars>) -> Result<(Token, usize), Box<dyn Error>> {
        // The first token was a " so we don't really need it, but it still counts towards the size
        let mut size = 1;

        let mut value = String::new();

        while !chars.peek().is_none() {
            // If we have encountered a "
            if *chars.peek().unwrap() == '"' {
                // Try to get the last character we parsed
                let last_index;

                if value.len() > 0 {
                    last_index = value.len() - 1;
                } else {
                    last_index = 0;
                }

                match value.get(last_index..) {
                    // If there is one
                    Some(s) => {
                        let c = s.chars().next();

                        // If it was not escaped, that is the end of the string
                        if c.is_none() || c.unwrap() != '\\' {
                            break;
                        }
                    }
                    // This would actually be the string "", which is valid
                    None => {
                        break;
                    }
                }
            }

            // If everything is in order, push the character, and add 1 to the size
            value.push(chars.next().unwrap());
            size += 1;
        }

        // This will be the ending quote, which does count for the size
        chars.next();
        size += 1;

        // This will resolve any escape sequences in the string to their proper values
        let fully = match Lexer::interpret_string(&value) {
            Ok(s) => s,
            Err(e) => match e {
                EscapeError::TrailingEscape => {
                    return Err("Found trailing escape while parsing string".into());
                }
                EscapeError::InvalidEscapedChar(c) => {
                    return Err(format!("Invalid escape sequence found: {}", c).into());
                }
            },
        };

        // Return the token, and the size of it
        Ok((Token::new(TokenType::STRING, TokenData::STRING(fully)), size))
    }

    /// This function lexes any string value that begins with a . character, this could be an inner label or a directive
    fn lex_dotted(chars: &mut Peekable<Chars>) -> Result<(Token, usize), Box<dyn Error>> {
        let mut value = String::new();
        let mut size;

        // The inner value of this token will just be whatever the alphanumeric characters following this will be
        value.push_str(&Lexer::parse_alphanumeric(chars));
        size = value.len();

        // If the next character is a :, then it is an inner label
        Ok(if Lexer::is_next_char(chars, ':') {
            // Consume the :, and add 1 to the size
            chars.next();
            size += 1;

            (Token::new(TokenType::INNERLABEL, TokenData::STRING(value)), size)
        } else {
            (Token::new(TokenType::DIRECTIVE, TokenData::STRING(value)), size)
        })
    }

    // Parses any type of number from the input, and produces either an int or double token
    fn lex_number(
        first_char: char,
        chars: &mut Peekable<Chars>,
    ) -> Result<(Token, usize), Box<dyn Error>> {
        // This function works by reading in the number as a string and parsing it later
        let mut parsable = String::from(first_char);

        // Basically, this will read in any characters that make a valid number
        while !chars.peek().is_none()
            && (chars.peek().unwrap().is_ascii_hexdigit()
                || chars.peek().unwrap().is_ascii_digit()
                || *chars.peek().unwrap() == '_'
                || *chars.peek().unwrap() == 'x'
                || *chars.peek().unwrap() == '.')
        {
            if *chars.peek().unwrap() != '_' {
                parsable.push(chars.next().unwrap());
            } else {
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

    fn parse_decimal_number(input: &str) -> Result<(Token, usize), Box<dyn Error>> {
        // Because of the nature of this, the size is actually just the size of the string
        let size = input.len();

        match i32::from_str(input) {
            Result::Ok(value) => Ok((Token::new(TokenType::INT, TokenData::INT(value)), size)),
            Err(_) => {
                return match i64::from_str(input) {
                    Result::Ok(_) => Err(format!("Integer literal {} too large.", input).into()),
                    Err(_) => Err(format!("Invalid int literal: {}", input).into()),
                }
            }
        }
    }

    fn parse_double_number(input: &str) -> Result<(Token, usize), Box<dyn Error>> {
        // Because of the nature of this, the size is actually just the size of the string
        let size = input.len();

        match f64::from_str(input) {
            Result::Ok(value) => Ok((Token::new(TokenType::DOUBLE, TokenData::DOUBLE(value)), size)),
            Err(_) => {
                return Err(format!("Invalid double literal: {}", input).into());
            }
        }
    }

    fn parse_binary_number(input: &str) -> Result<(Token, usize), Box<dyn Error>> {
        // Because of the nature of this, the size is actually just the size of the string
        let size = input.len();

        // This contains the number with the leading 0b stripped off
        let number_str = &input[2..];

        if number_str.is_empty() {
            return Err("Error trying to parse token 0b, expected hex literal".into());
        } else {
            match i32::from_str_radix(number_str, 2) {
                Result::Ok(value) => Ok((Token::new(TokenType::INT, TokenData::INT(value)), size)),
                Err(_) => {
                    return match i64::from_str_radix(number_str, 16) {
                        Result::Ok(_) => Err(format!(
                            "Binary literal {} too large to fit into an integer",
                            input
                        )
                        .into()),
                        Err(_) => Err(format!("Invalid binary literal {}", input).into()),
                    }
                }
            }
        }
    }

    fn parse_hex_number(input: &str) -> Result<(Token, usize), Box<dyn Error>> {
        // Because of the nature of this, the size is actually just the size of the string
        let size = input.len();

        // This contains the number with the leading 0x stripped off
        let number_str = &input[2..];

        if number_str.is_empty() {
            return Err("Error trying to parse token 0x, expected hex literal".into());
        } else {
            match i32::from_str_radix(number_str, 16) {
                Result::Ok(value) => Ok((Token::new(TokenType::INT, TokenData::INT(value)), size)),
                Err(_) => {
                    return match i64::from_str_radix(number_str, 16) {
                        Result::Ok(_) => Err(format!(
                            "Hex literal {} too large to fit into an integer",
                            input
                        )
                        .into()),
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

    fn lex_identifier(first_char: char, chars: &mut Peekable<Chars>) -> (Token, usize) {
        let mut id = String::from(first_char);
        let mut size;

        // The inner value of this token will just be whatever the alphanumeric characters following this will be
        id.push_str(&Lexer::parse_alphanumeric(chars));
        size = id.len();

        // If the next character is a :, then it is a label
        if Lexer::is_next_char(chars, ':') {
            // Consume the :, and add 1 to the size
            chars.next();
            size += 1;

            (Token::new(TokenType::LABEL, TokenData::STRING(id)), size)
        } else {
            (Token::new(TokenType::IDENTIFIER, TokenData::STRING(id)), size)
        }
    }

    /// Consumes a comment until the end of the stream or until a newline
    fn lex_comment(chars: &mut Peekable<Chars>) -> (Token, usize) {
        let mut contents = String::new();
        let size;

        // Get rid of that ;
        chars.next();

        // While there is another character, and it isn't a newline, consume it
        while chars.peek().is_some() && (*chars.peek().unwrap() != '\n' && *chars.peek().unwrap() != '\r') {
            contents.push(chars.next().unwrap());
        }

        size = contents.len() + 1;

        (Token::new(TokenType::COMMENT, TokenData::STRING(contents)), size)
    }
}

// Source: https://stackoverflow.com/questions/58551211/how-do-i-interpret-escaped-characters-in-a-string

#[derive(Debug, PartialEq)]
enum EscapeError {
    TrailingEscape,
    InvalidEscapedChar(char),
}

struct EscapedStringInterpreter<'a> {
    s: std::str::Chars<'a>,
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
