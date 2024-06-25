use std::{any::Any, ffi::c_void, fmt::Display};

#[derive(Debug, Clone)]
pub enum MalType {
    // collections
    List(Vec<MalType>),
    Vector(Vec<MalType>),
    Dictionary(Vec<MalType>),
    // primitives
    Number(i64),
    Symbol(String),
    Function(*const dyn Fn(usize, *const MalType) -> MalType),
    // Special symbols
    Quote(Box<MalType>),
    SpliceUnquote(Box<MalType>),
    Quasiquote(Box<MalType>),
    Unqoute(Box<MalType>),
    Deref(Box<MalType>),
    WithMeta(Box<MalType>, Box<MalType>),
    None,
}

unsafe impl Send for MalType {}
unsafe impl Sync for MalType {}

impl Display for MalType {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MalType::List(list) => {
                let mut res = String::new();

                for (i, item) in list.iter().enumerate() {
                    if i != 0 {
                        res.push(' ');
                    }
                    res.push_str(format!("{item}").as_str());
                }

                write!(fmt, "({res})")
            }
            MalType::Vector(vector) => {
                let mut res = String::new();

                for (i, item) in vector.iter().enumerate() {
                    if i != 0 {
                        res.push(' ');
                    }
                    res.push_str(format!("{item}").as_str());
                }

                write!(fmt, "[{res}]")
            }
            MalType::Dictionary(dict) => {
                let mut res = String::new();

                for (i, item) in dict.iter().enumerate() {
                    if i != 0 {
                        res.push(' ');
                    }
                    res.push_str(format!("{item}").as_str());
                }

                write!(fmt, "{{{res}}}")
            }
            MalType::Number(n) => write!(fmt, "{n}"),
            MalType::Symbol(s) => write!(fmt, "{s}"),
            MalType::Quote(quote) => {
                let res = quote.as_ref().to_string();
                write!(fmt, "(quote {res})")
            }
            MalType::SpliceUnquote(splice_unquote) => {
                let res = splice_unquote.as_ref().to_string();
                write!(fmt, "(splice-unquote {res})")
            }
            MalType::Quasiquote(quasiquote) => {
                let res = quasiquote.as_ref().to_string();
                write!(fmt, "(quasiquote {res})")
            }
            MalType::Unqoute(unquote) => {
                let res = unquote.as_ref().to_string();
                write!(fmt, "(unquote {res})")
            }
            MalType::Deref(var) => {
                let res = var.as_ref().to_string();
                write!(fmt, "(deref {res})")
            }
            MalType::WithMeta(var, meta) => {
                let var = var.as_ref().to_string();
                let meta = meta.as_ref().to_string();
                write!(fmt, "(with-meta {meta} {var})")
            }
            MalType::Function(_) => todo!(),
            MalType::None => Ok(()),
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
