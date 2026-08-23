use std::{error::Error, fmt};

#[derive(Debug)]
pub struct LexerError(pub String);

impl Error for LexerError {}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let LexerError(s) = self;
        write!(f, "{}", s)
    }
}

#[derive(Debug)]
pub struct ParserError(pub String);

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ParserError(s) = self;
        write!(f, "{}", s)
    }
}

#[derive(Debug)]
pub struct GeneratorError(pub String);

impl fmt::Display for GeneratorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let GeneratorError(s) = self;
        write!(f, "{}", s)
    }
}
