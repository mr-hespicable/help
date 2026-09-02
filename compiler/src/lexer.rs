use regex::Regex;
use std::fs::File;
use std::io::Read;

use crate::errors::TokenizerError;
use crate::tokens::{self, Token, REGEX_TABLE};

pub struct Tokenizer {
    in_file: File,
}

impl Tokenizer {
    pub fn new(in_file: File) -> Tokenizer {
        Tokenizer { in_file }
    }

    fn load_file(&self) -> std::io::Result<String> {
        let mut file = &self.in_file;
        // dbg![&file];
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(contents)
    }

    pub fn tokenize(&self) -> Result<Vec<Token>, TokenizerError> {
        let contents = self.load_file().unwrap_or(String::new());
        // dbg![&contents];
        let mut results = vec![];

        let token_matching_str = REGEX_TABLE.join("|");
        dbg![&token_matching_str];

        let rgx = Regex::new(&token_matching_str).unwrap();

        let unsorted_matches: Vec<&str> = rgx.find_iter(&contents).map(|m| m.as_str()).collect();

        for m in unsorted_matches {
            results.push(tokens::get_token_from_string(m)?);
        }

        Ok(results)
    }
}
