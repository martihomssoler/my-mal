use std::collections::VecDeque;

use crate::{print, MalType, Operator, Token, TokenKind};

pub fn read_str(source: &str) -> MalType {
    let mut tokens = tokenize(source);
    assert!(tokens.back().is_some_and(|t| t.kind == TokenKind::EOF));

    read_form(&mut tokens)
}

fn read_form(tokens: &mut VecDeque<Token>) -> MalType {
    match tokens.front().unwrap().kind {
        TokenKind::LeftParenthesis => {
            tokens.pop_front();
            MalType::List(read_collection(tokens, ")"))
        }
        TokenKind::LeftBracket => {
            tokens.pop_front();
            MalType::Vector(read_collection(tokens, "]"))
        }
        TokenKind::LeftBrace => {
            tokens.pop_front();
            let dict = read_collection(tokens, "}");
            assert_eq!(dict.len() % 2, 0);
            MalType::Dictionary(dict)
        }
        TokenKind::Quote => {
            tokens.pop_front();
            MalType::Quote(Box::new(read_form(tokens)))
        }
        TokenKind::SpliceUnquote => {
            tokens.pop_front();
            MalType::SpliceUnquote(Box::new(read_form(tokens)))
        }
        TokenKind::Quasiquote => {
            tokens.pop_front();
            MalType::Quasiquote(Box::new(read_form(tokens)))
        }
        TokenKind::Unquote => {
            tokens.pop_front();
            MalType::Unqoute(Box::new(read_form(tokens)))
        }
        TokenKind::Deref => {
            tokens.pop_front();
            MalType::List([MalType::Symbol("deref".to_owned()), read_form(tokens)].to_vec())
        }
        TokenKind::WithMeta => {
            tokens.pop_front();
            MalType::WithMeta(Box::new(read_form(tokens)), Box::new(read_form(tokens)))
        }
        _ => read_atom(tokens),
    }
}

fn read_collection(tokens: &mut VecDeque<Token>, end_token: &str) -> Vec<MalType> {
    let mut collection = Vec::new();

    loop {
        match read_form(tokens) {
            MalType::Symbol(s) if s.eq("EOF") || s.eq(end_token) => break,
            mt => collection.push(mt),
        }
    }

    collection
}

fn read_atom(tokens: &mut VecDeque<Token>) -> MalType {
    match tokens.pop_front() {
        Some(token) => match token.kind {
            TokenKind::Number(n) => MalType::Number(n),
            TokenKind::Identifier(id) if id.eq("true") => MalType::True,
            TokenKind::Identifier(id) if id.eq("false") => MalType::False,
            TokenKind::Identifier(id) if id.eq("nil") => MalType::Nil,
            TokenKind::Identifier(id) => MalType::Symbol(id),
            TokenKind::String(s) => MalType::String(s),
            TokenKind::EOF => MalType::Symbol("EOF".to_string()),
            _ => MalType::Symbol(token.kind.to_string()),
        },
        None => todo!(),
    }
}

fn tokenize(source: &str) -> VecDeque<Token> {
    let mut iter = source.chars().peekable();

    let mut line = 1;
    let mut col = 0;

    let mut tokens: VecDeque<Token> = VecDeque::new();
    loop {
        let Some(c) = iter.next() else {
            break;
        };
        col += 1;

        let kind = match c {
            ' ' | ',' => continue,
            '\t' => {
                // TODO(mhs): a tab counts as 4 columns, for now
                col += 3;
                continue;
            }
            '\n' => {
                col = 0;
                line += 1;
                continue;
            }
            '-' => {
                if iter.peek().is_some_and(|nt| nt.is_ascii_digit()) {
                    let c = iter.next().unwrap();
                    let number = parse_number(c, &mut iter, &mut col);
                    let value = number.parse::<i64>().unwrap();
                    TokenKind::Number(-value)
                } else if iter.peek().is_some_and(|nt| !nt.is_whitespace()) {
                    let id = parse_identifier(c, &mut iter, &mut col);
                    TokenKind::Identifier(id)
                } else {
                    TokenKind::Operator(Operator::Minus)
                }
            }
            '+' => TokenKind::Operator(Operator::Plus),
            '*' => {
                if iter.peek().is_some_and(|nt| '*'.eq(nt)) {
                    let _ = iter.next().unwrap();
                    TokenKind::Operator(Operator::DoubleStar)
                } else if iter.peek().is_some_and(|nt| ' '.eq(nt)) {
                    TokenKind::Operator(Operator::Star)
                } else {
                    parse_symbol('*', &mut iter, &mut col)
                }
            }
            '/' => TokenKind::Operator(Operator::Slash),
            '"' => parse_string(c, &mut iter, &mut col),
            ';' => {
                for nt in iter.by_ref() {
                    if '\n'.eq(&nt) {
                        break;
                    }
                }
                col = 0;
                line += 1;
                continue;
            }
            '(' => TokenKind::LeftParenthesis,
            ')' => TokenKind::RightParenthesis,
            '[' => TokenKind::LeftBracket,
            ']' => TokenKind::RightBracket,
            '{' => TokenKind::LeftBrace,
            '}' => TokenKind::RightBrace,
            '\'' => TokenKind::Quote,
            '`' => TokenKind::Quasiquote,
            '^' => TokenKind::WithMeta,
            '@' => TokenKind::Deref,
            '~' => {
                if iter.peek().is_some_and(|nt| '@'.eq(nt)) {
                    iter.next();
                    TokenKind::SpliceUnquote
                } else {
                    TokenKind::Unquote
                }
            }
            d if d.is_ascii_digit() => {
                let number = parse_number(c, &mut iter, &mut col);
                let value = number.parse::<i64>().unwrap();
                TokenKind::Number(value)
            }
            c => parse_symbol(c, &mut iter, &mut col),
        };

        let token = Token { kind, line, col };
        tokens.push_back(token);
    }

    let token = Token {
        kind: TokenKind::EOF,
        line,
        col,
    };
    tokens.push_back(token);

    tokens
}

fn parse_string(
    _c: char,
    iter: &mut std::iter::Peekable<std::str::Chars<'_>>,
    col: &mut usize,
) -> TokenKind {
    let mut can_escape = false;
    let mut id = String::new();
    while let Some(c) = iter.peek() {
        // replace("\\\\", "\\")
        // replace("\\\"", "\"")
        // replace("\\n", "\n")
        let ch = if can_escape {
            match c {
                '\\' => {
                    id.pop();
                    '\\'
                }
                'n' => {
                    id.pop();
                    '\n'
                }
                '\"' => {
                    id.pop();
                    '\"'
                }
                _ => *c,
            }
        } else {
            *c
        };

        let is_str_ending = !can_escape && c.eq(&'"');
        can_escape = !can_escape && c.eq(&'\\');
        iter.next();
        *col += 1;

        if is_str_ending {
            return TokenKind::String(id);
        }

        id.push(ch);
    }
    TokenKind::EOF
}

fn parse_symbol(
    c: char,
    iter: &mut std::iter::Peekable<std::str::Chars<'_>>,
    col: &mut usize,
) -> TokenKind {
    let mut id = c.to_string();
    while let Some(c) = iter.peek() {
        if is_char_symbol_separator(c) {
            break;
        }
        id.push(*c);
        iter.next();
        *col += 1;
    }
    TokenKind::Identifier(id)
}

fn parse_identifier(
    c: char,
    iter: &mut std::iter::Peekable<std::str::Chars<'_>>,
    col: &mut usize,
) -> String {
    let mut id = c.to_string();
    while let Some(c) = iter.peek() {
        if !(c.is_ascii_alphanumeric() || c.eq(&'_') || c.eq(&'-') || c.eq(&'>')) {
            break;
        }

        id.push(*c);
        iter.next();
        *col += 1;
    }
    id
}

fn parse_number(
    c: char,
    iter: &mut std::iter::Peekable<std::str::Chars<'_>>,
    col: &mut usize,
) -> String {
    let mut number = c.to_string();
    while let Some(c) = iter.peek() {
        if !c.is_ascii_digit() {
            break;
        }
        number.push(*c);
        iter.next();
        *col += 1;
    }
    number
}

fn is_char_symbol_separator(c: &char) -> bool {
    c.eq(&'(')
        || c.eq(&'[')
        || c.eq(&'{')
        || c.eq(&')')
        || c.eq(&']')
        || c.eq(&'}')
        || c.eq(&' ')
        || c.eq(&'\n')
        || c.eq(&'\t')
}
