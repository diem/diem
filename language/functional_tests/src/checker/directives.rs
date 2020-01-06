// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    common::{strip, Sp},
    errors::*,
};
use std::iter::Peekable;

/// The basic unit of input to the directive parser.
enum Token {
    String(String),
    QuotedString(String),
    Whitespace(String),
}

/// Find the next token at the beginning of the input char stream.
fn next_token<I>(input: &mut Peekable<I>) -> Result<Option<Token>>
where
    I: Iterator<Item = (usize, char)>,
{
    macro_rules! next {
        () => {
            input.next().map(|(_, c)| c)
        };
    }

    macro_rules! peek {
        () => {
            input.peek().map(|(_, c)| c)
        };
    }

    let res = match next!() {
        Some('"') => {
            let mut buffer = String::new();
            loop {
                match next!() {
                    Some('"') => break Some(Token::QuotedString(buffer)),
                    Some('\\') => match next!() {
                        Some('\\') => buffer.push('\\'),
                        Some('n') => buffer.push('\n'),
                        Some('t') => buffer.push('\t'),
                        Some('r') => buffer.push('\r'),
                        Some('"') => buffer.push('"'),
                        Some(c) => bail!("unrecognized escape character \\{}", c),
                        None => bail!("unclosed escape character"),
                    },
                    Some(c) => buffer.push(c),
                    None => bail!("unclosed string literal"),
                }
            }
        }
        Some(c) if c.is_ascii_whitespace() => {
            let mut buffer = String::new();
            buffer.push(c);
            loop {
                match peek!() {
                    Some(c) if c.is_ascii_whitespace() => {
                        buffer.push(*c);
                        input.next();
                    }
                    _ => break,
                }
            }
            Some(Token::Whitespace(buffer))
        }
        Some(c) => {
            let mut buffer = String::new();
            buffer.push(c);
            loop {
                match peek!() {
                    Some('"') | None => break,
                    Some(c) if c.is_ascii_whitespace() => break,

                    Some(c) => {
                        buffer.push(*c);
                        input.next();
                    }
                }
            }
            Some(Token::String(buffer))
        }
        None => None,
    };
    Ok(res)
}

/// Split the input string into tokens with spans.
/// The tokens will later be used to build directives.
fn tokenize_patterns(s: &str) -> Result<Vec<Sp<Token>>> {
    let mut input = s.char_indices().peekable();
    let mut tokens = vec![];
    #[allow(clippy::while_let_loop)]
    loop {
        let start = match input.peek() {
            Some((idx, _)) => *idx,
            None => break,
        };
        let tok = match next_token(&mut input)? {
            Some(tok) => tok,
            None => break,
        };
        let end = match input.peek() {
            Some((idx, _)) => *idx,
            None => s.len(),
        };
        tokens.push(Sp::new(tok, start, end));
    }
    Ok(tokens)
}

/// Specification of an expected text pattern in the output.
///
/// There are two types of directives: positive and negative.
/// A positive directive means the pattern should match some text in the output,
/// while a nagative one considers such match to be an error.
#[derive(Debug, Eq, PartialEq)]
pub enum Directive {
    Check(String),
    Not(String),
}

impl Directive {
    /// Returns if the directive is a positive pattern.
    pub fn is_positive(&self) -> bool {
        match self {
            Self::Check(_) => true,
            Self::Not(_) => false,
        }
    }

    /// Returns if the directive is a negative pattern.
    pub fn is_negative(&self) -> bool {
        match self {
            Self::Check(_) => false,
            Self::Not(_) => true,
        }
    }

    /// Returns the pattern of the directive.
    pub fn pattern_str(&self) -> &str {
        match self {
            Self::Check(s) | Self::Not(s) => &s,
        }
    }

    /// Parses the line and extracts one or more directives from it.
    pub fn parse_line(s: &str) -> Result<Vec<Sp<Directive>>> {
        // TODO: rewrite how the offset is counted.
        let mut offset = 0;

        macro_rules! trim {
            ($s: expr) => {{
                let s = $s;
                let mut iter = s.char_indices();
                loop {
                    match iter.next() {
                        Some((idx, c)) => {
                            if !c.is_ascii_whitespace() {
                                offset += idx;
                                break &s[idx..];
                            }
                        }
                        None => break &s[s.len()..],
                    }
                }
            }};
        }

        macro_rules! strip {
            ($s: expr, $pat: expr) => {{
                let pat = $pat;
                let res = strip($s, pat);
                if res.is_some() {
                    offset += pat.len();
                }
                res
            }};
        }

        let s =
            strip!(trim!(s), "//").ok_or_else(|| format_err!("directives must start with //"))?;

        let (s, check) = {
            let s = trim!(s);
            if let Some(s) = strip!(s, "check") {
                (s, true)
            } else if let Some(s) = strip!(s, "not") {
                (s, false)
            } else {
                bail!("expects 'check' or 'not' after //")
            }
        };
        let s =
            strip!(trim!(s), ":").ok_or_else(|| format_err!("expects ':' after directive name"))?;

        let directives: Vec<_> = tokenize_patterns(s)?
            .into_iter()
            .filter_map(|Sp { inner, start, end }| match inner {
                Token::String(s) | Token::QuotedString(s) => {
                    let d = if check {
                        Directive::Check(s)
                    } else {
                        Directive::Not(s)
                    };
                    Some(Sp::new(d, start + offset, end + offset))
                }
                _ => None,
            })
            .collect();
        if directives.is_empty() {
            bail!("no directives found in line");
        }
        Ok(directives)
    }
}

impl AsRef<Directive> for Directive {
    fn as_ref(&self) -> &Directive {
        self
    }
}
