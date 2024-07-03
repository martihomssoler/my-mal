use std::{cell::RefCell, fmt::Display, ops::Deref, rc::Rc};

use crate::{env::*, print_string};

pub type Atom = Rc<RefCell<MalType>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MalType {
    Atom(Atom),
    Dictionary(Vec<MalType>),
    False,
    Func(fn(Vec<MalType>) -> MalType),
    List(Vec<MalType>),
    MalFunc {
        params: Box<MalType>,
        body: Box<MalType>,
        env: Option<Env>,
        eval: fn(ast: MalType, env: Env) -> MalType,
    },
    Nil,
    Number(i64),
    String(String),
    Symbol(String),
    True,
    Vector(Vec<MalType>),
    WithMeta(Box<MalType>, Box<MalType>),
}

unsafe impl Send for MalType {}
unsafe impl Sync for MalType {}

impl MalType {
    pub fn boolean(value: bool) -> MalType {
        if value {
            MalType::True
        } else {
            MalType::False
        }
    }

    pub fn discriminant_name(value: &MalType) -> String {
        match value {
            MalType::Atom(_) => "Atom".to_owned(),
            MalType::Dictionary(_) => "Dictionary".to_owned(),
            MalType::False => "False".to_owned(),
            MalType::Func(_) => "Func".to_owned(),
            MalType::List(_) => "List".to_owned(),
            MalType::MalFunc { .. } => "MalFunc".to_owned(),
            MalType::Nil => "Nil".to_owned(),
            MalType::Number(_) => "Number".to_owned(),
            MalType::String(_) => "String".to_owned(),
            MalType::Symbol(_) => "Symbol".to_owned(),
            MalType::True => "True".to_owned(),
            MalType::Vector(_) => "Vector".to_owned(),
            MalType::WithMeta(_, _) => "WithMeta".to_owned(),
        }
    }

    pub fn apply(&self, args: Vec<MalType>) -> MalType {
        match self {
            MalType::MalFunc {
                params,
                env,
                eval,
                body,
            } => {
                let fn_env = env_bind(env.clone(), params.deref().clone(), args);
                eval(body.deref().clone(), fn_env)
            }
            MalType::Func(f) => (f)(args),
            _ => {
                println!("Trying to call a non-function");
                MalType::Nil
            }
        }
    }
}

impl Display for MalType {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "{}", print_string(self, true))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Operator {
    Minus,
    Plus,
    Star,
    DoubleStar,
    Slash,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TokenKind {
    // Operators
    Operator(Operator),
    // Literals
    Number(i64),
    String(String),
    // Others
    Identifier(String),
    LeftParenthesis,
    RightParenthesis,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Quote,
    SpliceUnquote,
    Quasiquote,
    Unquote,
    WithMeta,
    Deref,
    // EOF
    EOF,
}

impl Display for TokenKind {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Token::Identifier(id) => write!(f, "{id}"),
            TokenKind::EOF => write!(fmt, "EOF"),
            TokenKind::Operator(o) => o.fmt(fmt),
            TokenKind::Identifier(id) => write!(fmt, "'{id}'"),
            TokenKind::Number(n) => write!(fmt, "{n}"),
            TokenKind::LeftParenthesis => write!(fmt, "("),
            TokenKind::RightParenthesis => write!(fmt, ")"),
            TokenKind::LeftBracket => write!(fmt, "["),
            TokenKind::RightBracket => write!(fmt, "]"),
            TokenKind::LeftBrace => write!(fmt, "{{"),
            TokenKind::RightBrace => write!(fmt, "}}"),
            TokenKind::Quote => write!(fmt, "quote"),
            TokenKind::Quasiquote => write!(fmt, "quasiquote"),
            TokenKind::Unquote => write!(fmt, "unquote"),
            TokenKind::WithMeta => write!(fmt, "with-meta"),
            TokenKind::Deref => write!(fmt, "deref"),
            TokenKind::SpliceUnquote => write!(fmt, "spliceunquote"),
            TokenKind::String(s) => write!(fmt, "{s}"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub col: usize,
}

impl Token {
    pub fn location(&self) -> String {
        format!("[ line:{} ; col:{} ]", self.line, self.col)
    }
}

impl Display for Token {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.kind.fmt(fmt)
    }
}

impl Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operator::Minus => write!(f, "-"),
            Operator::Plus => write!(f, "+"),
            Operator::Star => write!(f, "*"),
            Operator::DoubleStar => write!(f, "**"),
            Operator::Slash => write!(f, "/"),
        }
    }
}
