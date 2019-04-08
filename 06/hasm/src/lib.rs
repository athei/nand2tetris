#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![deny(clippy::correctness)]

mod parser;

use parser::ACommand;
use parser::Command::{A, C, L};
use std::path::Path;

pub fn assemble(path: &Path) -> Result<String, String> {
    let parser = parser::Parser::from_file(path).map_err(|err| err.to_string())?;

    for line in parser.iter() {
        match line?.command {
            A(ACommand::Constant(num)) => format!("0{:015b}\n", num),
            C(code) => format!("111{:013b}\n", code),
            L(_) => "".to_string(),
            _ => unimplemented!(),
        };
    }

    unimplemented!()
}
