// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::syntax::ParseError;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tok {
    EOF,
    AccountAddressValue,
    U8Value,
    U64Value,
    U128Value,
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
    ColonEqual,
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
    ToU8,
    ToU64,
    ToU128,
    If,
    Import,
    /// For spec language
    Invariant,
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
    /// Return in the specification language
    SpecReturn,
    /// Return statement in the Move language
    Return,
    Script,
    Struct,
    SucceedsIf,
    Synthetic,
    True,
    /// Transaction sender in the specification language
    TxnSender,
    U8,
    U64,
    U128,
    Unrestricted,
    While,
    LBrace,
    Pipe,
    PipePipe,
    RBrace,
    LSquare,
    RSquare,
    PeriodPeriod,
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

pub struct Lexer<'input> {
    pub spec_mode: bool,
    text: &'input str,
    prev_end: usize,
    cur_start: usize,
    cur_end: usize,
    token: Tok,
}

impl<'input> Lexer<'input> {
    pub fn new(s: &'input str) -> Lexer {
        Lexer {
            spec_mode: false, // read tokens without trailing punctuation during specs.
            text: s,
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

    pub fn start_loc(&self) -> usize {
        self.cur_start
    }

    pub fn previous_end_loc(&self) -> usize {
        self.prev_end
    }

    pub fn lookahead(&self) -> Result<Tok, ParseError<usize, anyhow::Error>> {
        let text = self.text[self.cur_end..].trim_start();
        let offset = self.text.len() - text.len();
        let (tok, _) = find_token(text, offset, self.spec_mode)?;
        Ok(tok)
    }

    pub fn advance(&mut self) -> Result<(), ParseError<usize, anyhow::Error>> {
        self.prev_end = self.cur_end;
        let text = self.text[self.cur_end..].trim_start();
        self.cur_start = self.text.len() - text.len();
        let (token, len) = find_token(text, self.cur_start, self.spec_mode)?;
        self.cur_end = self.cur_start + len;
        self.token = token;
        Ok(())
    }

    pub fn replace_token(
        &mut self,
        token: Tok,
        len: usize,
    ) -> Result<(), ParseError<usize, anyhow::Error>> {
        self.token = token;
        self.cur_end = self.cur_start + len;
        Ok(())
    }
}

// Find the next token and its length without changing the state of the lexer.
fn find_token(
    text: &str,
    start_offset: usize,
    spec_mode: bool,
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
                get_decimal_number(&text)
            }
        }
        'a'..='z' | 'A'..='Z' | '$' | '_' => {
            let len = get_name_len(&text);
            let name = &text[..len];
            if !spec_mode {
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
            } else {
                (get_name_token(name), len) // just return the name in spec_mode
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
        '%' => (Tok::Percent, 1),
        '(' => (Tok::LParen, 1),
        ')' => (Tok::RParen, 1),
        '*' => (Tok::Star, 1),
        '+' => (Tok::Plus, 1),
        ',' => (Tok::Comma, 1),
        '-' => (Tok::Minus, 1),
        '.' => {
            if text.starts_with("..") {
                (Tok::PeriodPeriod, 2) // range, for specs
            } else {
                (Tok::Period, 1)
            }
        }
        '/' => (Tok::Slash, 1),
        ':' => {
            if text.starts_with(":=") {
                (Tok::ColonEqual, 2) // spec update
            } else {
                (Tok::Colon, 1)
            }
        }
        ';' => (Tok::Semicolon, 1),
        '^' => (Tok::Caret, 1),
        '{' => (Tok::LBrace, 1),
        '}' => (Tok::RBrace, 1),
        '[' => (Tok::LSquare, 1), // for vector specs
        ']' => (Tok::RSquare, 1), // for vector specs
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
        (Tok::U64Value, len)
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
        "global" => Tok::Global,              // spec language
        "global_exists" => Tok::GlobalExists, // spec language
        "to_u8" => Tok::ToU8,
        "to_u64" => Tok::ToU64,
        "to_u128" => Tok::ToU128,
        "if" => Tok::If,
        "import" => Tok::Import,
        "let" => Tok::Let,
        "loop" => Tok::Loop,
        "main" => Tok::Main,
        "module" => Tok::Module,
        "native" => Tok::Native,
        "invariant" => Tok::Invariant,
        "old" => Tok::Old,
        "public" => Tok::Public,
        "requires" => Tok::Requires,
        "resource" => Tok::Resource,
        "RET" => Tok::SpecReturn,
        "return" => Tok::Return,
        "struct" => Tok::Struct,
        "succeeds_if" => Tok::SucceedsIf,
        "synthetic" => Tok::Synthetic,
        "true" => Tok::True,
        "txn_sender" => Tok::TxnSender,
        "u8" => Tok::U8,
        "u64" => Tok::U64,
        "u128" => Tok::U128,
        "unrestricted" => Tok::Unrestricted,
        "while" => Tok::While,
        _ => Tok::NameValue,
    }
}
