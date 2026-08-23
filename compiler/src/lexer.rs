use regex::Regex;
use std::fs::File;
use std::io::Read;

use crate::errors::LexerError;

#[derive(Debug, PartialEq)]
pub enum Token {
    Keyword(String),
    DataType(String),
    LeftBrace,
    RightBrace,
    LeftParen,
    RightParen,
    Semicolon,
    Identifier(String),
    DecimalIntegarLiteral(usize),
    Other,
    Negation,          // minus sign
    BitwiseComplement, // 4 = 100. !4 = 011 = 3
    LogicalNegation,   // !1 = 0; !24 = 0, !0 = 1
}

pub struct Tokenizer {
    in_file: File,
}

impl Tokenizer {
    pub fn new(in_file: File) -> Tokenizer {
        Tokenizer { in_file }
    }

    fn load_file(&self) -> std::io::Result<String> {
        let mut file = &self.in_file;
        dbg![&file];
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(contents)
    }

    pub fn tokenize(&self) -> Result<Vec<Token>, LexerError> {
        let contents = self.load_file().unwrap_or(String::new());
        dbg![&contents];
        let mut results = vec![];

        let tokens = [
            ("dt", r"int"),
            ("kw", r"return"),
            ("lb", r"\{"),
            ("rb", r"\}"),
            ("lp", r"\("),
            ("rp", r"\)"),
            ("sc", r";"),
            ("id", r"[a-zA-Z]\w*"),
            ("dc", r"[0-9]+"),
            ("ng", r"-"),
            ("bc", r"~"),
            ("lng", r"!"),
        ];

        let token_matching_str = tokens
            .into_iter()
            .map(|x| format!("({})", x.1))
            .collect::<Vec<String>>()
            .join("|");

        let rgx = Regex::new(&token_matching_str).unwrap();

        let unsorted_matches: Vec<&str> = rgx.find_iter(&contents).map(|m| m.as_str()).collect();

        for m in unsorted_matches {
            for (mc, re) in &tokens {
                if Regex::new(re).unwrap().is_match(m) {
                    let token: Token;
                    match *mc {
                        "kw" => {
                            token = Token::Keyword(m.to_string());
                        }
                        "dt" => {
                            token = Token::DataType(m.to_string());
                        }
                        "lb" => {
                            token = Token::LeftBrace;
                        }
                        "rb" => {
                            token = Token::RightBrace;
                        }
                        "lp" => {
                            token = Token::LeftParen;
                        }
                        "rp" => {
                            token = Token::RightParen;
                        }
                        "sc" => {
                            token = Token::Semicolon;
                        }
                        "id" => {
                            token = Token::Identifier(m.to_string());
                        }
                        "dc" => {
                            token = Token::DecimalIntegarLiteral(m.parse().unwrap());
                        }
                        "ng" => {
                            token = Token::Negation;
                        }
                        "bc" => {
                            token = Token::BitwiseComplement;
                        }
                        "lng" => {
                            token = Token::LogicalNegation;
                        }
                        _ => return Err(self.fail()),
                    };
                    results.push(token);
                    break;
                }
            }
        }

        if results.len() == 0 {
            return Err(self.fail());
        }

        Ok(results)
    }

    #[track_caller]
    fn fail(&self) -> LexerError {
        let caller_location = std::panic::Location::caller();
        let err_string = "failed at".to_string();
        LexerError(format!(
            "{}; in {} @ {}:{}",
            err_string,
            caller_location.file(),
            caller_location.line(),
            caller_location.column()
        ))
    }
}
