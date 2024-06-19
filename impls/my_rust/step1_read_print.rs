extern crate rustyline;

pub mod printer;
pub mod reader;
pub mod types;

use printer::*;
use reader::*;
use rustyline::{error::ReadlineError, DefaultEditor};
use types::*;

fn read(input: &str) -> MalType {
    read_str(input)
}

fn eval(ast: MalType) -> MalType {
    ast
}

fn print(res: MalType) -> String {
    pr_str(res)
}

fn rep(input: &str) -> String {
    let ast = read(input);
    let result = eval(ast);
    print(result)
}

pub fn main() {
    let mut rl = DefaultEditor::new().unwrap(); // TODO(mhs): remove unwrap
    let _ = rl.load_history(".mal-history");

    loop {
        let readline = rl.readline("user> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(&line).unwrap(); // TODO(mhs): remove unwrap
                rl.save_history(".mal-history").unwrap(); // TODO(mhs): remove unwrap
                if !line.is_empty() {
                    let output = rep(&line);
                    // print result
                    println!("{output}");
                }
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
