use std::ops::Deref;

use crate::MalType;

pub fn print_string(mal_type: &MalType, print_readably: bool) -> String {
    match mal_type {
        MalType::List(seq) => print_seq(seq, print_readably, "(", ")", " "),
        MalType::Vector(seq) => print_seq(seq, print_readably, "[", "]", " "),
        MalType::Dictionary(seq) => print_seq(seq, print_readably, "{", "}", " "),
        MalType::WithMeta(var, meta) => {
            let var = print_string(var.as_ref(), true);
            let meta = print_string(meta.as_ref(), true);
            format!("(with-meta {meta} {var})")
        }
        MalType::Func(_) | MalType::MalFunc { .. } => "#<function>".to_string(),
        MalType::Nil => "nil".to_string(),
        MalType::True => "true".to_string(),
        MalType::False => "false".to_string(),
        MalType::Symbol(s) => s.to_string(),
        MalType::Number(n) => format!("{n}"),
        MalType::String(s) => {
            if print_readably {
                format!("\"{}\"", escape_str(s))
            } else {
                s.clone()
            }
        }
        MalType::Atom(a) => {
            let atom = print_string(a.deref().borrow().deref(), true);
            format!("(atom {})", atom)
        }
    }
}

fn escape_str(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '"' => "\\\"".to_string(),
            '\n' => "\\n".to_string(),
            '\\' => "\\\\".to_string(),
            _ => c.to_string(),
        })
        .collect::<Vec<String>>()
        .join("")
}

pub fn print_seq(
    seq: &[MalType],
    print_readably: bool,
    prefix: &str,
    postfix: &str,
    join_with: &str,
) -> String {
    let strs: Vec<String> = seq
        .iter()
        .map(|mt| print_string(mt, print_readably))
        .collect();
    format!("{}{}{}", prefix, strs.join(join_with), postfix)
}
