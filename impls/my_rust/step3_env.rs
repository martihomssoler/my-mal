#![feature(iter_array_chunks)]

extern crate rustyline;

pub mod env;
pub mod printer;
pub mod reader;
pub mod types;

use env::Env;
use printer::*;
use reader::*;
use rustyline::{error::ReadlineError, DefaultEditor};
use types::*;

fn read(rl: &mut DefaultEditor) -> Result<MalType, ReadlineError> {
    let mut line = String::new();

    while line.is_empty() {
        line = rl.readline("user> ")?;
        rl.add_history_entry(&line).unwrap(); // TODO(mhs): remove unwrap
        rl.save_history(".mal-history").unwrap(); // TODO(mhs): remove unwrap
    }

    Ok(read_str(line.as_str()))
}

fn eval(ast: MalType, env: &mut Env) -> MalType {
    match ast {
        MalType::List(ref list) if list.is_empty() => ast,
        MalType::List(list) => apply(list, env),
        _ => eval_ast(ast, env),
    }
}

fn apply(list: Vec<MalType>, env: &mut Env) -> MalType {
    let first_elem = &list[0];

    match first_elem {
        MalType::Symbol(define) if define.eq("def!") => {
            let MalType::Symbol(ref s) = list[1] else {
                println!("ERROR: first element `{}` in def! is not a Symbol", list[1]);
                return MalType::None;
            };

            let v = eval(list[2].clone(), env);

            if matches!(v, MalType::None) {
                return MalType::None;
            }

            env.set(s, v.clone());
            return v;
        }
        MalType::Symbol(s) if s.eq("let*") => {
            let mut new_env = Env::new(Some(env.clone()));

            let new_bindings = match &list[1] {
                MalType::List(l) => l,
                MalType::Vector(v) => v,
                _ => {
                    println!(
                        "ERROR: first element `{}` in let* binding is not a List/Vector",
                        list[1]
                    );
                    return MalType::None;
                }
            };

            for [s, v] in new_bindings.iter().array_chunks::<2>() {
                let MalType::Symbol(ref new_key) = s else {
                    println!("ERROR: first element `{s}` in let* binding is not a Symbol");
                    return MalType::None;
                };
                let new_value = eval(v.clone(), &mut new_env);

                new_env.set(new_key, new_value);
            }

            let res = eval(list[2].clone(), &mut new_env);

            return res;
        }
        _ => (),
    }
    // new list as a result of calling eval on each member
    let mut evaled_list = Vec::new();

    for item in list {
        evaled_list.push(eval_ast(item, env));
    }

    // println!("evaluated list is {evaled_list:?}");
    let MalType::Function(ref b) = evaled_list[0] else {
        return MalType::None;
    };
    let func: *const dyn Fn(usize, *const MalType) -> MalType = *b;
    // println!("-> func is {func:?}");

    let owned_args = evaled_list.into_iter().skip(1).collect::<Vec<_>>();
    // println!("-> args are {owned_args:?}");
    let n = owned_args.len();
    let args: *const MalType = owned_args.as_ptr();

    let res = unsafe { (*func)(n, args) };
    // println!("==> res is {res:?}");

    res
}

fn eval_ast(ast: MalType, env: &mut Env) -> MalType {
    match ast {
        MalType::Symbol(s) => {
            // lookup symbol and return value or raise error
            if let Some(item) = env.get(s.as_str()) {
                item.clone()
            } else {
                println!("{s} not found.");
                MalType::None
            }
        }
        MalType::Vector(vector) => {
            let mut res = Vec::new();

            for item in vector {
                res.push(eval_ast(item, env));
            }

            MalType::Vector(res)
        }
        MalType::Dictionary(dict) => {
            let mut res = Vec::new();

            for item in dict {
                res.push(eval_ast(item, env));
            }

            MalType::Dictionary(res)
        }
        MalType::List(_) => eval(ast, env),
        _ => ast,
    }
}

fn print(res: MalType) -> String {
    pr_str(res)
}

fn rep(rl: &mut DefaultEditor, env: &mut Env) -> Result<String, ReadlineError> {
    let ast = read(rl)?;
    let result = eval(ast, env);
    Ok(print(result))
}

pub fn main() {
    let mut rl = DefaultEditor::new().unwrap(); // TODO(mhs): remove unwrap
    let _ = rl.load_history(".mal-history");
    let mut repl_env = env::Env::default();

    loop {
        match rep(&mut rl, &mut repl_env) {
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
