use std::{fmt::Display, ops::Deref};

use crate::env::*;

#[derive(Debug, Clone)]
pub enum MalType {
    // collections
    List(Vec<MalType>),
    Vector(Vec<MalType>),
    Dictionary(Vec<MalType>),
    // primitives
    Number(i64),
    String(String),
    Symbol(String),
    True,
    False,
    Func(fn(Vec<MalType>) -> MalType),
    MalFunc {
        params: Box<MalType>,
        body: Box<MalType>,
        env: Option<Env>,
        eval: fn(ast: MalType, env: Env) -> MalType,
    },
    // Special symbols
    Quote(Box<MalType>),
    SpliceUnquote(Box<MalType>),
    Quasiquote(Box<MalType>),
    Unqoute(Box<MalType>),
    Deref(Box<MalType>),
    WithMeta(Box<MalType>, Box<MalType>),
    Nil,
}

unsafe impl Send for MalType {}
unsafe impl Sync for MalType {}

impl MalType {
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
            _ => {
                println!("Trying to call a non-function");
                MalType::Nil
            }
        }
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
