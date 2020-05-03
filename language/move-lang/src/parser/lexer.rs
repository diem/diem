// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{errors::*, parser::syntax::make_loc, FileCommentMap, MatchedFileCommentMap};
use codespan::{ByteIndex, Span};
use move_ir_types::location::Loc;
use std::{collections::BTreeMap, fmt};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tok {
    EOF,
    AddressValue,
    NumValue,
    U8Value,
    U64Value,
    U128Value,
    ByteStringValue,
    IdentifierValue,
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
    Acquires,
    As,
    Break,
    Continue,
    Copy,
    Copyable,
    Define,
    Else,
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
    Script,
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
            ByteStringValue => "[ByteString]",
            IdentifierValue => "[Identifier]",
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
            Acquires => "acquires",
            As => "as",
            Break => "break",
            Continue => "continue",
            Copy => "copy",
            Copyable => "copyable",
            Define => "define",
            Else => "else",
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
            Script => "script",
        };
        fmt::Display::fmt(s, formatter)
    }
}

pub struct Lexer<'input> {
    text: &'input str,
    file: &'static str,
    doc_comments: FileCommentMap,
    matched_doc_comments: MatchedFileCommentMap,
    prev_end: usize,
    cur_start: usize,
    cur_end: usize,
    token: Tok,
}

impl<'input> Lexer<'input> {
    pub fn new(
        text: &'input str,
        file: &'static str,
        doc_comments: BTreeMap<Span, String>,
    ) -> Lexer<'input> {
        Lexer {
            text,
            file,
            doc_comments,
            matched_doc_comments: BTreeMap::new(),
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

    // Matches the doc comments after the last token (or the beginning of the file) to the position
    // of the current token. This moves the comments out of `doc_comments` and
    // into `matched_doc_comments`. At the end of parsing, if `doc_comments` is not empty, errors
    // for stale doc comments will be produced.
    //
    // Calling this function during parsing effectively marks a valid point for documentation
    // comments. The documentation comments are not stored in the AST, but can be retrieved by
    // using the start position of an item as an index into `matched_doc_comments`.
    pub fn match_doc_comments(&mut self) {
        let start = self.previous_end_loc() as u32;
        let end = self.cur_start as u32;
        let mut matched = vec![];
        let merged = self
            .doc_comments
            .range(Span::new(start, start)..Span::new(end, end))
            .map(|(span, s)| {
                matched.push(*span);
                s.clone()
            })
            .collect::<Vec<String>>()
            .join("\n");
        for span in matched {
            self.doc_comments.remove(&span);
        }
        self.matched_doc_comments.insert(ByteIndex(end), merged);
    }

    // At the end of parsing, checks whether there are any unmatched documentation comments,
    // producing errors if so. Otherwise returns a map from file position to associated
    // documentation.
    pub fn check_and_get_doc_comments(&mut self) -> Result<MatchedFileCommentMap, Errors> {
        let errors = self
            .doc_comments
            .iter()
            .map(|(span, _)| {
                vec![(
                    Loc::new(self.file, *span),
                    "documentation comment cannot be matched to a language item".to_string(),
                )]
            })
            .collect::<Errors>();
        if errors.is_empty() {
            Ok(std::mem::take(&mut self.matched_doc_comments))
        } else {
            Err(errors)
        }
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
            if text.starts_with("0x") && text.len() > 2 {
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
            if text.starts_with("x\"") {
                // Search the current source line for a closing quote.
                let line = text.lines().next().unwrap();
                let len = line[2..].find('"').unwrap_or_else(|| line.len() - 2);
                if line.len() == 2 || !&text[(2 + len)..].starts_with('"') {
                    let loc = make_loc(file, start_offset, start_offset + 2 + len);
                    return Err(vec![(
                        loc,
                        "Missing closing quote (\") after byte string".to_string(),
                    )]);
                }
                (Tok::ByteStringValue, 2 + len + 1)
            } else {
                let len = get_name_len(&text);
                (get_name_token(&text[..len]), len)
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
            let loc = make_loc(file, start_offset, start_offset);
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
    text.find(|c| match c {
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
        "define" => Tok::Define,
        "else" => Tok::Else,
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
        "script" => Tok::Script,
        "spec" => Tok::Spec,
        "struct" => Tok::Struct,
        "true" => Tok::True,
        "use" => Tok::Use,
        "while" => Tok::While,
        _ => Tok::IdentifierValue,
    }
}
