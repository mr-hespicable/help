use crate::errors::TokenizerError;
use regex::Regex;

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
    DecimalIntegerLiteral(usize),
    Plus,   // 6 + 7
    Minus,  // minus sign
    Times,  // 6 * 9
    Divide, // 20 / 4
    Remainder, 
    LeftShift,
    RightShift,
    Lt,
    Gt,
    Le,
    Ge,
    Eq,
    Ne,
    LogicalAnd,
    LogicalOr,
    BitwiseAnd,
    BitwiseExclusiveOr,
    BitwiseOr,
    BitwiseComplement, // 4 = 100. ~4 = 011 = 3
    LogicalNegation,   // !1 = 0; !24 = 0, !0 = 1
    SimpleAssign,
    SumAssign,
    DiffAssign,
    ProdAssign,
    QuotAssign,
    RemAssign,
    LShAssign,
    RShAssign,
    AndAssign,
    XOrAssign,
    OrAssign,
}

pub const REGEX_TABLE: [&str; 40] = [
    r"int",
    r"return",
    r"\{",
    r"\}",
    r"\(",
    r"\)",
    r";",
    r"[a-zA-Z]\w*",
    r"[0-9]+",
    r"\+=",
    r"\-=",
    r"\*=",
    r"\/=",
    r"\%=",
    r"\<<=",
    r"\>>=",
    r"\&=",
    r"\^=",
    r"\|=",
    r"%",
    r"\+",
    r"-",
    r"\*",
    r"/",
    r"<<",
    r">>",
    r"<=",
    r">=",
    r"<",
    r">",
    r"==",
    r"!=",
    r"&&",
    r"\|\|",
    r"&",
    r"\^",
    r"\|",
    r"~",
    r"!",
    r"=",
];

pub fn get_token_from_string(s: &str) -> Result<Token, TokenizerError> {
    for re in REGEX_TABLE {
        if Regex::new(re).unwrap().is_match(s) {
            return match re {
                r"int" => Ok(Token::DataType(s.to_string())),
                r"return" => Ok(Token::Keyword(s.to_string())),
                r"\{" => Ok(Token::LeftBrace),
                r"\}" => Ok(Token::RightBrace),
                r"\(" => Ok(Token::LeftParen),
                r"\)" => Ok(Token::RightParen),
                r";" => Ok(Token::Semicolon),
                r"[a-zA-Z]\w*" => Ok(Token::Identifier(s.to_string())),
                r"[0-9]+" => Ok(Token::DecimalIntegerLiteral(s.parse().unwrap())),
                r"\+" => Ok(Token::Plus),
                r"-" => Ok(Token::Minus),
                r"\*" => Ok(Token::Times),
                r"/" => Ok(Token::Divide),
                r"%" => Ok(Token::Remainder),
                r"<<" => Ok(Token::LeftShift),
                r">>" => Ok(Token::RightShift),
                r"<=" => Ok(Token::Le),
                r">=" => Ok(Token::Ge),
                r"<" => Ok(Token::Lt),
                r">" => Ok(Token::Gt),
                r"==" => Ok(Token::Eq),
                r"!=" => Ok(Token::Ne),
                r"&&" => Ok(Token::LogicalAnd),
                r"\|\|" => Ok(Token::LogicalOr),
                r"&" => Ok(Token::BitwiseAnd),
                r"\^" => Ok(Token::BitwiseExclusiveOr),
                r"\|" => Ok(Token::BitwiseOr),
                r"~" => Ok(Token::BitwiseComplement),
                r"!" => Ok(Token::LogicalNegation),
                r"=" => Ok(Token::SimpleAssign),
                r"\+=" => Ok(Token::SumAssign),
                r"\-=" => Ok(Token::DiffAssign),
                r"\*=" => Ok(Token::ProdAssign),
                r"\/=" => Ok(Token::QuotAssign),
                r"\%=" => Ok(Token::RemAssign),
                r"\<<=" => Ok(Token::LShAssign),
                r"\>>=" => Ok(Token::RShAssign),
                r"\&=" => Ok(Token::AndAssign),
                r"\^=" => Ok(Token::XOrAssign),
                r"\|=" => Ok(Token::OrAssign),
                _ => Err(fail("regex does not match any in REGEX_TABLE")),
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
