#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![deny(clippy::correctness)]

mod parser;
mod symbol_table;

use parser::Command::{A, C, L};
use parser::{ACommand, Parser};
use std::path::Path;
use symbol_table::SymbolTable;

pub fn assemble_file(path: &Path) -> Result<String, String> {
    let parser = parser::Parser::from_file(path).map_err(|err| err.to_string())?;
    assemble(&parser)
}

pub fn assemble_string(text: &str) -> Result<String, String> {
    let parser = parser::Parser::from_string(text).map_err(|err| err.to_string())?;
    assemble(&parser)
}

fn assemble(parser: &Parser) -> Result<String, String> {
    let mut result = String::new();
    let mut table = SymbolTable::new(parser)?;
    for line in parser.iter() {
        let asm = match line?.command {
            A(ACommand::Constant(num)) => format!("{:016b}\n", num),
            A(ACommand::Symbol(sym)) => format!("{:016b}\n", table.get_address(&sym)),
            C(code) => format!("{:016b}\n", code),
            L(_) => "".to_string(),
        };
        result.push_str(&asm);
    }
    Ok(result)
}
