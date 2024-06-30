#![allow(clippy::not_unsafe_ptr_arg_deref)]
//! TODO(mhs): Handle exceptions like divide by zero

use super::*;

type FuncTuple = (
    &'static str,
    fn(std::vec::Vec<types::MalType>) -> types::MalType,
);

const NS: [FuncTuple; 17] = [
    ("+", core::add),
    ("-", core::sub),
    ("*", core::mul),
    ("/", core::div),
    ("prn", core::prn),
    ("pr-str", core::pr_str),
    ("str", core::str),
    ("println", core::println),
    ("list", core::list),
    ("list?", core::is_list),
    ("empty?", core::is_empty),
    ("count", core::count),
    ("=", core::eq),
    ("<", core::lt),
    ("<=", core::lteq),
    (">", core::gt),
    (">=", core::gteq),
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
