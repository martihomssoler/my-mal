extern crate rustyline;

pub mod env;
pub mod printer;
pub mod reader;
pub mod types;

use once_cell::sync::Lazy;
use printer::*;
use reader::*;
use rustyline::{error::ReadlineError, DefaultEditor};
use std::sync::Mutex;
use types::*;

pub static ENV: Lazy<Mutex<env::ReplEnv>> = Lazy::new(|| {
    let env = env::ReplEnv::default();
    Mutex::new(env)
});

fn read(rl: &mut DefaultEditor) -> Result<MalType, ReadlineError> {
    let mut line = String::new();

    while line.is_empty() {
        line = rl.readline("user> ")?;
        rl.add_history_entry(&line).unwrap(); // TODO(mhs): remove unwrap
        rl.save_history(".mal-history").unwrap(); // TODO(mhs): remove unwrap
    }

    Ok(read_str(line.as_str()))
}

fn eval(ast: MalType) -> MalType {
    match ast {
        MalType::List(ref list) if list.is_empty() => ast,
        MalType::List(list) => {
            // new list as a result of calling eval on each member
            let mut evaled_list = Vec::new();

            for item in list {
                //     println!("evaluating item {item}");
                evaled_list.push(eval_ast(item));
            }

            // println!("evaluated list is {evaled_list:?}");

            let MalType::Function(ref b) = evaled_list[0] else {
                return MalType::None;
            };
            let func: *const env::FuncPtr<i64> = b.cast();
            // println!("-> func is {func:?}");

            let owned_args = evaled_list
                .into_iter()
                .skip(1)
                .map(coerse_to_i64)
                .collect::<Vec<_>>();
            // println!("-> args are {owned_args:?}");
            let n = owned_args.len();
            let args: *const i64 = owned_args.as_ptr();

            let res = unsafe { (*func)(n, args) };
            // println!("==> res is {res:?}");

            MalType::Number(res)
        }
        _ => eval_ast(ast),
    }
}

fn coerse_to_i64(mt: MalType) -> i64 {
    match mt {
        MalType::List(_) => coerse_to_i64(eval_ast(mt)),
        MalType::Number(n) => n,
        _ => panic!("wrong data type for term {mt}"),
    }
}

fn eval_ast(ast: MalType) -> MalType {
    match ast {
        MalType::Symbol(s) => {
            // lookup symbol and return value or raise error
            if let Some(func) = ENV.lock().unwrap().symbols.get(s.as_str()) {
                MalType::Function(func as *const dyn std::any::Any)
            } else {
                panic!("Undefined symbol {s:?}")
            }
        }
        MalType::Vector(vector) => {
            let mut res = Vec::new();

            for item in vector {
                res.push(eval_ast(item));
            }

            MalType::Vector(res)
        }
        MalType::Dictionary(dict) => {
            let mut res = Vec::new();

            for item in dict {
                res.push(eval_ast(item));
            }

            MalType::Dictionary(res)
        }
        MalType::List(_) => eval(ast),
        _ => ast,
    }
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
