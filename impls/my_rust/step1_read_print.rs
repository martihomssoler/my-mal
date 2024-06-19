extern crate rustyline;

pub mod printer;
pub mod reader;
pub mod types;

use printer::*;
use reader::*;
use rustyline::{error::ReadlineError, DefaultEditor};
use types::*;

fn read(rl: &mut DefaultEditor) -> Result<MalType, ReadlineError> {
    let line = rl.readline("user> ")?;
    rl.add_history_entry(&line).unwrap(); // TODO(mhs): remove unwrap
    rl.save_history(".mal-history").unwrap(); // TODO(mhs): remove unwrap

    if !line.is_empty() {
        Ok(read_str(line.as_str()))
    } else {
        Err(ReadlineError::Eof)
    }
}

fn eval(ast: MalType) -> MalType {
    ast
}

fn print(res: MalType) -> String {
    pr_str(res)
}

fn rep(rl: &mut DefaultEditor) -> Result<String, ReadlineError> {
    let ast = read(rl)?;
    let result = eval(ast);
    Ok(print(result))
}

pub fn main() {
    let mut rl = DefaultEditor::new().unwrap(); // TODO(mhs): remove unwrap
    let _ = rl.load_history(".mal-history");

    loop {
        match rep(&mut rl) {
            Ok(line) => {
                println!("{line}");
            }
            Err(ReadlineError::Interrupted) => continue,
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
}
