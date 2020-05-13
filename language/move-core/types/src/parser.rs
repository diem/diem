// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::{StructTag, TypeTag},
};
use anyhow::{bail, Result};
use std::iter::Peekable;

#[derive(Eq, PartialEq, Debug)]
enum Token {
    U8,
    U64,
    U128,
    Bool,
    AddressType,
    Vector,
    Whitespace(String),
    Name(String),
    Address(String),
    ColonColon,
    Lt,
    Gt,
    Comma,
    EOF,
    LBraceBrace,
    RBraceBrace,
}

impl Token {
    fn is_whitespace(&self) -> bool {
        match self {
            Self::Whitespace(_) => true,
            _ => false,
        }
    }
}

fn name_token(s: String) -> Token {
    match s.as_str() {
        "u8" => Token::U8,
        "u64" => Token::U64,
        "u128" => Token::U128,
        "bool" => Token::Bool,
        "address" => Token::AddressType,
        "vector" => Token::Vector,
        _ => Token::Name(s),
    }
}

#[allow(clippy::many_single_char_names)]
fn next_token(s: &str) -> Result<Option<(Token, usize)>> {
    let mut it = s.chars();
    match it.next() {
        None => Ok(None),
        Some(c) => Ok(Some(match c {
            '<' => (Token::Lt, 1),
            '>' => (Token::Gt, 1),
            ',' => (Token::Comma, 1),
            ':' => match it.next() {
                Some(':') => (Token::ColonColon, 2),
                _ => bail!("unrecognized token"),
            },
            '{' => match it.next() {
                Some('{') => (Token::LBraceBrace, 2),
                _ => bail!("unrecognized token"),
            },
            '}' => match it.next() {
                Some('}') => (Token::RBraceBrace, 2),
                _ => bail!("unrecognized token"),
            },
            '0' => match it.next() {
                Some(x @ 'x') | Some(x @ 'X') => match it.next() {
                    Some(c) if c.is_ascii_hexdigit() => {
                        let mut n: usize = 3;
                        let mut r = String::new();
                        r.push('0');
                        r.push(x);
                        r.push(c);
                        for c in it {
                            if c.is_ascii_hexdigit() {
                                r.push(c);
                                n += 1;
                            } else {
                                break;
                            }
                        }
                        (Token::Address(r), n)
                    }
                    _ => bail!("unrecognized token"),
                },
                _ => bail!("unrecognized token"),
            },
            c if c.is_ascii_whitespace() => {
                let mut n = 1;
                let mut r = String::new();
                r.push(c);
                for c in it {
                    if c.is_ascii_whitespace() {
                        r.push(c);
                        n += 1;
                    } else {
                        break;
                    }
                }
                (Token::Whitespace(r), n)
            }
            c if c.is_ascii_alphabetic() => {
                let mut n = 1;
                let mut r = String::new();
                r.push(c);
                for c in it {
                    if c.is_ascii_alphanumeric() {
                        r.push(c);
                        n += 1;
                    } else {
                        break;
                    }
                }
                (name_token(r), n)
            }
            _ => bail!("unrecognized token"),
        })),
    }
}

fn tokenize(mut s: &str) -> Result<Vec<Token>> {
    let mut v = vec![];
    while let Some((tok, n)) = next_token(s)? {
        v.push(tok);
        s = &s[n..];
    }
    Ok(v)
}

struct Parser<I: Iterator<Item = Token>> {
    it: Peekable<I>,
}

impl<I: Iterator<Item = Token>> Parser<I> {
    fn new<T: IntoIterator<Item = Token, IntoIter = I>>(v: T) -> Self {
        Self {
            it: v.into_iter().peekable(),
        }
    }

    fn next(&mut self) -> Result<Token> {
        match self.it.next() {
            Some(tok) => Ok(tok),
            None => bail!("out of tokens, this should not happen"),
        }
    }

    fn peek(&mut self) -> Option<&Token> {
        self.it.peek()
    }

    fn consume(&mut self, tok: Token) -> Result<()> {
        let t = self.next()?;
        if t != tok {
            bail!("expected token {:?}, got {:?}", tok, t)
        }
        Ok(())
    }

    fn parse_comma_list<F, R>(
        &mut self,
        parse_list_item: F,
        end_token: Token,
        allow_trailing_comma: bool,
    ) -> Result<Vec<R>>
    where
        F: Fn(&mut Self) -> Result<R>,
        R: std::fmt::Debug,
    {
        let mut v = vec![];
        if !(self.peek() == Some(&end_token)) {
            loop {
                v.push(parse_list_item(self)?);
                if self.peek() == Some(&end_token) {
                    break;
                }
                self.consume(Token::Comma)?;
                if self.peek() == Some(&end_token) && allow_trailing_comma {
                    break;
                }
            }
        }
        Ok(v)
    }

    fn parse_type_tag(&mut self) -> Result<TypeTag> {
        Ok(match self.next()? {
            Token::U8 => TypeTag::U8,
            Token::U64 => TypeTag::U64,
            Token::U128 => TypeTag::U128,
            Token::Bool => TypeTag::Bool,
            Token::AddressType => TypeTag::Address,
            Token::Vector => {
                self.consume(Token::Lt)?;
                let ty = self.parse_type_tag()?;
                self.consume(Token::Gt)?;
                TypeTag::Vector(Box::new(ty))
            }
            Token::Address(addr) => {
                self.consume(Token::ColonColon)?;
                match self.next()? {
                    Token::Name(module) => {
                        self.consume(Token::ColonColon)?;
                        match self.next()? {
                            Token::Name(name) => {
                                let ty_args = if self.peek() == Some(&Token::Lt) {
                                    self.next()?;
                                    let ty_args = self.parse_comma_list(
                                        |parser| parser.parse_type_tag(),
                                        Token::Gt,
                                        true,
                                    )?;
                                    self.consume(Token::Gt)?;
                                    ty_args
                                } else {
                                    vec![]
                                };
                                TypeTag::Struct(StructTag {
                                    address: AccountAddress::from_hex_literal(&addr)?,
                                    module: Identifier::new(module)?,
                                    name: Identifier::new(name)?,
                                    type_params: ty_args,
                                })
                            }
                            t => bail!("expected name, got {:?}", t),
                        }
                    }
                    t => bail!("expected name, got {:?}", t),
                }
            }
            tok => bail!("unexpected token {:?}, expected type tag", tok),
        })
    }
}

pub fn parse_type_tags(s: &str) -> Result<Vec<TypeTag>> {
    let mut tokens: Vec<_> = tokenize(s)?
        .into_iter()
        .filter(|tok| !tok.is_whitespace())
        .collect();
    tokens.push(Token::EOF);
    let mut parser = Parser::new(tokens);
    let tags = parser.parse_comma_list(|parser| parser.parse_type_tag(), Token::EOF, true);
    let tags = tags?;
    parser.consume(Token::EOF)?;
    Ok(tags)
}
