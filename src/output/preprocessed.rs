use crate::{Token, TokenType, TokenData};

/// Converts tokens back to source as best as it can
pub fn tokens_to_text(tokens: &Vec<Token>) -> String {
    let mut output = String::new();

    // For each token, output it to the string
    for token in tokens.iter() {
        let data = token.data();
        let tt = token.tt();

        let st = match tt {
            TokenType::OPENPAREN => String::from("( "),
            TokenType::CLOSEPAREN => String::from(") "),
            TokenType::IDENTIFIER => {
                match data { TokenData::STRING(s) => format!("{} ", s), _ => unreachable!() }
            },
            TokenType::INT => {
                match data { TokenData::INT(i) => format!("{} ", i), _ => unreachable!() }
            },
            TokenType::DOUBLE => {
                match data { TokenData::DOUBLE(d) => format!("{} ", d), _ => unreachable!() }
            },
            TokenType::STRING => {
                match data { TokenData::STRING(s) => format!("\"{}\" ", s), _ => unreachable!() }
            },
            TokenType::MINUS => String::from("- "),
            TokenType::COMP => String::from("~"),
            TokenType::NEGATE => String::from("!"),
            TokenType::ADD => String::from("+ "),
            TokenType::MULT => String::from("* "),
            TokenType::DIV => String::from("/ "),
            TokenType::MOD => String::from("% "),
            TokenType::AND => String::from("&& "),
            TokenType::OR => String::from("|| "),
            TokenType::EQ => String::from("== "),
            TokenType::NE => String::from("!= "),
            TokenType::LT => String::from("< "),
            TokenType::LTE => String::from("<= "),
            TokenType::GT => String::from(">= "),
            TokenType::GTE => String::from(">= "),
            TokenType::QUESTION => String::from("? "),
            TokenType::COLON => String::from(": "),
            TokenType::COMMA => String::from(", "),
            TokenType::NEWLINE => String::from("\n"),
            TokenType::AMPERSAND => String::from("&"),
            TokenType::LABEL => {
                match data { TokenData::STRING(s) => format!("{}:", s), _ => unreachable!() }
            },
            TokenType::INNERLABEL => {
                match data { TokenData::STRING(s) => format!(".{}:", s), _ => unreachable!() }
            },
            TokenType::DIRECTIVE => {
                match data { TokenData::STRING(s) => format!(".{}", s), _ => unreachable!() }
            },
            TokenType::LINECONTINUE => String::from("\\"),
            TokenType::PLACEHOLDER => {
                match data { TokenData::INT(i) => format!("{} ", i), _ => unreachable!() }
            },
            TokenType::COMMENT => {
                match data { TokenData::STRING(s) => format!("; {}", s), _ => unreachable!() }
            },
            TokenType::DOLLAR => String::from("$"),
            TokenType::HASH => String::from("#"),
            TokenType::ATLabel => String::from("@"),
        };

        output.push_str(&st);
    }

    output
}