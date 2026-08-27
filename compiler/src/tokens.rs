use regex::Regex;
use crate::errors::TokenizerError;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    DataType(String),
    Keyword(String),
    LeftBrace,
    RightBrace,
    LeftParen,
    RightParen,
    Semicolon,
    Identifier(String),
    DecimalIntegarLiteral(usize),
    BitwiseComplement, // 4 = 100. ~4 = 011 = 3
    LogicalNegation,   // !1 = 0; !24 = 0, !0 = 1
    Plus,               // 6 + 7
    Minus,             // minus sign
    Times,          // 6 * 9
    Divide,            // 20 / 4
}

pub const REGEX_TABLE: [&str; 15] = [
    r"int", // datatype
    r"return", // keyword
    r"\{", // left brace
    r"\}", // right brace
    r"\(", // left paren
    r"\)", // right paren
    r";", // semicolon
    r"[a-zA-Z]\w*", // identifier
    r"[0-9]+", // decimalintegerliteral
    r"~", // bitwise complement
    r"!", // logical negation
    r"\+", // add
    r"-", // minus
    r"\*", // multiply
    r"/", // divide
];

pub fn get_token_from_string(s: &str) -> Result<Token, TokenizerError> {
    for (i, re) in REGEX_TABLE.iter().enumerate() {
        if Regex::new(re).unwrap().is_match(s) {
            return match i {
                0 => Ok(Token::DataType(s.to_string())),
                1 => Ok(Token::Keyword(s.to_string())),
                2 => Ok(Token::LeftBrace),
                3 => Ok(Token::RightBrace),
                4 => Ok(Token::LeftParen),
                5 => Ok(Token::RightParen),
                6 => Ok(Token::Semicolon),
                7 => Ok(Token::Identifier(s.to_string())),
                8 => Ok(Token::DecimalIntegarLiteral(s.parse().unwrap())),
                9 => Ok(Token::BitwiseComplement),
                10 => Ok(Token::LogicalNegation),
                11 => Ok(Token::Plus),
                12 => Ok(Token::Minus),
                13 => Ok(Token::Times),
                14 => Ok(Token::Divide),
                _ => Err(fail("regex does not match any in REGEX_TABLE")) 
            };
        }
    }
    Err(fail("string does not match any regex in REGEX_TABLE"))
}

#[track_caller]
fn fail(message: &str) -> TokenizerError {
    let caller_location = std::panic::Location::caller();
    let err_string = "failed at".to_string();
    TokenizerError(format!(
        "{} {}; in {} @ {}:{}",
        message,
        err_string,
        caller_location.file(),
        caller_location.line(),
        caller_location.column()
    ))
}
