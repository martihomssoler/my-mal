use std::collections::HashMap;

pub type FuncPtr<T> = fn(usize, *const T) -> T;

fn apply(func: impl Fn(i64, i64) -> i64, n: usize, args: *const i64) -> i64 {
    let mut res = unsafe { *args.add(0) };

    (1..n).for_each(|i| {
        let arg = unsafe { *args.add(i) };
        res = func(res, arg);
    });

    res
}

pub struct ReplEnv {
    pub symbols: HashMap<&'static str, FuncPtr<i64>>,
}

impl Default for ReplEnv {
    fn default() -> Self {
        let mut symbols = HashMap::new();

        symbols.insert("+", (|n, args| apply(|a, b| a + b, n, args)) as _);
        symbols.insert("-", (|n, args| apply(|a, b| a - b, n, args)) as _);
        symbols.insert("*", (|n, args| apply(|a, b| a * b, n, args)) as _);
        symbols.insert("/", (|n, args| apply(|a, b| a / b, n, args)) as _);

        Self { symbols }
    }
}
