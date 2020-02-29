// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use codespan::{ByteIndex, Span};
use move_ir_types::location::*;
use std::fmt;

use crate::errors::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tok {
    EOF,
    AddressValue,
    NumValue,
    U8Value,
    U64Value,
    U128Value,
    NameValue,
    Exclaim,
    ExclaimEqual,
    Percent,
    Amp,
    AmpAmp,
    AmpMut,
    LParen,
    RParen,
    LBracket,
    RBracket,
    Star,
    Plus,
    Comma,
    Minus,
    Period,
    PeriodPeriod,
    Slash,
    Colon,
    ColonColon,
    Semicolon,
    Less,
    LessEqual,
    LessLess,
    Equal,
    EqualEqual,
    EqualEqualGreater,
    Greater,
    GreaterEqual,
    GreaterGreater,
    Caret,
    Abort,
    AbortsIf,
    Acquires,
    As,
    Break,
    Continue,
    Copy,
    Copyable,
    Define,
    Else,
    Ensures,
    False,
    If,
    Invariant,
    Let,
    Loop,
    Module,
    Move,
    Native,
    Public,
    Resource,
    Return,
    Spec,
    Struct,
    True,
    Use,
    While,
    LBrace,
    Pipe,
    PipePipe,
    RBrace,
    Fun,
}

impl fmt::Display for Tok {
    fn fmt<'f>(&self, formatter: &mut fmt::Formatter<'f>) -> Result<(), fmt::Error> {
        use Tok::*;
        let s = match *self {
            EOF => "[end-of-file]",
            AddressValue => "[Address]",
            NumValue => "[Num]",
            U8Value => "[U8]",
            U64Value => "[U64]",
            U128Value => "[U128]",
            NameValue => "[Name]",
            Exclaim => "!",
            ExclaimEqual => "!=",
            Percent => "%",
            Amp => "&",
            AmpAmp => "&&",
            AmpMut => "&mut",
            LParen => "(",
            RParen => ")",
            LBracket => "[",
            RBracket => "]",
            Star => "*",
            Plus => "+",
            Comma => ",",
            Minus => "-",
            Period => ".",
            PeriodPeriod => "..",
            Slash => "/",
            Colon => ":",
            ColonColon => "::",
            Semicolon => ";",
            Less => "<",
            LessEqual => "<=",
            LessLess => "<<",
            Equal => "=",
            EqualEqual => "==",
            EqualEqualGreater => "==>",
            Greater => ">",
            GreaterEqual => ">=",
            GreaterGreater => ">>",
            Caret => "^",
            Abort => "abort",
            AbortsIf => "aborts_if",
            Acquires => "acquires",
            As => "as",
            Break => "break",
            Continue => "continue",
            Copy => "copy",
            Copyable => "copyable",
            Define => "define",
            Else => "else",
            Ensures => "ensures",
            False => "false",
            If => "if",
            Invariant => "invariant",
            Let => "let",
            Loop => "loop",
            Module => "module",
            Move => "move",
            Native => "native",
            Public => "public",
            Resource => "resource",
            Return => "return",
            Spec => "spec",
            Struct => "struct",
            True => "true",
            Use => "use",
            While => "while",
            LBrace => "{",
            Pipe => "|",
            PipePipe => "||",
            RBrace => "}",
            Fun => "fun",
        };
        fmt::Display::fmt(s, formatter)
    }
}

pub struct Lexer<'input> {
    text: &'input str,
    file: &'static str,
    prev_end: usize,
    cur_start: usize,
    cur_end: usize,
    token: Tok,
}

impl<'input> Lexer<'input> {
    pub fn new(s: &'input str, f: &'static str) -> Lexer<'input> {
        Lexer {
            text: s,
            file: f,
            prev_end: 0,
            cur_start: 0,
            cur_end: 0,
            token: Tok::EOF,
        }
    }

    pub fn peek(&self) -> Tok {
        self.token
    }

    pub fn content(&self) -> &str {
        &self.text[self.cur_start..self.cur_end]
    }

    pub fn file_name(&self) -> &'static str {
        self.file
    }

    pub fn start_loc(&self) -> usize {
        self.cur_start
    }

    pub fn previous_end_loc(&self) -> usize {
        self.prev_end
    }

    // Look ahead to the next token after the current one and return it without advancing
    // the state of the lexer.
    pub fn lookahead(&self) -> Result<Tok, Error> {
        let text = self.text[self.cur_end..].trim_start();
        let offset = self.text.len() - text.len();
        let (tok, _) = find_token(self.file, text, offset)?;
        Ok(tok)
    }

    // Return the starting offset for the next token after the current one.
    pub fn lookahead_start_loc(&self) -> usize {
        let text = self.text[self.cur_end..].trim_start();
        self.text.len() - text.len()
    }

    pub fn advance(&mut self) -> Result<(), Error> {
        self.prev_end = self.cur_end;
        let text = self.text[self.cur_end..].trim_start();
        self.cur_start = self.text.len() - text.len();
        let (token, len) = find_token(self.file, text, self.cur_start)?;
        self.cur_end = self.cur_start + len;
        self.token = token;
        Ok(())
    }

    // Replace the current token. The lexer will always match the longest token,
    // but sometimes the parser will prefer to replace it with a shorter one,
    // e.g., ">" instead of ">>".
    pub fn replace_token(&mut self, token: Tok, len: usize) {
        self.token = token;
        self.cur_end = self.cur_start + len
    }
}

// Find the next token and its length without changing the state of the lexer.
fn find_token(file: &'static str, text: &str, start_offset: usize) -> Result<(Tok, usize), Error> {
    let c: char = match text.chars().next() {
        Some(next_char) => next_char,
        None => {
            return Ok((Tok::EOF, 0));
        }
    };
    let (tok, len) = match c {
        '0'..='9' => {
            if (text.starts_with("0x") || text.starts_with("0X")) && text.len() > 2 {
                let hex_len = get_hex_digits_len(&text[2..]);
                if hex_len == 0 {
                    // Fall back to treating this as a "0" token.
                    (Tok::NumValue, 1)
                } else {
                    (Tok::AddressValue, 2 + hex_len)
                }
            } else {
                get_decimal_number(&text)
            }
        }
        'A'..='Z' | 'a'..='z' | '_' => {
            let len = get_name_len(&text);
            (get_name_token(&text[..len]), len)
        }
        '&' => {
            if text.starts_with("&mut ") {
                (Tok::AmpMut, 5)
            } else if text.starts_with("&&") {
                (Tok::AmpAmp, 2)
            } else {
                (Tok::Amp, 1)
            }
        }
        '|' => {
            if text.starts_with("||") {
                (Tok::PipePipe, 2)
            } else {
                (Tok::Pipe, 1)
            }
        }
        '=' => {
            if text.starts_with("==>") {
                (Tok::EqualEqualGreater, 3)
            } else if text.starts_with("==") {
                (Tok::EqualEqual, 2)
            } else {
                (Tok::Equal, 1)
            }
        }
        '!' => {
            if text.starts_with("!=") {
                (Tok::ExclaimEqual, 2)
            } else {
                (Tok::Exclaim, 1)
            }
        }
        '<' => {
            if text.starts_with("<=") {
                (Tok::LessEqual, 2)
            } else if text.starts_with("<<") {
                (Tok::LessLess, 2)
            } else {
                (Tok::Less, 1)
            }
        }
        '>' => {
            if text.starts_with(">=") {
                (Tok::GreaterEqual, 2)
            } else if text.starts_with(">>") {
                (Tok::GreaterGreater, 2)
            } else {
                (Tok::Greater, 1)
            }
        }
        ':' => {
            if text.starts_with("::") {
                (Tok::ColonColon, 2)
            } else {
                (Tok::Colon, 1)
            }
        }
        '%' => (Tok::Percent, 1),
        '(' => (Tok::LParen, 1),
        ')' => (Tok::RParen, 1),
        '[' => (Tok::LBracket, 1),
        ']' => (Tok::RBracket, 1),
        '*' => (Tok::Star, 1),
        '+' => (Tok::Plus, 1),
        ',' => (Tok::Comma, 1),
        '-' => (Tok::Minus, 1),
        '.' => {
            if text.starts_with("..") {
                (Tok::PeriodPeriod, 2)
            } else {
                (Tok::Period, 1)
            }
        }
        '/' => (Tok::Slash, 1),
        ';' => (Tok::Semicolon, 1),
        '^' => (Tok::Caret, 1),
        '{' => (Tok::LBrace, 1),
        '}' => (Tok::RBrace, 1),
        _ => {
            let span = Span::new(
                ByteIndex(start_offset as u32),
                ByteIndex(start_offset as u32),
            );
            let loc = Loc::new(file, span);
            return Err(vec![(loc, format!("Invalid character: '{}'", c))]);
        }
    };

    Ok((tok, len))
}

// Return the length of the substring matching [a-zA-Z0-9_]. Note that
// this does not do any special check for whether the first character
// starts with a number, so the caller is responsible for any additional
// checks on the first character.
fn get_name_len(text: &str) -> usize {
    text.chars()
        .position(|c| match c {
            'a'..='z' | 'A'..='Z' | '_' | '0'..='9' => false,
            _ => true,
        })
        .unwrap_or_else(|| text.len())
}

fn get_decimal_number(text: &str) -> (Tok, usize) {
    let len = text
        .chars()
        .position(|c| match c {
            '0'..='9' => false,
            _ => true,
        })
        .unwrap_or_else(|| text.len());
    let rest = &text[len..];
    if rest.starts_with("u8") {
        (Tok::U8Value, len + 2)
    } else if rest.starts_with("u64") {
        (Tok::U64Value, len + 3)
    } else if rest.starts_with("u128") {
        (Tok::U128Value, len + 4)
    } else {
        (Tok::NumValue, len)
    }
}

// Return the length of the substring containing characters in [0-9a-fA-F].
fn get_hex_digits_len(text: &str) -> usize {
    text.chars()
        .position(|c| match c {
            'a'..='f' | 'A'..='F' | '0'..='9' => false,
            _ => true,
        })
        .unwrap_or_else(|| text.len())
}

fn get_name_token(name: &str) -> Tok {
    match name {
        "abort" => Tok::Abort,
        "aborts_if" => Tok::AbortsIf,
        "acquires" => Tok::Acquires,
        "as" => Tok::As,
        "break" => Tok::Break,
        "continue" => Tok::Continue,
        "copy" => Tok::Copy,
        "copyable" => Tok::Copyable,
        "define" => Tok::Define,
        "else" => Tok::Else,
        "ensures" => Tok::Ensures,
        "false" => Tok::False,
        "fun" => Tok::Fun,
        "if" => Tok::If,
        "invariant" => Tok::Invariant,
        "let" => Tok::Let,
        "loop" => Tok::Loop,
        "module" => Tok::Module,
        "move" => Tok::Move,
        "native" => Tok::Native,
        "public" => Tok::Public,
        "resource" => Tok::Resource,
        "return" => Tok::Return,
        "spec" => Tok::Spec,
        "struct" => Tok::Struct,
        "true" => Tok::True,
        "use" => Tok::Use,
        "while" => Tok::While,
        _ => Tok::NameValue,
    }
}
