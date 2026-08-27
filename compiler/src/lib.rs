use crate::codegen::Generator;
use crate::lexer::Tokenizer;
use crate::parser::Parser;
use std::fs::File;
use std::io::Error;
use std::io::Write;
use std::process::Command;

pub mod ast;
pub mod codegen;
pub mod errors;
pub mod lexer;
pub mod parser;

pub fn to_file(filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file_name_no_ext: String = filename.rsplitn(2, ".").collect::<Vec<_>>()[1].to_string();
    let asm_file_name = format!("{}.s", file_name_no_ext);
    // dbg![&file_name_no_ext];
    let mut asm_file = File::create(&asm_file_name)?;

    // dbg![&filename];

    let tk = Tokenizer::new(File::open(filename).unwrap());
    let ag = tk.tokenize()?;
    // dbg![&ag];

    let mut prs = Parser::new(ag);
    let parsed = prs.parse()?;
    dbg![&parsed];

    let mut gn = Generator::new(parsed);
    let generated: String = gn.generate()?;

    println!("{}", generated);
    // let _ = asm_file.write_all(&generated.as_bytes());

    let _ = Command::new("gcc")
        .args(["-m64", &asm_file_name, "-o", &file_name_no_ext])
        .output()?;

    // println!("{:?}", out);

    Ok(())
}
