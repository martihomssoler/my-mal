#![feature(iter_array_chunks)]

extern crate rustyline;

pub mod core;
pub mod env;
pub mod printer;
pub mod reader;
pub mod types;

use core::*;
use env::*;
use printer::*;
use reader::*;
use rustyline::{error::ReadlineError, DefaultEditor};
use std::{borrow::Borrow, ops::Deref};
use types::*;

fn read(line: &str) -> MalType {
    read_str(line)
}

fn eval(mut ast: MalType, mut env: Env) -> MalType {
    loop {
        match ast {
            MalType::List(ref list) if list.is_empty() => return ast,
            MalType::List(list) => {
                let first_elem = &list[0];

                match first_elem {
                    MalType::Symbol(define_symbol) if define_symbol.eq("def!") => {
                        let v = eval_ast(list[2].clone(), &env);

                        if matches!(v, MalType::Nil) {
                            return MalType::Nil;
                        }

                        env_set(&env, &list[1], v.clone());
                        return v;
                    }
                    MalType::Symbol(let_symbol) if let_symbol.eq("let*") => {
                        let new_env =
                            env_bind(Some(env.clone()), list[1].clone(), list[2..].to_vec());

                        let new_bindings = match &list[1] {
                            MalType::List(l) => l,
                            MalType::Vector(v) => v,
                            _ => {
                                println!(
                                    "ERROR: first element `{}` in let* binding is not a List/Vector",
                                    print_string(&list[1], true)
                                );
                                return MalType::Nil;
                            }
                        };

                        for [s, v] in new_bindings.iter().array_chunks::<2>() {
                            let new_value = eval(v.clone(), new_env.clone());
                            env_set(&new_env, s, new_value);
                        }

                        // tco
                        env = new_env;
                        ast = list[2].clone();
                    }
                    MalType::Symbol(do_symbol) if do_symbol.eq("do") => {
                        let mut last_item = MalType::Nil;

                        for item in list.iter().skip(1) {
                            last_item = eval_ast(item.clone(), &env);
                        }

                        // tco
                        ast = last_item;
                    }
                    MalType::Symbol(if_symbol) if if_symbol.eq("if") => {
                        let condition = eval(list[1].clone(), env.clone());

                        return match condition {
                            MalType::Nil | MalType::False => {
                                // check if there is an "else" clause
                                if list.len() < 4 {
                                    MalType::Nil
                                } else {
                                    eval(list[3].clone(), env.clone())
                                }
                            }
                            _ => eval(list[2].clone(), env.clone()),
                        };
                    }
                    MalType::Symbol(fn_symbol) if fn_symbol.eq("fn*") => {
                        return MalType::MalFunc {
                            params: Box::new(list[1].clone()),
                            body: Box::new(list[2].clone()),
                            env: Some(env.clone()),
                            eval: crate::eval,
                        };
                    }
                    _ => {
                        // new list as a result of calling eval on each member
                        let mut evaled_list = Vec::new();

                        for item in list {
                            evaled_list.push(eval_ast(item, &env));
                        }

                        // println!("evaluated list is {evaled_list:?}");
                        match &evaled_list[0] {
                            MalType::Func(func) => {
                                let args = evaled_list.iter().skip(1).cloned().collect::<Vec<_>>();
                                return (func)(args);
                            }
                            MalType::MalFunc {
                                params,
                                body,
                                env: func_env,
                                ..
                            } => {
                                let args = evaled_list.iter().skip(1).cloned().collect::<Vec<_>>();
                                ast = body.deref().clone();
                                let new_env =
                                    env_bind(func_env.clone(), params.deref().clone(), args);
                                env = new_env;
                                // f.apply(args)
                            }
                            _ => return MalType::Nil,
                        };
                    }
                }
            }
            _ => return eval_ast(ast, &env),
        }
    }
}

fn apply(list: Vec<MalType>, env: &Env) -> MalType {
    let first_elem = &list[0];

    match first_elem {
        MalType::Symbol(define_symbol) if define_symbol.eq("def!") => {
            let v = eval_ast(list[2].clone(), env);

            if matches!(v, MalType::Nil) {
                return MalType::Nil;
            }

            env_set(env, &list[1], v.clone());
            v
        }
        MalType::Symbol(let_symbol) if let_symbol.eq("let*") => {
            let new_env = env_bind(Some(env.clone()), list[1].clone(), list[2..].to_vec());

            let new_bindings = match &list[1] {
                MalType::List(l) => l,
                MalType::Vector(v) => v,
                _ => {
                    println!(
                        "ERROR: first element `{}` in let* binding is not a List/Vector",
                        print_string(&list[1], true)
                    );
                    return MalType::Nil;
                }
            };

            for [s, v] in new_bindings.iter().array_chunks::<2>() {
                let new_value = eval(v.clone(), new_env.clone());
                env_set(&new_env, s, new_value);
            }

            eval(list[2].clone(), new_env)
        }
        MalType::Symbol(do_symbol) if do_symbol.eq("do") => {
            let mut last_item = MalType::Nil;

            for item in list.iter().skip(1) {
                last_item = eval(item.clone(), env.clone());
            }

            last_item
        }
        MalType::Symbol(if_symbol) if if_symbol.eq("if") => {
            let condition = eval(list[1].clone(), env.clone());

            match condition {
                MalType::Nil | MalType::False => {
                    // check if there is an "else" clause
                    if list.len() < 4 {
                        MalType::Nil
                    } else {
                        eval(list[3].clone(), env.clone())
                    }
                }
                _ => eval(list[2].clone(), env.clone()),
            }
        }
        MalType::Symbol(fn_symbol) if fn_symbol.eq("fn*") => MalType::MalFunc {
            params: Box::new(list[1].clone()),
            body: Box::new(list[2].clone()),
            env: Some(env.clone()),
            eval: crate::eval,
        },
        _ => {
            // new list as a result of calling eval on each member
            let mut evaled_list = Vec::new();

            for item in list {
                evaled_list.push(eval_ast(item, env));
            }

            // println!("evaluated list is {evaled_list:?}");
            match &evaled_list[0] {
                MalType::Func(func) => {
                    let args = evaled_list.iter().skip(1).cloned().collect::<Vec<_>>();
                    (func)(args)
                }
                f @ MalType::MalFunc { .. } => {
                    let args = evaled_list.iter().skip(1).cloned().collect::<Vec<_>>();
                    f.apply(args)
                }
                _ => MalType::Nil,
            }
        }
    }
}

fn eval_ast(ast: MalType, env: &Env) -> MalType {
    match ast {
        MalType::Symbol(s) => {
            // lookup symbol and return value or raise error
            if let Some(item) = env_get(env, s.as_str()) {
                item
            } else {
                // println!("{s} not found.");
                // MalType::Nil
                MalType::Symbol(s)
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
        MalType::List(_) => eval(ast, env.clone()),
        _ => ast,
    }
}

fn print(typ: MalType) -> String {
    print_string(&typ, true)
}

fn rep(line: &str, env: &Env) -> Result<String, ReadlineError> {
    let ast = read(line);
    let result = eval(ast, env.clone());
    Ok(print(result))
}

pub fn main() {
    let mut rl = DefaultEditor::new().unwrap(); // TODO(mhs): remove unwrap
    let _ = rl.load_history(".mal-history");
    let repl_env = core_env();

    // declare "not"
    let _ = rep("(def! not (fn* (a) (if a false true)))", &repl_env);

    loop {
        let mut line = String::new();

        while line.is_empty() {
            let Ok(res) = rl.readline("user> ") else {
                return;
            };
            line = res;
            rl.add_history_entry(&line).unwrap(); // TODO(mhs): remove unwrap
            rl.save_history(".mal-history").unwrap(); // TODO(mhs): remove unwrap
        }

        match rep(line.as_str(), &repl_env) {
            Ok(line) => println!("{line}"),
            Err(ReadlineError::Interrupted) => continue,
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
}
