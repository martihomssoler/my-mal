extern crate rustyline;

use rustyline::{error::ReadlineError, DefaultEditor};

fn read(input: &str) -> &str {
    input
}

fn eval(input: &str) -> &str {
    input
}

fn print(input: &str) -> &str {
    input
}

fn rep(input: &str) -> &str {
    let ast = read(input);
    let result = eval(ast);
    print(result)
}

pub fn main() {
    let mut rl = DefaultEditor::new().unwrap(); // TODO(mhs): remove unwrap
    let _ = rl.load_history(".my-mal-history");

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
