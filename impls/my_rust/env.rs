use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{print_string, MalType};

pub type Env = Rc<EnvStruct>;

#[derive(Clone, Debug)]
pub struct EnvStruct {
    data: RefCell<HashMap<String, MalType>>,
    outer: Option<Env>,
}

pub fn env_new(outer: Option<Env>) -> Env {
    Rc::new(EnvStruct {
        data: RefCell::new(HashMap::new()),
        outer,
    })
}

pub fn env_bind(outer: Option<Env>, binds: MalType, exprs: Vec<MalType>) -> Env {
    let env = env_new(outer);

    match binds {
        MalType::List(bs) | MalType::Vector(bs) => {
            let mut bind_count = 0;
            let mut i = 0;

            while i < bs.len() {
                if let MalType::Symbol(s) = &bs[i] {
                    if s.eq("&") {
                        let rest_exprs = MalType::List(exprs[i..].to_vec());
                        env_set(&env, &bs[i + 1], rest_exprs);
                        bind_count += 1;
                        i += 2;
                        continue;
                    }
                }

                if exprs.len() <= bind_count {
                    break;
                }

                env_set(&env, &bs[i], exprs[i].clone());
                bind_count += 1;
                i += 1;
            }
        }
        _ => {
            println!("env_bind binds is not a List/Vector");
        }
    }

    env
}

pub fn env_set(env: &Env, k: &MalType, v: MalType) {
    match k {
        MalType::Symbol(s) => {
            env.data.borrow_mut().insert(s.to_string(), v.clone());
        }
        _ => {
            println!("env_get called with a non-Symbol {}", print_string(k, true));
        }
    }
}

pub fn env_find(env: &Env, k: &str) -> Option<Env> {
    match (env.data.borrow().contains_key(k), env.outer.clone()) {
        (true, _) => Some(env.clone()),
        (false, Some(o)) => env_find(&o, k),
        _ => None,
    }
}

pub fn env_get(env: &Env, k: &str) -> Option<MalType> {
    if let Some(env) = env_find(env, k) {
        env.data.borrow().get(k).cloned()
    } else {
        None
    }
}
