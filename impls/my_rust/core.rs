#![allow(clippy::not_unsafe_ptr_arg_deref)]
//! TODO(mhs): Handle exceptions like divide by zero

use std::{cell::RefCell, ops::Deref, rc::Rc};

use super::*;

type FuncTuple = (
    &'static str,
    fn(std::vec::Vec<types::MalType>) -> types::MalType,
);

const NS: [FuncTuple; 27] = [
    ("+", core::add),
    ("-", core::sub),
    ("*", core::mul),
    ("/", core::div),
    ("prn", core::prn),
    ("pr-str", core::pr_str),
    ("str", core::str),
    ("println", core::println),
    ("read-string", core::read_string),
    ("slurp", core::slurp),
    ("list", core::list),
    ("list?", core::is_list),
    ("empty?", core::is_empty),
    ("count", core::count),
    ("=", core::eq),
    ("<", core::lt),
    ("<=", core::lteq),
    (">", core::gt),
    (">=", core::gteq),
    ("atom", core::atom),
    ("atom?", core::is_atom),
    ("deref", core::deref),
    ("reset!", core::reset),
    ("swap!", core::swap),
    ("cons", core::cons),
    ("concat", core::concat),
    ("vec", core::vec),
];

pub fn core_env() -> Env {
    let env = env_new(None);

    for (symbol, func) in NS {
        env_set(
            &env,
            &MalType::Symbol(symbol.to_owned()),
            MalType::Func(func),
        )
    }

    env
}

fn add(args: Vec<MalType>) -> MalType {
    if args.is_empty() {
        return MalType::Nil;
    }

    let mut res = {
        let MalType::Number(arg) = args[0] else {
            return MalType::Nil;
        };

        arg
    };

    for i in 1..args.len() {
        let MalType::Number(arg) = args[i] else {
            return MalType::Nil;
        };

        res += arg;
    }

    MalType::Number(res)
}

fn sub(args: Vec<MalType>) -> MalType {
    if args.is_empty() {
        return MalType::Nil;
    }

    let mut res = {
        let MalType::Number(arg) = args[0] else {
            return MalType::Nil;
        };

        arg
    };

    for i in 1..args.len() {
        let MalType::Number(arg) = args[i] else {
            return MalType::Nil;
        };

        res -= arg;
    }

    MalType::Number(res)
}

fn mul(args: Vec<MalType>) -> MalType {
    if args.is_empty() {
        return MalType::Nil;
    }

    let mut res = {
        let MalType::Number(arg) = args[0] else {
            return MalType::Nil;
        };

        arg
    };

    for i in 1..args.len() {
        let MalType::Number(arg) = args[i] else {
            return MalType::Nil;
        };

        res *= arg;
    }

    MalType::Number(res)
}

fn div(args: Vec<MalType>) -> MalType {
    if args.is_empty() {
        return MalType::Nil;
    }

    let mut res = {
        let MalType::Number(arg) = args[0] else {
            return MalType::Nil;
        };

        arg
    };

    for i in 1..args.len() {
        let MalType::Number(arg) = args[i] else {
            return MalType::Nil;
        };

        res /= arg;
    }

    MalType::Number(res)
}

fn pr_str(args: Vec<MalType>) -> MalType {
    let s = print_seq(&args, true, "", "", " ");
    MalType::String(s)
}

fn str(args: Vec<MalType>) -> MalType {
    let s = print_seq(&args, false, "", "", "");
    MalType::String(s)
}

fn prn(args: Vec<MalType>) -> MalType {
    let s = print_seq(&args, true, "", "", " ");
    println!("{}", s);
    MalType::Nil
}

fn println(args: Vec<MalType>) -> MalType {
    let s = print_seq(&args, false, "", "", " ");
    println!("{}", s);
    MalType::Nil
}

fn read_string(args: Vec<MalType>) -> MalType {
    if args.is_empty() {
        return MalType::Nil;
    }

    match &args[0] {
        MalType::String(s) => reader::read_str(s),
        _ => MalType::Nil,
    }
}

fn slurp(args: Vec<MalType>) -> MalType {
    if args.is_empty() {
        return MalType::Nil;
    }

    match &args[0] {
        MalType::String(s) => {
            if let Ok(file_content) = std::fs::read_to_string(s) {
                MalType::String(file_content)
            } else {
                MalType::Nil
            }
        }
        _ => MalType::Nil,
    }
}

fn list(args: Vec<MalType>) -> MalType {
    MalType::List(args)
}

fn is_list(args: Vec<MalType>) -> MalType {
    if args.is_empty() {
        return MalType::False;
    }

    if matches!(args[0], MalType::List(_)) {
        MalType::True
    } else {
        MalType::False
    }
}

fn is_empty(args: Vec<MalType>) -> MalType {
    if args.is_empty() {
        return MalType::True;
    }

    match args[0] {
        MalType::List(ref c) | MalType::Vector(ref c) => {
            if c.is_empty() {
                MalType::True
            } else {
                MalType::False
            }
        }
        _ => MalType::False,
    }
}

fn count(args: Vec<MalType>) -> MalType {
    if args.is_empty() {
        return MalType::Number(0);
    }

    match args[0] {
        MalType::List(ref c) | MalType::Vector(ref c) => MalType::Number(c.len() as i64),
        _ => MalType::Number(0),
    }
}

fn eq(args: Vec<MalType>) -> MalType {
    if args.len() < 2 {
        return MalType::False;
    }

    match (&args[0], &args[1]) {
        (MalType::List(c0), MalType::List(c1))
        | (MalType::Vector(c0), MalType::Vector(c1))
        | (MalType::List(c0), MalType::Vector(c1))
        | (MalType::Vector(c0), MalType::List(c1))
        | (MalType::Dictionary(c0), MalType::Dictionary(c1)) => {
            if c0.len() != c1.len() {
                return MalType::False;
            }

            for i in 0..c0.len() {
                if let MalType::False = eq([c0[i].clone(), c1[i].clone()].to_vec()) {
                    return MalType::False;
                }
            }

            MalType::True
        }
        (MalType::Number(i0), MalType::Number(i1)) => {
            if i0 == i1 {
                MalType::True
            } else {
                MalType::False
            }
        }
        (MalType::String(s0), MalType::String(s1)) => {
            if s0.eq(s1) {
                MalType::True
            } else {
                MalType::False
            }
        }
        (MalType::Symbol(s0), MalType::Symbol(s1)) => {
            if s0.eq(s1) {
                MalType::True
            } else {
                MalType::False
            }
        }
        (MalType::True, MalType::True) => MalType::True,
        (MalType::False, MalType::False) => MalType::True,
        (MalType::Nil, MalType::Nil) => MalType::True,
        _ => MalType::False,
    }
}

fn lteq(args: Vec<MalType>) -> MalType {
    if args.len() < 2 {
        return MalType::False;
    }

    let (MalType::Number(first), MalType::Number(second)) = (&args[0], &args[1]) else {
        return MalType::False;
    };

    if first <= second {
        MalType::True
    } else {
        MalType::False
    }
}

fn lt(args: Vec<MalType>) -> MalType {
    if args.len() < 2 {
        return MalType::False;
    }

    let (MalType::Number(first), MalType::Number(second)) = (&args[0], &args[1]) else {
        return MalType::False;
    };

    if first < second {
        MalType::True
    } else {
        MalType::False
    }
}

fn gteq(args: Vec<MalType>) -> MalType {
    if args.len() < 2 {
        return MalType::False;
    }

    let (MalType::Number(first), MalType::Number(second)) = (&args[0], &args[1]) else {
        return MalType::False;
    };

    if first >= second {
        MalType::True
    } else {
        MalType::False
    }
}

fn gt(args: Vec<MalType>) -> MalType {
    if args.len() < 2 {
        return MalType::False;
    }

    let (MalType::Number(first), MalType::Number(second)) = (&args[0], &args[1]) else {
        return MalType::False;
    };

    if first > second {
        MalType::True
    } else {
        MalType::False
    }
}

fn atom(args: Vec<MalType>) -> MalType {
    if args.len() != 1 {
        println!(
            "Error: wrong number of arguments provided. Expected 1, got {}",
            args.len()
        );
        return MalType::Nil;
    }

    MalType::Atom(Rc::new(RefCell::new(args[0].clone())))
}

fn is_atom(args: Vec<MalType>) -> MalType {
    if args.len() != 1 {
        println!(
            "Error: wrong number of arguments provided. Expected 1, got {}",
            args.len()
        );
        return MalType::Nil;
    }

    MalType::boolean(matches!(args[0], MalType::Atom(_)))
}

fn deref(args: Vec<MalType>) -> MalType {
    if args.len() != 1 {
        println!(
            "Error: wrong number of arguments provided. Expected 1, got {}",
            args.len()
        );
        return MalType::Nil;
    }

    match &args[0] {
        MalType::Atom(a) => a.deref().borrow().clone(),
        _ => {
            println!(
                "Error: wrong argument type provided. Expected an Atom, got {}",
                MalType::discriminant_name(&args[0])
            );
            MalType::Nil
        }
    }
}

fn reset(args: Vec<MalType>) -> MalType {
    if args.len() != 2 {
        println!(
            "Error: wrong number of arguments provided. Expected 2, got {}",
            args.len()
        );
        return MalType::Nil;
    }

    match &args[0] {
        MalType::Atom(a) => {
            a.deref().replace(args[1].clone());
            args[1].clone()
        }
        _ => {
            println!(
                "Error: wrong argument type provided. Expected an Atom, got {}",
                MalType::discriminant_name(&args[0])
            );
            MalType::Nil
        }
    }
}

// mal is single threaded, but in multithreaded Clojure swap promises atomic changes
fn swap(args: Vec<MalType>) -> MalType {
    if args.len() < 2 {
        println!(
            "Error: wrong number of arguments provided. Expected 2 or more, got {}",
            args.len()
        );
        return MalType::Nil;
    }

    match (&args[0], &args[1]) {
        (MalType::Atom(a), f @ MalType::MalFunc { .. })
        | (MalType::Atom(a), f @ MalType::Func(_)) => {
            let mut func_args = [deref([args[0].clone()].to_vec())].to_vec();
            args.iter()
                .skip(2)
                .for_each(|arg| func_args.push(arg.clone()));

            let new_value = f.apply(func_args);
            a.deref().replace(new_value.clone());

            new_value
        }
        _ => {
            println!(
                "Error: wrong argument type provided. Expected an Atom and a Function, got {} and {}",
                MalType::discriminant_name(&args[0]),
                MalType::discriminant_name(&args[1])
            );
            MalType::Nil
        }
    }
}

fn cons(args: Vec<MalType>) -> MalType {
    if args.len() < 2 {
        println!(
            "Error: wrong number of arguments provided. Expected 2 or more, got {}",
            args.len()
        );
        return MalType::Nil;
    }

    match &args[1] {
        MalType::List(end) | MalType::Vector(end) => {
            MalType::List([[args[0].clone()].to_vec(), end.clone()].concat())
        }
        _ => {
            println!(
                "Error: wrong argument type provided. Expected second argument to be List, got {}",
                MalType::discriminant_name(&args[1])
            );
            MalType::Nil
        }
    }
}

fn concat(args: Vec<MalType>) -> MalType {
    let mut res = Vec::new();

    for (i, arg) in args.into_iter().enumerate() {
        match arg {
            MalType::List(end) | MalType::Vector(end) => res = [res, end].concat(),
            _ => {
                println!(
                    "Error: wrong argument type provided. Expected argument #{} to be List, got {}",
                    i,
                    MalType::discriminant_name(&arg)
                );
                return MalType::Nil;
            }
        };
    }

    MalType::List(res)
}

fn vec(args: Vec<MalType>) -> MalType {
    if args.len() != 1 {
        return MalType::Vector([].to_vec());
    }

    let arg = args[0].clone();

    match &arg {
        MalType::List(v) | MalType::Vector(v) => MalType::Vector(v.to_owned()),
        _ => MalType::Vector([arg].to_vec()),
    }
}
