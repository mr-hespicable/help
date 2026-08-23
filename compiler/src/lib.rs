use std::io::Write;
use std::process::Command;
use crate::lexer::Tokenizer;
use crate::parser::Parser;
use crate::codegen::Generator;
use std::fs::File;

pub mod lexer;
pub mod parser;
pub mod ast;
pub mod codegen;
pub mod errors;


pub fn to_file(filename: &str) -> Result<(), std::io::Error>{
    let file_name_no_ext: String = filename.rsplitn(2, ".").collect::<Vec<_>>()[1].to_string();
    let asm_file_name = format!("{}.s", file_name_no_ext);
    dbg![&file_name_no_ext];
    let mut asm_file = File::create(&asm_file_name)?;

    dbg![&filename];

    let tk = Tokenizer::new(File::open(filename).unwrap());
    let ag = tk.tokenize().expect("errored in tokenizer!");
    dbg![&ag];

    let mut prs = Parser::new(ag);
    let parsed = prs.parse().expect("errored in parser!");
    dbg![&parsed];

    let mut gn = Generator::new(parsed);
    let generated: String = gn.generate().expect("errored in generator!");

    println!("{}", generated);
    let _ = asm_file.write_all(&generated.as_bytes());

    let out = Command::new("gcc")
        .args(["-m64", &asm_file_name, "-o", &file_name_no_ext]).output().expect("failed to assemble assembly file into executable");

    println!("{:?}", out);

    Ok(())
}


