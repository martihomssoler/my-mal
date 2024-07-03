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
use std::ops::Deref;
use types::*;

fn read(line: &str) -> MalType {
    read_str(line)
}

fn quasiquote(ast: &MalType) -> MalType {
    match ast {
        MalType::List(ast_list)
            if !ast_list.is_empty() && ast_list[0] == MalType::Symbol("unquote".to_owned()) =>
        {
            ast_list[1].clone()
        }
        MalType::List(ast_list) => qq_iter(ast_list),
        MalType::Vector(ast_vec) => {
            MalType::List([MalType::Symbol("vec".to_owned()), qq_iter(ast_vec)].to_vec())
        }
        MalType::Dictionary(_) | MalType::Symbol(_) => {
            MalType::List([MalType::Symbol("quote".to_owned()), ast.clone()].to_vec())
        }
        _ => ast.clone(),
    }
}

fn qq_iter(ast_list: &[MalType]) -> MalType {
    let mut res = Vec::new();

    for elem in ast_list.iter().rev() {
        match elem {
            MalType::List(elem_list)
                if !elem_list.is_empty()
                    && elem_list[0] == MalType::Symbol("splice-unquote".to_owned()) =>
            {
                res = [
                    MalType::Symbol("concat".to_owned()),
                    elem_list[1].clone(),
                    MalType::List(res),
                ]
                .to_vec();
            }
            _ => {
                res = [
                    MalType::Symbol("cons".to_owned()),
                    quasiquote(elem),
                    MalType::List(res),
                ]
                .to_vec();
            }
        }
    }

    MalType::List(res)
}

fn eval(mut ast: MalType, mut env: Env) -> MalType {
    loop {
        match ast {
            MalType::List(ref list) if list.is_empty() => return ast,
            MalType::List(ref list) => {
                let first_elem = &list[0];

                match first_elem {
                    MalType::Symbol(quote_symbol) if quote_symbol.eq("quote") => {
                        return list[1].to_owned();
                    }
                    MalType::Symbol(qqexpand_symbol) if qqexpand_symbol.eq("quasiquoteexpand") => {
                        return quasiquote(&list[1]);
                    }
                    MalType::Symbol(quasiquote_symbol) if quasiquote_symbol.eq("quasiquote") => {
                        ast = quasiquote(&list[1]);
                    }
                    MalType::Symbol(eval_symbol) if eval_symbol.eq("eval") => {
                        ast = eval(list[1].clone(), env.clone());
                        while let Some(ref outer_env) = env.clone().outer {
                            env = outer_env.clone();
                        }
                    }
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
                            evaled_list.push(eval_ast(item.clone(), &env));
                        }

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

fn eval_ast(ast: MalType, env: &Env) -> MalType {
    match ast {
        MalType::Symbol(s) => {
            // lookup symbol and return value or raise error
            if let Some(item) = env_get(env, s.as_str()) {
                item
            } else {
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
    let mut args = std::env::args();
    let arg1 = args.nth(1);

    env_set(
        &repl_env,
        &MalType::Symbol("*ARGV*".to_owned()),
        MalType::List(args.map(MalType::String).collect()),
    );

    // defining functions with mal itself
    let _ = rep("(def! not (fn* (a) (if a false true)))", &repl_env);
    let _ = rep(
        "(def! load-file (fn* (f) (eval (read-string (str \"(do \" (slurp f) \"\nnil)\")))))",
        &repl_env,
    );

    if let Some(filename) = arg1 {
        // filename is the first argument, so there is always at least one arg
        let _ = rep(&format!("( load-file {} )", filename), &repl_env);
    }

    // REPL
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
