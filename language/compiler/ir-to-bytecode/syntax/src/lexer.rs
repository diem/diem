// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::syntax::ParseError;
use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tok {
    EOF,
    AccountAddressValue,
    U64Value,
    NameValue,
    NameBeginTyValue,
    DotNameValue,
    ByteArrayValue,
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
    Semicolon,
    Less,
    LessEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Caret,
    Underscore,
    /// Abort statement in the Move language
    Abort,
    /// Aborts if in the spec language
    AbortsIf,
    Acquires,
    Address,
    As,
    Assert,
    Bool,
    BorrowGlobal,
    BorrowGlobalMut,
    Break,
    Bytearray,
    Continue,
    Copy,
    Else,
    Ensures,
    Exists,
    False,
    Freeze,
    /// Function to get transaction sender in the Move language
    GetTxnSender,
    /// Like borrow_global, but for spec language
    Global,
    /// Like exists, but for spec language
    GlobalExists,
    If,
    Import,
    Let,
    Loop,
    Main,
    Module,
    Modules,
    Move,
    MoveFrom,
    MoveToSender,
    Native,
    Old,
    Public,
    Requires,
    Resource,
    Return,
    Script,
    Struct,
    SucceedsIf,
    True,
    /// Transaction sender in the specification language
    TxnSender,
    U64,
    Unrestricted,
    While,
    LBrace,
    Pipe,
    PipePipe,
    RBrace,
}

impl Tok {
    /// Return true if the given token is the beginning of a specification directive for the Move
    /// prover
    pub fn is_spec_directive(&self) -> bool {
        match self {
            Tok::Ensures | Tok::Requires | Tok::SucceedsIf | Tok::AbortsIf => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Token<'input>(pub Tok, pub &'input str);

impl<'a> fmt::Display for Token<'a> {
    fn fmt<'f>(&self, formatter: &mut fmt::Formatter<'f>) -> Result<(), fmt::Error> {
        fmt::Display::fmt(self.1, formatter)
    }
}

pub struct Lexer<'input> {
    text: &'input str,
    consumed: usize,
    previous_end: usize,
    token: (usize, Token<'input>, usize),
}

impl<'input> Lexer<'input> {
    pub fn new(s: &'input str) -> Lexer {
        Lexer {
            text: s,
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

    pub fn start_loc(&self) -> usize {
        self.token.0
    }

    pub fn previous_end_loc(&self) -> usize {
        self.previous_end
    }

    pub fn lookahead(&self) -> Result<Tok, ParseError<usize, anyhow::Error>> {
        let text = self.text.trim_start();
        let whitespace = self.text.len() - text.len();
        let start_offset = self.consumed + whitespace;
        let (tok, _) = find_token(text, start_offset)?;
        Ok(tok)
    }

    pub fn advance(&mut self) -> Result<(), ParseError<usize, anyhow::Error>> {
        self.previous_end = self.token.2;
        let text = self.text.trim_start();
        let whitespace = self.text.len() - text.len();
        let start_offset = self.consumed + whitespace;
        let (tok, len) = find_token(text, start_offset)?;
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
fn find_token(
    text: &str,
    start_offset: usize,
) -> Result<(Tok, usize), ParseError<usize, anyhow::Error>> {
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
                    (Tok::AccountAddressValue, 2 + hex_len)
                }
            } else {
                (Tok::U64Value, get_decimal_digits_len(&text))
            }
        }
        'a'..='z' | 'A'..='Z' | '$' | '_' => {
            let len = get_name_len(&text);
            let name = &text[..len];
            match &text[len..].chars().next() {
                Some('"') => {
                    // Special case for ByteArrayValue: h\"[0-9A-Fa-f]*\"
                    let mut bvlen = 0;
                    if name == "h" && {
                        bvlen = get_byte_array_value_len(&text[(len + 1)..]);
                        bvlen > 0
                    } {
                        (Tok::ByteArrayValue, 2 + bvlen)
                    } else {
                        (get_name_token(name), len)
                    }
                }
                Some('.') => {
                    let len2 = get_name_len(&text[(len + 1)..]);
                    if len2 > 0 {
                        (Tok::DotNameValue, len + 1 + len2)
                    } else {
                        (get_name_token(name), len)
                    }
                }
                Some('<') => match name {
                    "borrow_global" => (Tok::BorrowGlobal, len + 1),
                    "borrow_global_mut" => (Tok::BorrowGlobalMut, len + 1),
                    "global" => (Tok::Global, len + 1),
                    "global_exists" => (Tok::GlobalExists, len + 1),
                    "exists" => (Tok::Exists, len + 1),
                    "move_from" => (Tok::MoveFrom, len + 1),
                    "move_to_sender" => (Tok::MoveToSender, len + 1),
                    _ => (Tok::NameBeginTyValue, len + 1),
                },
                Some('(') => match name {
                    "assert" => (Tok::Assert, len + 1),
                    "copy" => (Tok::Copy, len + 1),
                    "move" => (Tok::Move, len + 1),
                    _ => (get_name_token(name), len),
                },
                Some(':') => match name {
                    "modules" => (Tok::Modules, len + 1),
                    "script" => (Tok::Script, len + 1),
                    _ => (get_name_token(name), len),
                },
                _ => (get_name_token(name), len),
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
        '%' => (Tok::Percent, 1),
        '(' => (Tok::LParen, 1),
        ')' => (Tok::RParen, 1),
        '*' => (Tok::Star, 1),
        '+' => (Tok::Plus, 1),
        ',' => (Tok::Comma, 1),
        '-' => (Tok::Minus, 1),
        '.' => (Tok::Period, 1),
        '/' => (Tok::Slash, 1),
        ':' => (Tok::Colon, 1),
        ';' => (Tok::Semicolon, 1),
        '^' => (Tok::Caret, 1),
        '{' => (Tok::LBrace, 1),
        '}' => (Tok::RBrace, 1),
        _ => {
            return Err(ParseError::InvalidToken {
                location: start_offset,
            });
        }
    };

    Ok((tok, len))
}

// Return the length of the substring matching [a-zA-Z$_][a-zA-Z0-9$_]
fn get_name_len(text: &str) -> usize {
    // If the first character is 0..=9 or EOF, then return a length of 0.
    let first_char = text.chars().next().unwrap_or('0');
    if ('0'..='9').contains(&first_char) {
        return 0;
    }
    text.chars()
        .position(|c| match c {
            'a'..='z' | 'A'..='Z' | '$' | '_' | '0'..='9' => false,
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

// Check for an optional sequence of hex digits following by a double quote, and return
// the length of that string if found. This is used to lex ByteArrayValue tokens after
// seeing the 'h"' prefix.
fn get_byte_array_value_len(text: &str) -> usize {
    let hex_len = get_hex_digits_len(text);
    match &text[hex_len..].chars().next() {
        Some('"') => hex_len + 1,
        _ => 0,
    }
}

fn get_name_token(name: &str) -> Tok {
    match name {
        "_" => Tok::Underscore,
        "abort" => Tok::Abort,
        "aborts_if" => Tok::AbortsIf,
        "acquires" => Tok::Acquires,
        "address" => Tok::Address,
        "as" => Tok::As,
        "bool" => Tok::Bool,
        "break" => Tok::Break,
        "bytearray" => Tok::Bytearray,
        "continue" => Tok::Continue,
        "else" => Tok::Else,
        "ensures" => Tok::Ensures,
        "false" => Tok::False,
        "freeze" => Tok::Freeze,
        "get_txn_sender" => Tok::GetTxnSender,
        "if" => Tok::If,
        "import" => Tok::Import,
        "let" => Tok::Let,
        "loop" => Tok::Loop,
        "main" => Tok::Main,
        "module" => Tok::Module,
        "native" => Tok::Native,
        "old" => Tok::Old,
        "public" => Tok::Public,
        "requires" => Tok::Requires,
        "resource" => Tok::Resource,
        "return" => Tok::Return,
        "struct" => Tok::Struct,
        "succeeds_if" => Tok::SucceedsIf,
        "true" => Tok::True,
        "txn_sender" => Tok::TxnSender,
        "u64" => Tok::U64,
        "unrestricted" => Tok::Unrestricted,
        "while" => Tok::While,
        _ => Tok::NameValue,
    }
}
