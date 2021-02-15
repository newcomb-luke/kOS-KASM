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
    HASH,
    ATSYMBOL,
    BOOL,
    FUNCTION,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenData {
    STRING(String),
    INT(i32),
    DOUBLE(f64),
    BOOL(bool),
    NONE,
}

// Produced by the lexer, it is the smallest element that can be parsed, it contains the token's data and position in the source code
#[derive(Debug, Clone)]
pub struct Token {
    tt: TokenType,
    data: TokenData,
    line: usize,
    file: usize,
}

impl Token {
    pub fn new(tt: TokenType, data: TokenData) -> Token {
        Token {
            tt,
            data,
            line: 0,
            file: 0,
        }
    }

    pub fn set_line(&mut self, line: usize) {
        self.line = line
    }

    pub fn set_file(&mut self, file: usize) {
        self.file = file
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
        let data_str = match &self.data {
            TokenData::DOUBLE(d) => {
                format!("{}", d)
            }
            TokenData::INT(i) => {
                format!("{}", i)
            }
            TokenData::STRING(v) => {
                format!("\"{}\"", v)
            }
            TokenData::BOOL(v) => {
                if *v {
                    String::from("true")
                } else {
                    String::from("false")
                }
            }
            TokenData::NONE => {
                return format!("{:?}", self.tt);
            }
        };

        format!("{:?}: {}", self.tt, data_str)
    }
}
