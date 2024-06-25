use std::{collections::HashMap, ops::Deref};

use crate::MalType;
use functions::*;

#[derive(Clone, Debug)]
pub struct Env {
    data: HashMap<String, MalType>,
    outer: Box<Option<Env>>,
}

impl Default for Env {
    fn default() -> Self {
        let mut env = Env::new(None);

        env.set("+", MalType::Function(&|c, a| -> MalType { add(c, a) }));
        env.set("-", MalType::Function(&|c, a| -> MalType { sub(c, a) }));
        env.set("*", MalType::Function(&|c, a| -> MalType { mul(c, a) }));
        env.set("/", MalType::Function(&|c, a| -> MalType { div(c, a) }));

        env
    }
}

impl Env {
    pub fn new(outer: Option<Env>) -> Env {
        Self {
            data: HashMap::new(),
            outer: Box::new(outer),
        }
    }

    pub fn set(&mut self, k: &str, v: MalType) {
        self.data.insert(k.to_owned(), v);
    }

    pub fn find(&self, k: &str) -> Option<&Env> {
        if self.data.contains_key(k) {
            Some(self)
        } else if let Some(outer) = self.outer.deref() {
            outer.find(k)
        } else {
            None
        }
    }

    pub fn get(&self, k: &str) -> Option<&MalType> {
        if let Some(env) = self.find(k) {
            env.data.get(k)
        } else {
            None
        }
    }
}

// TODO(mhs): Handle exceptions like divide by zero
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub mod functions {
    use super::*;

    pub fn add(argc: usize, argv: *const MalType) -> MalType {
        if argc == 0 {
            return MalType::None;
        }

        let mut res = unsafe {
            let MalType::Number(arg) = *argv.add(0) else {
                return MalType::None;
            };

            arg
        };

        for i in 1..argc {
            unsafe {
                let MalType::Number(arg) = *argv.add(i) else {
                    return MalType::None;
                };

                res += arg;
            };
        }

        MalType::Number(res)
    }

    pub fn sub(argc: usize, argv: *const MalType) -> MalType {
        if argc == 0 {
            return MalType::None;
        }

        let mut res = unsafe {
            let MalType::Number(arg) = *argv.add(0) else {
                return MalType::None;
            };

            arg
        };

        for i in 1..argc {
            unsafe {
                let MalType::Number(arg) = *argv.add(i) else {
                    return MalType::None;
                };

                res -= arg;
            };
        }

        MalType::Number(res)
    }

    pub fn mul(argc: usize, argv: *const MalType) -> MalType {
        if argc == 0 {
            return MalType::None;
        }

        let mut res = unsafe {
            let MalType::Number(arg) = *argv.add(0) else {
                return MalType::None;
            };

            arg
        };

        for i in 1..argc {
            unsafe {
                let MalType::Number(arg) = *argv.add(i) else {
                    return MalType::None;
                };

                res *= arg;
            };
        }

        MalType::Number(res)
    }

    pub fn div(argc: usize, argv: *const MalType) -> MalType {
        if argc == 0 {
            return MalType::None;
        }

        let mut res = unsafe {
            let MalType::Number(arg) = *argv.add(0) else {
                return MalType::None;
            };

            arg
        };

        for i in 1..argc {
            unsafe {
                let MalType::Number(arg) = *argv.add(i) else {
                    return MalType::None;
                };

                res /= arg;
            };
        }

        MalType::Number(res)
    }
}
