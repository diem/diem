// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::{StructTag, TypeTag},
    transaction_argument::TransactionArgument,
};
use anyhow::{bail, Result};
use std::iter::Peekable;

#[derive(Eq, PartialEq, Debug)]
enum Token {
    U8Type,
    U64Type,
    U128Type,
    BoolType,
    AddressType,
    VectorType,
    Whitespace(String),
    Name(String),
    Address(String),
    U8(String),
    U64(String),
    U128(String),
    Bytes(String),
    True,
    False,
    ColonColon,
    Lt,
    Gt,
    Comma,
    EOF,
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
        "u8" => Token::U8Type,
        "u64" => Token::U64Type,
        "u128" => Token::U128Type,
        "bool" => Token::BoolType,
        "address" => Token::AddressType,
        "vector" => Token::VectorType,
        "true" => Token::True,
        "false" => Token::False,
        _ => Token::Name(s),
    }
}

fn next_number(initial: char, mut it: impl Iterator<Item = char>) -> Result<(Token, usize)> {
    let mut num = String::new();
    num.push(initial);
    loop {
        match it.next() {
            Some(c) if c.is_ascii_digit() => num.push(c),
            Some(c) if c.is_alphanumeric() => {
                let mut suffix = String::new();
                suffix.push(c);
                loop {
                    match it.next() {
                        Some(c) if c.is_ascii_alphanumeric() => suffix.push(c),
                        _ => {
                            let len = num.len() + suffix.len();
                            let tok = match suffix.as_str() {
                                "u8" => Token::U8(num),
                                "u64" => Token::U64(num),
                                "u128" => Token::U128(num),
                                _ => bail!("invalid suffix"),
                            };
                            return Ok((tok, len));
                        }
                    }
                }
            }
            _ => {
                let len = num.len();
                return Ok((Token::U64(num), len));
            }
        }
    }
}

#[allow(clippy::many_single_char_names)]
fn next_token(s: &str) -> Result<Option<(Token, usize)>> {
    let mut it = s.chars().peekable();
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
            '0' if it.peek() == Some(&'x') || it.peek() == Some(&'X') => {
                it.next().unwrap();
                match it.next() {
                    Some(c) if c.is_ascii_hexdigit() => {
                        let mut r = String::new();
                        r.push('0');
                        r.push('x');
                        r.push(c);
                        for c in it {
                            if c.is_ascii_hexdigit() {
                                r.push(c);
                            } else {
                                break;
                            }
                        }
                        let len = r.len();
                        (Token::Address(r), len)
                    }
                    _ => bail!("unrecognized token"),
                }
            }
            c if c.is_ascii_digit() => next_number(c, it)?,
            'b' if it.peek() == Some(&'"') => {
                it.next().unwrap();
                let mut r = String::new();
                loop {
                    match it.next() {
                        Some('"') => break,
                        Some(c) if c.is_ascii_hexdigit() => r.push(c),
                        _ => bail!("unrecognized token"),
                    }
                }
                let len = r.len() + 3;
                (Token::Bytes(r), len)
            }
            c if c.is_ascii_whitespace() => {
                let mut r = String::new();
                r.push(c);
                for c in it {
                    if c.is_ascii_whitespace() {
                        r.push(c);
                    } else {
                        break;
                    }
                }
                let len = r.len();
                (Token::Whitespace(r), len)
            }
            c if c.is_ascii_alphabetic() => {
                let mut r = String::new();
                r.push(c);
                for c in it {
                    if c.is_ascii_alphanumeric() {
                        r.push(c);
                    } else {
                        break;
                    }
                }
                let len = r.len();
                (name_token(r), len)
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
            Token::U8Type => TypeTag::U8,
            Token::U64Type => TypeTag::U64,
            Token::U128Type => TypeTag::U128,
            Token::BoolType => TypeTag::Bool,
            Token::AddressType => TypeTag::Address,
            Token::VectorType => {
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

    fn parse_transaction_argument(&mut self) -> Result<TransactionArgument> {
        Ok(match self.next()? {
            Token::U8(s) => TransactionArgument::U8(s.parse()?),
            Token::U64(s) => TransactionArgument::U64(s.parse()?),
            Token::U128(s) => TransactionArgument::U128(s.parse()?),
            Token::True => TransactionArgument::Bool(true),
            Token::False => TransactionArgument::Bool(false),
            Token::Address(addr) => {
                TransactionArgument::Address(AccountAddress::from_hex_literal(&addr)?)
            }
            Token::Bytes(s) => TransactionArgument::U8Vector(hex::decode(s)?),
            tok => bail!("unexpected token {:?}, expected transaction argument", tok),
        })
    }
}

fn parse<F, T>(s: &str, f: F) -> Result<T>
where
    F: Fn(&mut Parser<std::vec::IntoIter<Token>>) -> Result<T>,
{
    let mut tokens: Vec<_> = tokenize(s)?
        .into_iter()
        .filter(|tok| !tok.is_whitespace())
        .collect();
    tokens.push(Token::EOF);
    let mut parser = Parser::new(tokens);
    let res = f(&mut parser)?;
    parser.consume(Token::EOF)?;
    Ok(res)
}

pub fn parse_type_tags(s: &str) -> Result<Vec<TypeTag>> {
    parse(s, |parser| {
        parser.parse_comma_list(|parser| parser.parse_type_tag(), Token::EOF, true)
    })
}

pub fn parse_transaction_arguments(s: &str) -> Result<Vec<TransactionArgument>> {
    parse(s, |parser| {
        parser.parse_comma_list(
            |parser| parser.parse_transaction_argument(),
            Token::EOF,
            true,
        )
    })
}

pub fn parse_transaction_argument(s: &str) -> Result<TransactionArgument> {
    parse(s, |parser| parser.parse_transaction_argument())
}

#[allow(clippy::unreadable_literal)]
#[test]
fn tests_parse_transaction_argument_positive() {
    use TransactionArgument as T;

    for (s, expected) in &[
        ("  0u8", T::U8(0)),
        ("0u8", T::U8(0)),
        ("255u8", T::U8(255)),
        ("0", T::U64(0)),
        ("0123", T::U64(123)),
        ("0u64", T::U64(0)),
        ("18446744073709551615", T::U64(18446744073709551615)),
        ("18446744073709551615u64", T::U64(18446744073709551615)),
        ("0u128", T::U128(0)),
        (
            "340282366920938463463374607431768211455u128",
            T::U128(340282366920938463463374607431768211455),
        ),
        ("true", T::Bool(true)),
        ("false", T::Bool(false)),
        (
            "0x0",
            T::Address(AccountAddress::from_hex_literal("0x0").unwrap()),
        ),
        (
            "0x54afa3526",
            T::Address(AccountAddress::from_hex_literal("0x54afa3526").unwrap()),
        ),
        (
            "0X54afa3526",
            T::Address(AccountAddress::from_hex_literal("0x54afa3526").unwrap()),
        ),
        ("b\"7fff\"", T::U8Vector(vec![0x7f, 0xff])),
        ("b\"\"", T::U8Vector(vec![])),
        ("b\"00\"", T::U8Vector(vec![0x00])),
        ("b\"deadbeef\"", T::U8Vector(vec![0xde, 0xad, 0xbe, 0xef])),
    ] {
        assert_eq!(&parse_transaction_argument(s).unwrap(), expected)
    }
}

#[test]
fn tests_parse_transaction_argument_negative() {
    for s in &[
        "-3",
        "0u42",
        "0u645",
        "0u64x",
        "0u6 4",
        "0u",
        "256u8",
        "18446744073709551616",
        "18446744073709551616u64",
        "340282366920938463463374607431768211456u128",
        "0xg",
        "0x00g0",
        "0x",
        "0x_",
        "",
        "b\"ffff",
        "b\"a \"",
        "b\" \"",
        "b\"0g\"",
        "b\"0\"",
        "garbage",
        "true3",
        "3false",
        "3 false",
        "",
    ] {
        assert!(parse_transaction_argument(s).is_err())
    }
}
