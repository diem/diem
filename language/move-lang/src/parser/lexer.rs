// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use codespan::{ByteIndex, Span};
use std::fmt;

use crate::errors::*;
use crate::shared::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tok {
    EOF,
    AddressValue,
    U64Value,
    NameValue,
    NameBeginArgsValue,
    Exclaim,
    ExclaimEqual,
    Percent,
    Amp,
    AmpAmp,
    AmpMut,
    LParen,
    RParen,
    Star,
    Plus,
    Comma,
    Minus,
    Period,
    Slash,
    Colon,
    ColonColon,
    Semicolon,
    Less,
    LessEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Caret,
    Abort,
    Acquires,
    As,
    Break,
    Continue,
    Copy,
    Copyable,
    Else,
    False,
    If,
    Let,
    Loop,
    Module,
    Move,
    Native,
    Public,
    Resource,
    Return,
    Struct,
    True,
    Use,
    While,
    LBrace,
    Pipe,
    PipePipe,
    RBrace,
}

impl fmt::Display for Tok {
    fn fmt<'f>(&self, formatter: &mut fmt::Formatter<'f>) -> Result<(), fmt::Error> {
        use Tok::*;
        let s = match *self {
            EOF => "[end-of-file]",
            AddressValue => "[Address]",
            U64Value => "[U64]",
            NameValue => "[Name]",
            NameBeginArgsValue => "[Name<Types>]",
            Exclaim => "!",
            ExclaimEqual => "!=",
            Percent => "%",
            Amp => "&",
            AmpAmp => "&&",
            AmpMut => "&mut",
            LParen => "(",
            RParen => ")",
            Star => "*",
            Plus => "+",
            Comma => ",",
            Minus => "-",
            Period => ".",
            Slash => "/",
            Colon => ":",
            ColonColon => "::",
            Semicolon => ";",
            Less => "<",
            LessEqual => "<=",
            Equal => "=",
            EqualEqual => "==",
            Greater => ">",
            GreaterEqual => ">=",
            Caret => "^",
            Abort => "abort",
            Acquires => "acquires",
            As => "as",
            Break => "break",
            Continue => "continue",
            Copy => "copy",
            Copyable => "copyable",
            Else => "else",
            False => "false",
            If => "if",
            Let => "let",
            Loop => "loop",
            Module => "module",
            Move => "move",
            Native => "native",
            Public => "public",
            Resource => "resource",
            Return => "return",
            Struct => "struct",
            True => "true",
            Use => "use",
            While => "while",
            LBrace => "{",
            Pipe => "|",
            PipePipe => "||",
            RBrace => "}",
        };
        fmt::Display::fmt(s, formatter)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Token<'input>(pub Tok, pub &'input str);

impl<'a> fmt::Display for Token<'a> {
    fn fmt<'f>(&self, formatter: &mut fmt::Formatter<'f>) -> Result<(), fmt::Error> {
        match self.0 {
            Tok::AddressValue | Tok::U64Value | Tok::NameValue | Tok::NameBeginArgsValue => {
                fmt::Display::fmt(self.1, formatter)
            }
            _ => fmt::Display::fmt(&self.0.to_string(), formatter),
        }
    }
}

pub struct Lexer<'input> {
    text: &'input str,
    file: &'static str,
    consumed: usize,
    previous_end: usize,
    token: (usize, Token<'input>, usize),
}

impl<'input> Lexer<'input> {
    pub fn new(s: &'input str, f: &'static str) -> Lexer<'input> {
        Lexer {
            text: s,
            file: f,
            consumed: 0,
            previous_end: 0,
            token: (0, Token(Tok::EOF, ""), 0),
        }
    }

    pub fn peek(&self) -> Tok {
        (self.token.1).0
    }

    pub fn content(&self) -> &str {
        (self.token.1).1
    }

    pub fn file_name(&self) -> &'static str {
        self.file
    }

    pub fn start_loc(&self) -> usize {
        self.token.0
    }

    pub fn previous_end_loc(&self) -> usize {
        self.previous_end
    }

    pub fn lookahead(&self) -> Result<Tok, Error> {
        let text = self.text.trim_start();
        let whitespace = self.text.len() - text.len();
        let start_offset = self.consumed + whitespace;
        let (tok, _) = find_token(self.file, text, start_offset)?;
        Ok(tok)
    }

    pub fn advance(&mut self) -> Result<(), Error> {
        self.previous_end = self.token.2;
        let text = self.text.trim_start();
        let whitespace = self.text.len() - text.len();
        let start_offset = self.consumed + whitespace;
        let (tok, len) = find_token(self.file, text, start_offset)?;
        let result = &text[..len];
        let remaining = &text[len..];
        let end_offset = start_offset + len;
        self.text = remaining;
        self.consumed = end_offset;
        self.token = (start_offset, Token(tok, result), end_offset);
        Ok(())
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
                    (Tok::U64Value, 1)
                } else {
                    (Tok::AddressValue, 2 + hex_len)
                }
            } else {
                (Tok::U64Value, get_decimal_digits_len(&text))
            }
        }
        'A'..='Z' | 'a'..='z' | '_' => {
            let len = get_name_len(&text);
            match &text[len..].chars().next() {
                Some('<') => (Tok::NameBeginArgsValue, len + 1),
                _ => (get_name_token(&text[..len]), len),
            }
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
            if text.starts_with("==") {
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
            } else {
                (Tok::Less, 1)
            }
        }
        '>' => {
            if text.starts_with(">=") {
                (Tok::GreaterEqual, 2)
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
        '*' => (Tok::Star, 1),
        '+' => (Tok::Plus, 1),
        ',' => (Tok::Comma, 1),
        '-' => (Tok::Minus, 1),
        '.' => (Tok::Period, 1),
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
            return Err(vec![(loc, "Invalid token".into())]);
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

// Return the length of the substring containing characters in [0-9].
fn get_decimal_digits_len(text: &str) -> usize {
    text.chars()
        .position(|c| match c {
            '0'..='9' => false,
            _ => true,
        })
        .unwrap_or_else(|| text.len())
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
        "acquires" => Tok::Acquires,
        "as" => Tok::As,
        "break" => Tok::Break,
        "continue" => Tok::Continue,
        "copy" => Tok::Copy,
        "copyable" => Tok::Copyable,
        "else" => Tok::Else,
        "false" => Tok::False,
        "if" => Tok::If,
        "let" => Tok::Let,
        "loop" => Tok::Loop,
        "module" => Tok::Module,
        "move" => Tok::Move,
        "native" => Tok::Native,
        "public" => Tok::Public,
        "resource" => Tok::Resource,
        "return" => Tok::Return,
        "struct" => Tok::Struct,
        "true" => Tok::True,
        "use" => Tok::Use,
        "while" => Tok::While,
        _ => Tok::NameValue,
    }
}
