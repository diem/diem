// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use codespan::{ByteIndex, Span};
use move_ir_types::location::*;
use std::str::FromStr;

use crate::{
    errors::*,
    parser::{ast::*, lexer::*},
    shared::*,
};
use std::collections::BTreeMap;

// In the informal grammar comments in this file, Comma<T> is shorthand for:
//      (<T> ",")* <T>?
// Note that this allows an optional trailing comma.

//**************************************************************************************************
// Error Handling
//**************************************************************************************************

fn unexpected_token_error<'input>(tokens: &Lexer<'input>, expected: &str) -> Error {
    let loc = current_token_loc(tokens);
    let unexpected = if tokens.peek() == Tok::EOF {
        "end-of-file".to_string()
    } else {
        format!("'{}'", tokens.content())
    };
    vec![
        (loc, format!("Unexpected {}", unexpected)),
        (loc, format!("Expected {}", expected)),
    ]
}

//**************************************************************************************************
// Miscellaneous Utilities
//**************************************************************************************************

pub fn make_loc(file: &'static str, start: usize, end: usize) -> Loc {
    Loc::new(
        file,
        Span::new(ByteIndex(start as u32), ByteIndex(end as u32)),
    )
}

fn current_token_loc<'input>(tokens: &Lexer<'input>) -> Loc {
    let start_loc = tokens.start_loc();
    make_loc(
        tokens.file_name(),
        start_loc,
        start_loc + tokens.content().len(),
    )
}

fn spanned<T>(file: &'static str, start: usize, end: usize, value: T) -> Spanned<T> {
    Spanned {
        loc: make_loc(file, start, end),
        value,
    }
}

// Check for the specified token and consume it if it matches.
// Returns true if the token matches.
fn match_token<'input>(tokens: &mut Lexer<'input>, tok: Tok) -> Result<bool, Error> {
    if tokens.peek() == tok {
        tokens.advance()?;
        Ok(true)
    } else {
        Ok(false)
    }
}

// Check for the specified token and return an error if it does not match.
fn consume_token<'input>(tokens: &mut Lexer<'input>, tok: Tok) -> Result<(), Error> {
    if tokens.peek() != tok {
        let expected = format!("'{}'", &tok.to_string());
        return Err(unexpected_token_error(tokens, &expected));
    }
    tokens.advance()?;
    Ok(())
}

// Check for the identifier token with specified value and return an error if it does not match.
fn consume_identifier<'input>(tokens: &mut Lexer<'input>, value: &str) -> Result<(), Error> {
    if tokens.peek() == Tok::IdentifierValue && tokens.content() == value {
        tokens.advance()
    } else {
        let expected = format!("'{}'", value);
        Err(unexpected_token_error(tokens, &expected))
    }
}

// If the next token is the specified kind, consume it and return
// its source location.
fn consume_optional_token_with_loc<'input>(
    tokens: &mut Lexer<'input>,
    tok: Tok,
) -> Result<Option<Loc>, Error> {
    if tokens.peek() == tok {
        let start_loc = tokens.start_loc();
        tokens.advance()?;
        let end_loc = tokens.previous_end_loc();
        Ok(Some(make_loc(tokens.file_name(), start_loc, end_loc)))
    } else {
        Ok(None)
    }
}

// While parsing a list and expecting a ">" token to mark the end, replace
// a ">>" token with the expected ">". This handles the situation where there
// are nested type parameters that result in two adjacent ">" tokens, e.g.,
// "A<B<C>>".
fn adjust_token<'input>(tokens: &mut Lexer<'input>, end_token: Tok) {
    if tokens.peek() == Tok::GreaterGreater && end_token == Tok::Greater {
        tokens.replace_token(Tok::Greater, 1);
    }
}

// Parse a comma-separated list of items, including the specified starting and
// ending tokens.
fn parse_comma_list<'input, F, R>(
    tokens: &mut Lexer<'input>,
    start_token: Tok,
    end_token: Tok,
    parse_list_item: F,
    item_description: &str,
) -> Result<Vec<R>, Error>
where
    F: Fn(&mut Lexer<'input>) -> Result<R, Error>,
{
    let start_loc = tokens.start_loc();
    consume_token(tokens, start_token)?;
    parse_comma_list_after_start(
        tokens,
        start_loc,
        start_token,
        end_token,
        parse_list_item,
        item_description,
    )
}

// Parse a comma-separated list of items, including the specified ending token, but
// assuming that the starting token has already been consumed.
fn parse_comma_list_after_start<'input, F, R>(
    tokens: &mut Lexer<'input>,
    start_loc: usize,
    start_token: Tok,
    end_token: Tok,
    parse_list_item: F,
    item_description: &str,
) -> Result<Vec<R>, Error>
where
    F: Fn(&mut Lexer<'input>) -> Result<R, Error>,
{
    adjust_token(tokens, end_token);
    if match_token(tokens, end_token)? {
        return Ok(vec![]);
    }
    let mut v = vec![];
    loop {
        if tokens.peek() == Tok::Comma {
            let current_loc = tokens.start_loc();
            let loc = make_loc(tokens.file_name(), current_loc, current_loc);
            return Err(vec![(loc, format!("Expected {}", item_description))]);
        }
        v.push(parse_list_item(tokens)?);
        adjust_token(tokens, end_token);
        if match_token(tokens, end_token)? {
            break Ok(v);
        }
        if !match_token(tokens, Tok::Comma)? {
            let current_loc = tokens.start_loc();
            let loc = make_loc(tokens.file_name(), current_loc, current_loc);
            let loc2 = make_loc(tokens.file_name(), start_loc, start_loc);
            return Err(vec![
                (loc, format!("Expected '{}'", end_token)),
                (loc2, format!("To match this '{}'", start_token)),
            ]);
        }
        adjust_token(tokens, end_token);
        if match_token(tokens, end_token)? {
            break Ok(v);
        }
    }
}

// Parse a list of items, without specified start and end tokens, and the separator determined by
// the passed function `parse_list_continue`.
fn parse_list<'input, C, F, R>(
    tokens: &mut Lexer<'input>,
    mut parse_list_continue: C,
    parse_list_item: F,
) -> Result<Vec<R>, Error>
where
    C: FnMut(&mut Lexer<'input>) -> Result<bool, Error>,
    F: Fn(&mut Lexer<'input>) -> Result<R, Error>,
{
    let mut v = vec![];
    loop {
        v.push(parse_list_item(tokens)?);
        if !parse_list_continue(tokens)? {
            break Ok(v);
        }
    }
}

//**************************************************************************************************
// Identifiers, Addresses, and Names
//**************************************************************************************************

// Parse an identifier:
//      Identifier = <IdentifierValue>
fn parse_identifier<'input>(tokens: &mut Lexer<'input>) -> Result<Name, Error> {
    if tokens.peek() != Tok::IdentifierValue {
        return Err(unexpected_token_error(tokens, "an identifier"));
    }
    let start_loc = tokens.start_loc();
    let id = tokens.content().to_string();
    tokens.advance()?;
    let end_loc = tokens.previous_end_loc();
    Ok(spanned(tokens.file_name(), start_loc, end_loc, id))
}

// Parse an account address:
//      Address = <AddressValue>
fn parse_address<'input>(tokens: &mut Lexer<'input>) -> Result<Address, Error> {
    if tokens.peek() != Tok::AddressValue {
        return Err(unexpected_token_error(tokens, "an account address value"));
    }
    let addr =
        Address::parse_str(&tokens.content()).map_err(|msg| vec![(current_token_loc(tokens), msg)]);
    tokens.advance()?;
    addr
}

// Parse a variable name:
//      Var = <Identifier>
fn parse_var<'input>(tokens: &mut Lexer<'input>) -> Result<Var, Error> {
    Ok(Var(parse_identifier(tokens)?))
}

// Parse a field name:
//      Field = <Identifier>
fn parse_field<'input>(tokens: &mut Lexer<'input>) -> Result<Field, Error> {
    Ok(Field(parse_identifier(tokens)?))
}

// Parse a module name:
//      ModuleName = <Identifier>
fn parse_module_name<'input>(tokens: &mut Lexer<'input>) -> Result<ModuleName, Error> {
    Ok(ModuleName(parse_identifier(tokens)?))
}

// Parse a module identifier:
//      ModuleIdent = <Address> "::" <ModuleName>
fn parse_module_ident<'input>(tokens: &mut Lexer<'input>) -> Result<ModuleIdent, Error> {
    let start_loc = tokens.start_loc();
    let address = parse_address(tokens)?;
    consume_token(tokens, Tok::ColonColon)?;
    let name = parse_module_name(tokens)?;
    let end_loc = tokens.previous_end_loc();
    let m = ModuleIdent_ { address, name };
    Ok(ModuleIdent(spanned(
        tokens.file_name(),
        start_loc,
        end_loc,
        m,
    )))
}

// Parse a module access (a variable, struct type, or function):
//      ModuleAccess =
//          <Identifier>
//          | <ModuleName> "::" <Identifier>
//          | <ModuleIdent> "::" <Identifier>
fn parse_module_access<'input, F: FnOnce() -> String>(
    tokens: &mut Lexer<'input>,
    item_description: F,
) -> Result<ModuleAccess, Error> {
    let start_loc = tokens.start_loc();
    let acc = match tokens.peek() {
        Tok::IdentifierValue => {
            // Check if this is a ModuleName followed by "::".
            let m = parse_identifier(tokens)?;
            if match_token(tokens, Tok::ColonColon)? {
                let n = parse_identifier(tokens)?;
                ModuleAccess_::ModuleAccess(ModuleName(m), n)
            } else {
                ModuleAccess_::Name(m)
            }
        }

        Tok::AddressValue => {
            let m = parse_module_ident(tokens)?;
            consume_token(tokens, Tok::ColonColon)?;
            let n = parse_identifier(tokens)?;
            ModuleAccess_::QualifiedModuleAccess(m, n)
        }

        _ => {
            return Err(unexpected_token_error(tokens, &item_description()));
        }
    };
    let end_loc = tokens.previous_end_loc();
    Ok(spanned(tokens.file_name(), start_loc, end_loc, acc))
}

//**************************************************************************************************
// Fields and Bindings
//**************************************************************************************************

// Parse a field name optionally followed by a colon and an expression argument:
//      ExpField = <Field> <":" <Exp>>?
fn parse_exp_field<'input>(tokens: &mut Lexer<'input>) -> Result<(Field, Exp), Error> {
    let f = parse_field(tokens)?;
    let arg = if match_token(tokens, Tok::Colon)? {
        parse_exp(tokens)?
    } else {
        sp(
            f.loc(),
            Exp_::Name(sp(f.loc(), ModuleAccess_::Name(f.0.clone())), None),
        )
    };
    Ok((f, arg))
}

// Parse a field name optionally followed by a colon and a binding:
//      BindField = <Field> <":" <Bind>>?
//
// If the binding is not specified, the default is to use a variable
// with the same name as the field.
fn parse_bind_field<'input>(tokens: &mut Lexer<'input>) -> Result<(Field, Bind), Error> {
    let f = parse_field(tokens)?;
    let arg = if match_token(tokens, Tok::Colon)? {
        parse_bind(tokens)?
    } else {
        let v = Var(f.0.clone());
        sp(v.loc(), Bind_::Var(v))
    };
    Ok((f, arg))
}

// Parse a binding:
//      Bind =
//          <Var>
//          | <ModuleAccess> <OptionalTypeArgs> "{" Comma<BindField> "}"
fn parse_bind<'input>(tokens: &mut Lexer<'input>) -> Result<Bind, Error> {
    let start_loc = tokens.start_loc();
    if tokens.peek() == Tok::IdentifierValue {
        let next_tok = tokens.lookahead()?;
        if next_tok != Tok::LBrace && next_tok != Tok::Less && next_tok != Tok::ColonColon {
            let v = Bind_::Var(parse_var(tokens)?);
            let end_loc = tokens.previous_end_loc();
            return Ok(spanned(tokens.file_name(), start_loc, end_loc, v));
        }
    }
    // The item description specified here should include the special case above for
    // variable names, because if the current tokens cannot be parsed as a struct name
    // it is possible that the user intention was to use a variable name.
    let ty = parse_module_access(tokens, || "a variable or struct name".to_string())?;
    let ty_args = parse_optional_type_args(tokens)?;
    let args = parse_comma_list(
        tokens,
        Tok::LBrace,
        Tok::RBrace,
        parse_bind_field,
        "a field binding",
    )?;
    let end_loc = tokens.previous_end_loc();
    let unpack = Bind_::Unpack(ty, ty_args, args);
    Ok(spanned(tokens.file_name(), start_loc, end_loc, unpack))
}

// Parse a list of bindings, which can be zero, one, or more bindings:
//      BindList =
//          <Bind>
//          | "(" Comma<Bind> ")"
//
// The list is enclosed in parenthesis, except that the parenthesis are
// optional if there is a single Bind.
fn parse_bind_list<'input>(tokens: &mut Lexer<'input>) -> Result<BindList, Error> {
    let start_loc = tokens.start_loc();
    let b = if tokens.peek() != Tok::LParen {
        vec![parse_bind(tokens)?]
    } else {
        parse_comma_list(
            tokens,
            Tok::LParen,
            Tok::RParen,
            parse_bind,
            "a variable or structure binding",
        )?
    };
    let end_loc = tokens.previous_end_loc();
    Ok(spanned(tokens.file_name(), start_loc, end_loc, b))
}

// Parse a list of bindings for lambda.
//      LambdaBindList =
//          "|" Comma<Bind> "|"
fn parse_lambda_bind_list<'input>(tokens: &mut Lexer<'input>) -> Result<BindList, Error> {
    let start_loc = tokens.start_loc();
    let b = parse_comma_list(
        tokens,
        Tok::Pipe,
        Tok::Pipe,
        parse_bind,
        "a variable or structure binding",
    )?;
    let end_loc = tokens.previous_end_loc();
    Ok(spanned(tokens.file_name(), start_loc, end_loc, b))
}

//**************************************************************************************************
// Values
//**************************************************************************************************

// Parse a byte string:
//      ByteString = <ByteStringValue>
fn parse_byte_string<'input>(tokens: &mut Lexer<'input>) -> Result<Value_, Error> {
    if tokens.peek() != Tok::ByteStringValue {
        return Err(unexpected_token_error(tokens, "a byte string value"));
    }
    let s = tokens.content();
    let text = s[2..s.len() - 1].to_owned();
    let value_ = if s.starts_with("x\"") {
        Value_::HexString(text)
    } else {
        assert!(s.starts_with("b\""));
        Value_::ByteString(text)
    };
    tokens.advance()?;
    Ok(value_)
}

// Parse a value:
//      Value =
//          <Address>
//          | "true"
//          | "false"
//          | <U8Value>
//          | <U64Value>
//          | <U128Value>
//          | <ByteString>
fn parse_value<'input>(tokens: &mut Lexer<'input>) -> Result<Value, Error> {
    let start_loc = tokens.start_loc();
    let val = match tokens.peek() {
        Tok::AddressValue => {
            let addr = parse_address(tokens)?;
            Value_::Address(addr)
        }
        Tok::True => {
            tokens.advance()?;
            Value_::Bool(true)
        }
        Tok::False => {
            tokens.advance()?;
            Value_::Bool(false)
        }
        Tok::U8Value => {
            let mut s = tokens.content();
            if s.ends_with("u8") {
                s = &s[..s.len() - 2]
            }
            let i = u8::from_str(s).unwrap();
            tokens.advance()?;
            Value_::U8(i)
        }
        Tok::U64Value => {
            let mut s = tokens.content();
            if s.ends_with("u64") {
                s = &s[..s.len() - 3]
            }
            let i = u64::from_str(s).unwrap();
            tokens.advance()?;
            Value_::U64(i)
        }
        Tok::U128Value => {
            let mut s = tokens.content();
            if s.ends_with("u128") {
                s = &s[..s.len() - 4]
            }
            let i = u128::from_str(s).unwrap();
            tokens.advance()?;
            Value_::U128(i)
        }
        Tok::ByteStringValue => parse_byte_string(tokens)?,
        _ => unreachable!("parse_value called with invalid token"),
    };
    let end_loc = tokens.previous_end_loc();
    Ok(spanned(tokens.file_name(), start_loc, end_loc, val))
}

// Parse a num value:
//    Num = <NumValue>
fn parse_num(tokens: &mut Lexer) -> Result<u128, Error> {
    let start_loc = tokens.start_loc();
    assert_eq!(tokens.peek(), Tok::NumValue);
    let res = match u128::from_str(tokens.content()) {
        Ok(i) => Ok(i),
        Err(_) => {
            let end_loc = start_loc + tokens.content().len();
            let loc = make_loc(tokens.file_name(), start_loc, end_loc);
            let msg = "Invalid number literal. The given literal is too large to fit into the \
                       largest number type 'u128'";
            Err(vec![(loc, msg.to_owned())])
        }
    };
    tokens.advance()?;
    res
}

//**************************************************************************************************
// Sequences
//**************************************************************************************************

// Parse a sequence item:
//      SequenceItem =
//          <Exp>
//          | "let" <BindList> (":" <Type>)? ("=" <Exp>)?
fn parse_sequence_item<'input>(tokens: &mut Lexer<'input>) -> Result<SequenceItem, Error> {
    let start_loc = tokens.start_loc();
    let item = if match_token(tokens, Tok::Let)? {
        let b = parse_bind_list(tokens)?;
        let ty_opt = if match_token(tokens, Tok::Colon)? {
            Some(parse_type(tokens)?)
        } else {
            None
        };
        if match_token(tokens, Tok::Equal)? {
            let e = parse_exp(tokens)?;
            SequenceItem_::Bind(b, ty_opt, Box::new(e))
        } else {
            SequenceItem_::Declare(b, ty_opt)
        }
    } else {
        let e = parse_exp(tokens)?;
        SequenceItem_::Seq(Box::new(e))
    };
    let end_loc = tokens.previous_end_loc();
    Ok(spanned(tokens.file_name(), start_loc, end_loc, item))
}

// Parse a sequence:
//      Sequence = <UseDecl>* (<SequenceItem> ";")* <Exp>? "}"
//
// Note that this does not include the opening brace of a block but it
// does consume the closing right brace.
fn parse_sequence<'input>(tokens: &mut Lexer<'input>) -> Result<Sequence, Error> {
    let mut uses = vec![];
    while tokens.peek() == Tok::Use {
        uses.push(parse_use_decl(tokens)?);
    }

    let mut seq: Vec<SequenceItem> = vec![];
    let mut last_semicolon_loc = None;
    let mut eopt = None;
    while tokens.peek() != Tok::RBrace {
        let item = parse_sequence_item(tokens)?;
        if tokens.peek() == Tok::RBrace {
            // If the sequence ends with an expression that is not
            // followed by a semicolon, split out that expression
            // from the rest of the SequenceItems.
            if let SequenceItem_::Seq(e) = item.value {
                eopt = Some(Spanned {
                    loc: item.loc,
                    value: e.value,
                });
            } else {
                seq.push(item);
            }
            break;
        }
        seq.push(item);
        last_semicolon_loc = Some(current_token_loc(&tokens));
        consume_token(tokens, Tok::Semicolon)?;
    }
    tokens.advance()?; // consume the RBrace
    Ok((uses, seq, last_semicolon_loc, Box::new(eopt)))
}

//**************************************************************************************************
// Expressions
//**************************************************************************************************

// Parse an expression term:
//      Term =
//          "break"
//          | "continue"
//          | <NameExp>
//          | <Value>
//          | <Num>
//          | "(" Comma<Exp> ")"
//          | "(" <Exp> ":" <Type> ")"
//          | "(" <Exp> "as" <Type> ")"
//          | "{" <Sequence>
fn parse_term<'input>(tokens: &mut Lexer<'input>) -> Result<Exp, Error> {
    let start_loc = tokens.start_loc();
    let term = match tokens.peek() {
        Tok::Break => {
            tokens.advance()?;
            Exp_::Break
        }

        Tok::Continue => {
            tokens.advance()?;
            Exp_::Continue
        }

        Tok::IdentifierValue => parse_name_exp(tokens)?,

        Tok::AddressValue => {
            // Check if this is a ModuleIdent (in a ModuleAccess).
            if tokens.lookahead()? == Tok::ColonColon {
                parse_name_exp(tokens)?
            } else {
                Exp_::Value(parse_value(tokens)?)
            }
        }

        Tok::True
        | Tok::False
        | Tok::U8Value
        | Tok::U64Value
        | Tok::U128Value
        | Tok::ByteStringValue => Exp_::Value(parse_value(tokens)?),

        Tok::NumValue => Exp_::InferredNum(parse_num(tokens)?),

        // "(" Comma<Exp> ")"
        // "(" <Exp> ":" <Type> ")"
        // "(" <Exp> "as" <Type> ")"
        Tok::LParen => {
            let list_loc = tokens.start_loc();
            tokens.advance()?; // consume the LParen
            if match_token(tokens, Tok::RParen)? {
                Exp_::Unit
            } else {
                // If there is a single expression inside the parens,
                // then it may be followed by a colon and a type annotation.
                let e = parse_exp(tokens)?;
                if match_token(tokens, Tok::Colon)? {
                    let ty = parse_type(tokens)?;
                    consume_token(tokens, Tok::RParen)?;
                    Exp_::Annotate(Box::new(e), ty)
                } else if match_token(tokens, Tok::As)? {
                    let ty = parse_type(tokens)?;
                    consume_token(tokens, Tok::RParen)?;
                    Exp_::Cast(Box::new(e), ty)
                } else {
                    if tokens.peek() != Tok::RParen {
                        consume_token(tokens, Tok::Comma)?;
                    }
                    let mut es = parse_comma_list_after_start(
                        tokens,
                        list_loc,
                        Tok::LParen,
                        Tok::RParen,
                        parse_exp,
                        "an expression",
                    )?;
                    if es.is_empty() {
                        e.value
                    } else {
                        es.insert(0, e);
                        Exp_::ExpList(es)
                    }
                }
            }
        }

        // "{" <Sequence>
        Tok::LBrace => {
            tokens.advance()?; // consume the LBrace
            Exp_::Block(parse_sequence(tokens)?)
        }

        Tok::Spec => {
            let spec_block = parse_spec_block(tokens)?;
            Exp_::Spec(spec_block)
        }

        _ => {
            return Err(unexpected_token_error(tokens, "an expression term"));
        }
    };
    let end_loc = tokens.previous_end_loc();
    Ok(spanned(tokens.file_name(), start_loc, end_loc, term))
}

// Parse a pack, call, or other reference to a name:
//      NameExp =
//          <ModuleAccess> <OptionalTypeArgs> "{" Comma<ExpField> "}"
//          | <ModuleAccess> <OptionalTypeArgs> "(" Comma<Exp> ")"
//          | <ModuleAccess> <OptionalTypeArgs>
fn parse_name_exp<'input>(tokens: &mut Lexer<'input>) -> Result<Exp_, Error> {
    let n = parse_module_access(tokens, || {
        panic!("parse_name_exp with something other than a ModuleAccess")
    })?;

    // There's an ambiguity if the name is followed by a "<". If there is no whitespace
    // after the name, treat it as the start of a list of type arguments. Otherwise
    // assume that the "<" is a boolean operator.
    let mut tys = None;
    let start_loc = tokens.start_loc();
    if tokens.peek() == Tok::Less && start_loc == n.loc.span().end().to_usize() {
        let loc = make_loc(tokens.file_name(), start_loc, start_loc);
        tys = parse_optional_type_args(tokens).map_err(|mut e| {
            let msg = "Perhaps you need a blank space before this '<' operator?";
            e.push((loc, msg.to_owned()));
            e
        })?;
    }

    match tokens.peek() {
        // Pack: "{" Comma<ExpField> "}"
        Tok::LBrace => {
            let fs = parse_comma_list(
                tokens,
                Tok::LBrace,
                Tok::RBrace,
                parse_exp_field,
                "a field expression",
            )?;
            Ok(Exp_::Pack(n, tys, fs))
        }

        // Call: "(" Comma<Exp> ")"
        Tok::LParen => {
            let rhs = parse_call_args(tokens)?;
            Ok(Exp_::Call(n, tys, rhs))
        }

        // Other name reference...
        _ => Ok(Exp_::Name(n, tys)),
    }
}

// Parse the arguments to a call: "(" Comma<Exp> ")"
fn parse_call_args<'input>(tokens: &mut Lexer<'input>) -> Result<Spanned<Vec<Exp>>, Error> {
    let start_loc = tokens.start_loc();
    let args = parse_comma_list(
        tokens,
        Tok::LParen,
        Tok::RParen,
        parse_exp,
        "a call argument expression",
    )?;
    let end_loc = tokens.previous_end_loc();
    Ok(spanned(tokens.file_name(), start_loc, end_loc, args))
}

// Return true if the current token is one that might occur after an Exp.
// This is needed, for example, to check for the optional Exp argument to
// a return (where "return" is itself an Exp).
fn at_end_of_exp<'input>(tokens: &mut Lexer<'input>) -> bool {
    matches!(
        tokens.peek(),
        // These are the tokens that can occur after an Exp. If the grammar
        // changes, we need to make sure that these are kept up to date and that
        // none of these tokens can occur at the beginning of an Exp.
        Tok::Else | Tok::RBrace | Tok::RParen | Tok::Comma | Tok::Colon | Tok::Semicolon
    )
}

// Parse an expression:
//      Exp =
//            <LambdaBindList> <Exp>        spec only
//          | <Quantifier>                  spec only
//          | "if" "(" <Exp> ")" <Exp> ("else" <Exp>)?
//          | "while" "(" <Exp> ")" <Exp>
//          | "loop" <Exp>
//          | "return" <Exp>?
//          | "abort" <Exp>
//          | <BinOpExp>
//          | <UnaryExp> "=" <Exp>
fn parse_exp<'input>(tokens: &mut Lexer<'input>) -> Result<Exp, Error> {
    let start_loc = tokens.start_loc();
    let exp = match tokens.peek() {
        Tok::Pipe => {
            let bindings = parse_lambda_bind_list(tokens)?;
            let body = Box::new(parse_exp(tokens)?);
            Exp_::Lambda(bindings, body)
        }
        Tok::IdentifierValue if is_quant(tokens) => parse_quant(tokens)?,
        Tok::If => {
            tokens.advance()?;
            consume_token(tokens, Tok::LParen)?;
            let eb = Box::new(parse_exp(tokens)?);
            consume_token(tokens, Tok::RParen)?;
            let et = Box::new(parse_exp(tokens)?);
            let ef = if match_token(tokens, Tok::Else)? {
                Some(Box::new(parse_exp(tokens)?))
            } else {
                None
            };
            Exp_::IfElse(eb, et, ef)
        }
        Tok::While => {
            tokens.advance()?;
            consume_token(tokens, Tok::LParen)?;
            let eb = Box::new(parse_exp(tokens)?);
            consume_token(tokens, Tok::RParen)?;
            let eloop = Box::new(parse_exp(tokens)?);
            Exp_::While(eb, eloop)
        }
        Tok::Loop => {
            tokens.advance()?;
            let eloop = Box::new(parse_exp(tokens)?);
            Exp_::Loop(eloop)
        }
        Tok::Return => {
            tokens.advance()?;
            let e = if at_end_of_exp(tokens) {
                None
            } else {
                Some(Box::new(parse_exp(tokens)?))
            };
            Exp_::Return(e)
        }
        Tok::Abort => {
            tokens.advance()?;
            let e = Box::new(parse_exp(tokens)?);
            Exp_::Abort(e)
        }
        _ => {
            // This could be either an assignment or a binary operator
            // expression.
            let lhs = parse_unary_exp(tokens)?;
            if tokens.peek() != Tok::Equal {
                return parse_binop_exp(tokens, lhs, /* min_prec */ 1);
            }
            tokens.advance()?; // consume the "="
            let rhs = Box::new(parse_exp(tokens)?);
            Exp_::Assign(Box::new(lhs), rhs)
        }
    };
    let end_loc = tokens.previous_end_loc();
    Ok(spanned(tokens.file_name(), start_loc, end_loc, exp))
}

// Get the precedence of a binary operator. The minimum precedence value
// is 1, and larger values have higher precedence. For tokens that are not
// binary operators, this returns a value of zero so that they will be
// below the minimum value and will mark the end of the binary expression
// for the code in parse_binop_exp.
fn get_precedence(token: Tok) -> u32 {
    match token {
        // Reserved minimum precedence value is 1
        Tok::EqualEqualGreater => 2,
        Tok::PipePipe => 3,
        Tok::AmpAmp => 4,
        Tok::EqualEqual => 5,
        Tok::ExclaimEqual => 5,
        Tok::Less => 5,
        Tok::Greater => 5,
        Tok::LessEqual => 5,
        Tok::GreaterEqual => 5,
        Tok::PeriodPeriod => 6,
        Tok::Pipe => 7,
        Tok::Caret => 8,
        Tok::Amp => 9,
        Tok::LessLess => 10,
        Tok::GreaterGreater => 10,
        Tok::Plus => 11,
        Tok::Minus => 11,
        Tok::Star => 12,
        Tok::Slash => 12,
        Tok::Percent => 12,
        _ => 0, // anything else is not a binary operator
    }
}

// Parse a binary operator expression:
//      BinOpExp =
//          <BinOpExp> <BinOp> <BinOpExp>
//          | <UnaryExp>
//      BinOp = (listed from lowest to highest precedence)
//          "==>"                                       spec only
//          | "||"
//          | "&&"
//          | "==" | "!=" | "<" | ">" | "<=" | ">="
//          | ".."                                      spec only
//          | "|"
//          | "^"
//          | "&"
//          | "<<" | ">>"
//          | "+" | "-"
//          | "*" | "/" | "%"
//
// This function takes the LHS of the expression as an argument, and it
// continues parsing binary expressions as long as they have at least the
// specified "min_prec" minimum precedence.
fn parse_binop_exp<'input>(
    tokens: &mut Lexer<'input>,
    lhs: Exp,
    min_prec: u32,
) -> Result<Exp, Error> {
    let mut result = lhs;
    let mut next_tok_prec = get_precedence(tokens.peek());

    while next_tok_prec >= min_prec {
        // Parse the operator.
        let op_start_loc = tokens.start_loc();
        let op_token = tokens.peek();
        tokens.advance()?;
        let op_end_loc = tokens.previous_end_loc();

        let mut rhs = parse_unary_exp(tokens)?;

        // If the next token is another binary operator with a higher
        // precedence, then recursively parse that expression as the RHS.
        let this_prec = next_tok_prec;
        next_tok_prec = get_precedence(tokens.peek());
        if this_prec < next_tok_prec {
            rhs = parse_binop_exp(tokens, rhs, this_prec + 1)?;
            next_tok_prec = get_precedence(tokens.peek());
        }

        let op = match op_token {
            Tok::EqualEqual => BinOp_::Eq,
            Tok::ExclaimEqual => BinOp_::Neq,
            Tok::Less => BinOp_::Lt,
            Tok::Greater => BinOp_::Gt,
            Tok::LessEqual => BinOp_::Le,
            Tok::GreaterEqual => BinOp_::Ge,
            Tok::PipePipe => BinOp_::Or,
            Tok::AmpAmp => BinOp_::And,
            Tok::Caret => BinOp_::Xor,
            Tok::Pipe => BinOp_::BitOr,
            Tok::Amp => BinOp_::BitAnd,
            Tok::LessLess => BinOp_::Shl,
            Tok::GreaterGreater => BinOp_::Shr,
            Tok::Plus => BinOp_::Add,
            Tok::Minus => BinOp_::Sub,
            Tok::Star => BinOp_::Mul,
            Tok::Slash => BinOp_::Div,
            Tok::Percent => BinOp_::Mod,
            Tok::PeriodPeriod => BinOp_::Range,
            Tok::EqualEqualGreater => BinOp_::Implies,
            _ => panic!("Unexpected token that is not a binary operator"),
        };
        let sp_op = spanned(tokens.file_name(), op_start_loc, op_end_loc, op);

        let start_loc = result.loc.span().start().to_usize();
        let end_loc = tokens.previous_end_loc();
        let e = Exp_::BinopExp(Box::new(result), sp_op, Box::new(rhs));
        result = spanned(tokens.file_name(), start_loc, end_loc, e);
    }

    Ok(result)
}

// Parse a unary expression:
//      UnaryExp =
//          "!" <UnaryExp>
//          | "&mut" <UnaryExp>
//          | "&" <UnaryExp>
//          | "*" <UnaryExp>
//          | "move" <Var>
//          | "copy" <Var>
//          | <DotOrIndexChain>
fn parse_unary_exp<'input>(tokens: &mut Lexer<'input>) -> Result<Exp, Error> {
    let start_loc = tokens.start_loc();
    let exp = match tokens.peek() {
        Tok::Exclaim => {
            tokens.advance()?;
            let op_end_loc = tokens.previous_end_loc();
            let op = spanned(tokens.file_name(), start_loc, op_end_loc, UnaryOp_::Not);
            let e = parse_unary_exp(tokens)?;
            Exp_::UnaryExp(op, Box::new(e))
        }
        Tok::AmpMut => {
            tokens.advance()?;
            let e = parse_unary_exp(tokens)?;
            Exp_::Borrow(true, Box::new(e))
        }
        Tok::Amp => {
            tokens.advance()?;
            let e = parse_unary_exp(tokens)?;
            Exp_::Borrow(false, Box::new(e))
        }
        Tok::Star => {
            tokens.advance()?;
            let e = parse_unary_exp(tokens)?;
            Exp_::Dereference(Box::new(e))
        }
        Tok::Move => {
            tokens.advance()?;
            Exp_::Move(parse_var(tokens)?)
        }
        Tok::Copy => {
            tokens.advance()?;
            Exp_::Copy(parse_var(tokens)?)
        }
        _ => {
            return parse_dot_or_index_chain(tokens);
        }
    };
    let end_loc = tokens.previous_end_loc();
    Ok(spanned(tokens.file_name(), start_loc, end_loc, exp))
}

// Parse an expression term optionally followed by a chain of dot or index accesses:
//      DotOrIndexChain =
//          <DotOrIndexChain> "." <Identifier>
//          | <DotOrIndexChain> "[" <Exp> "]"                      spec only
//          | <Term>
fn parse_dot_or_index_chain<'input>(tokens: &mut Lexer<'input>) -> Result<Exp, Error> {
    let start_loc = tokens.start_loc();
    let mut lhs = parse_term(tokens)?;
    loop {
        let exp = match tokens.peek() {
            Tok::Period => {
                tokens.advance()?;
                let n = parse_identifier(tokens)?;
                Exp_::Dot(Box::new(lhs), n)
            }
            Tok::LBracket => {
                tokens.advance()?;
                let index = parse_exp(tokens)?;
                let exp = Exp_::Index(Box::new(lhs), Box::new(index));
                consume_token(tokens, Tok::RBracket)?;
                exp
            }
            _ => break,
        };
        let end_loc = tokens.previous_end_loc();
        lhs = spanned(tokens.file_name(), start_loc, end_loc, exp);
    }
    Ok(lhs)
}

// Lookahead to determine whether this is a quantifier. This matches
//
//      ( "exists" | "forall" ) <Identifier> ( ":" | <Identifier> ) ...
//
// as a sequence to identify a quantifier. While the <Identifier> after
// the exists/forall would by syntactically sufficient (Move does not
// have affixed identifiers in expressions), we add another token
// of lookahead to keep the result more precise in the presence of
// syntax errors.
fn is_quant<'input>(tokens: &mut Lexer<'input>) -> bool {
    if !matches!(tokens.content(), "exists" | "forall") {
        return false;
    }
    match tokens.lookahead2() {
        Err(_) => false,
        Ok((tok1, tok2)) => {
            tok1 == Tok::IdentifierValue && matches!(tok2, Tok::Colon | Tok::IdentifierValue)
        }
    }
}

// Parses a quantifier expressions, assuming is_quant(tokens) is true.
//
//   <Quantifier> = ( "forall" | "exists" ) <QuantifierBindings> ("where" <Exp>)? ":" Exp
//   <QuantifierBindings> = <QuantifierBind> ("," <QuantifierBind>)*
//   <QuantifierBind> = <Identifier> ":" <Type> | <Identifier> "in" <Exp>
//
// Parsing happens recursively and quantifiers are immediately reduced as syntactic sugar
// for lambdas.
fn parse_quant<'input>(tokens: &mut Lexer<'input>) -> Result<Exp_, Error> {
    let is_forall = matches!(tokens.content(), "forall");
    tokens.advance()?;
    parse_quant_cont(is_forall, tokens)
}

// Parses quantifier bindings recursively until the body is reached.
fn parse_quant_cont<'input>(is_forall: bool, tokens: &mut Lexer<'input>) -> Result<Exp_, Error> {
    // Parse the next quantifier variable binding
    let start_loc = tokens.start_loc();
    let ident = parse_identifier(tokens)?;
    let ident_end_loc = tokens.previous_end_loc();
    let range = if tokens.peek() == Tok::Colon {
        // This is a quantifier over the full domain of a type.
        // Built `domain<ty>()` expression.
        tokens.advance()?;
        let ty = parse_type(tokens)?;
        make_builtin_call(ty.loc, "$spec_domain", Some(vec![ty]), vec![])
    } else {
        // This is a quantifier over a value, like a vector or a range.
        consume_identifier(tokens, "in")?;
        parse_exp(tokens)?
    };

    // Continue parsing more bindings or the body of the quantifier
    let (body_loc, body_) = if tokens.peek() == Tok::Comma {
        tokens.advance()?;
        (tokens.start_loc(), parse_quant_cont(is_forall, tokens)?)
    } else {
        (tokens.start_loc(), parse_quant_body(is_forall, tokens)?)
    };
    let body = spanned(
        tokens.file_name(),
        body_loc,
        tokens.previous_end_loc(),
        body_,
    );

    // Construct ::<all|any>(range, |ident| body) as the result
    let bind = spanned(
        tokens.file_name(),
        start_loc,
        ident_end_loc,
        Bind_::Var(Var(ident)),
    );
    let bind_list = sp(bind.loc, vec![bind]);
    let lambda = spanned(
        tokens.file_name(),
        start_loc,
        tokens.previous_end_loc(),
        Exp_::Lambda(bind_list, Box::new(body)),
    );
    Ok(make_builtin_call(
        lambda.loc,
        if is_forall { "$spec_all" } else { "$spec_any" },
        None,
        vec![range, lambda],
    )
    .value)
}

// Parse quantifier body.
fn parse_quant_body<'input>(is_forall: bool, tokens: &mut Lexer<'input>) -> Result<Exp_, Error> {
    let opt_cond = match tokens.peek() {
        Tok::IdentifierValue if tokens.content() == "where" => {
            tokens.advance()?;
            Some(parse_exp(tokens)?)
        }
        _ => None,
    };
    consume_token(tokens, Tok::Colon)?;
    let body = parse_exp(tokens)?;
    if let Some(cond) = opt_cond {
        let op = sp(
            cond.loc,
            if is_forall {
                BinOp_::Implies
            } else {
                BinOp_::And
            },
        );
        Ok(Exp_::BinopExp(Box::new(cond), op, Box::new(body)))
    } else {
        Ok(body.value)
    }
}

fn make_builtin_call(loc: Loc, name: &str, type_args: Option<Vec<Type>>, args: Vec<Exp>) -> Exp {
    let maccess = sp(loc, ModuleAccess_::Name(sp(loc, name.to_string())));
    sp(loc, Exp_::Call(maccess, type_args, sp(loc, args)))
}

//**************************************************************************************************
// Types
//**************************************************************************************************

// Parse a Type:
//      Type =
//          <ModuleAccess> ("<" Comma<Type> ">")?
//          | "&" <Type>
//          | "&mut" <Type>
//          | "|" Comma<Type> "|" Type   (spec only)
//          | "(" Comma<Type> ")"
fn parse_type<'input>(tokens: &mut Lexer<'input>) -> Result<Type, Error> {
    let start_loc = tokens.start_loc();
    let t = match tokens.peek() {
        Tok::LParen => {
            let mut ts = parse_comma_list(tokens, Tok::LParen, Tok::RParen, parse_type, "a type")?;
            match ts.len() {
                0 => Type_::Unit,
                1 => ts.pop().unwrap().value,
                _ => Type_::Multiple(ts),
            }
        }
        Tok::Amp => {
            tokens.advance()?;
            let t = parse_type(tokens)?;
            Type_::Ref(false, Box::new(t))
        }
        Tok::AmpMut => {
            tokens.advance()?;
            let t = parse_type(tokens)?;
            Type_::Ref(true, Box::new(t))
        }
        Tok::Pipe => {
            let args = parse_comma_list(tokens, Tok::Pipe, Tok::Pipe, parse_type, "a type")?;
            let result = parse_type(tokens)?;
            return Ok(spanned(
                tokens.file_name(),
                start_loc,
                tokens.previous_end_loc(),
                Type_::Fun(args, Box::new(result)),
            ));
        }
        _ => {
            let tn = parse_module_access(tokens, || "a type name".to_string())?;
            let tys = if tokens.peek() == Tok::Less {
                parse_comma_list(tokens, Tok::Less, Tok::Greater, parse_type, "a type")?
            } else {
                vec![]
            };
            Type_::Apply(Box::new(tn), tys)
        }
    };
    let end_loc = tokens.previous_end_loc();
    Ok(spanned(tokens.file_name(), start_loc, end_loc, t))
}

// Parse an optional list of type arguments.
//    OptionalTypeArgs = "<" Comma<Type> ">" | <empty>
fn parse_optional_type_args<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<Option<Vec<Type>>, Error> {
    if tokens.peek() == Tok::Less {
        Ok(Some(parse_comma_list(
            tokens,
            Tok::Less,
            Tok::Greater,
            parse_type,
            "a type",
        )?))
    } else {
        Ok(None)
    }
}

// Parse a type parameter:
//      TypeParameter =
//          <Identifier> <Constraint>?
//      Constraint =
//          ":" "copyable"
//          | ":" "resource"
fn parse_type_parameter<'input>(tokens: &mut Lexer<'input>) -> Result<(Name, Kind), Error> {
    let n = parse_identifier(tokens)?;

    let kind = if match_token(tokens, Tok::Colon)? {
        let start_loc = tokens.start_loc();
        let k = match tokens.peek() {
            Tok::Copyable => Kind_::Affine,
            Tok::Resource => Kind_::Resource,
            _ => {
                let expected = "either 'copyable' or 'resource'";
                return Err(unexpected_token_error(tokens, expected));
            }
        };
        tokens.advance()?;
        let end_loc = tokens.previous_end_loc();
        spanned(tokens.file_name(), start_loc, end_loc, k)
    } else {
        sp(n.loc, Kind_::Unknown)
    };
    Ok((n, kind))
}

// Parse optional type parameter list.
//    OptionalTypeParameters = "<" Comma<TypeParameter> ">" | <empty>
fn parse_optional_type_parameters<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<Vec<(Name, Kind)>, Error> {
    if tokens.peek() == Tok::Less {
        parse_comma_list(
            tokens,
            Tok::Less,
            Tok::Greater,
            parse_type_parameter,
            "a type parameter",
        )
    } else {
        Ok(vec![])
    }
}

//**************************************************************************************************
// Functions
//**************************************************************************************************

// Parse a function declaration:
//      FunctionDecl =
//          <NativeFunctionDecl>
//          | <MoveFunctionDecl>
//      NativeFunctionDecl =
//          <DocComments> "native" ( "public" )? "fun"
//          <FunctionDefName> "(" Comma<Parameter> ")"
//          (":" <Type>)?
//          ("acquires" <ModuleAccess> ("," <ModuleAccess>)*)?
//          ";"
//      MoveFunctionDecl =
//          <DocComments> ( "public" )? "fun"
//          <FunctionDefName> "(" Comma<Parameter> ")"
//          (":" <Type>)?
//          ("acquires" <ModuleAccess> ("," <ModuleAccess>)*)?
//          "{" <Sequence>
//      FunctionDefName =
//          <Identifier> <OptionalTypeParameters>
//
// If the "allow_native" parameter is false, this will only accept Move
// functions.
fn parse_function_decl<'input>(
    tokens: &mut Lexer<'input>,
    allow_native: bool,
) -> Result<Function, Error> {
    tokens.match_doc_comments();
    let start_loc = tokens.start_loc();
    // Record the source location of the "native" keyword (if there is one).
    let native_opt = if allow_native {
        consume_optional_token_with_loc(tokens, Tok::Native)?
    } else {
        if tokens.peek() == Tok::Native {
            let loc = current_token_loc(tokens);
            return Err(vec![(
                loc,
                "Native functions can only be declared inside a module".to_string(),
            )]);
        }
        None
    };

    // (<Public>)?
    let public_opt = consume_optional_token_with_loc(tokens, Tok::Public)?;
    let visibility = if let Some(loc) = public_opt {
        FunctionVisibility::Public(loc)
    } else {
        FunctionVisibility::Internal
    };

    // "fun" <FunctionDefName>
    consume_token(tokens, Tok::Fun)?;
    let name = FunctionName(parse_identifier(tokens)?);
    let type_parameters = parse_optional_type_parameters(tokens)?;

    // "(" Comma<Parameter> ")"
    let parameters = parse_comma_list(
        tokens,
        Tok::LParen,
        Tok::RParen,
        parse_parameter,
        "a function parameter",
    )?;

    // (":" <Type>)?
    let return_type = if match_token(tokens, Tok::Colon)? {
        parse_type(tokens)?
    } else {
        sp(name.loc(), Type_::Unit)
    };

    // ("acquires" (<ModuleAccess> ",")* <ModuleAccess> ","?
    let mut acquires = vec![];
    if match_token(tokens, Tok::Acquires)? {
        let follows_acquire = |tok| matches!(tok, Tok::Semicolon | Tok::LBrace);
        loop {
            acquires.push(parse_module_access(tokens, || {
                "a resource struct name".to_string()
            })?);
            if follows_acquire(tokens.peek()) {
                break;
            }
            consume_token(tokens, Tok::Comma)?;
            if follows_acquire(tokens.peek()) {
                break;
            }
        }
    }

    let body = match native_opt {
        Some(loc) => {
            consume_token(tokens, Tok::Semicolon)?;
            sp(loc, FunctionBody_::Native)
        }
        _ => {
            let start_loc = tokens.start_loc();
            consume_token(tokens, Tok::LBrace)?;
            let seq = parse_sequence(tokens)?;
            let end_loc = tokens.previous_end_loc();
            sp(
                make_loc(tokens.file_name(), start_loc, end_loc),
                FunctionBody_::Defined(seq),
            )
        }
    };

    let signature = FunctionSignature {
        type_parameters,
        parameters,
        return_type,
    };

    let loc = make_loc(tokens.file_name(), start_loc, tokens.previous_end_loc());
    Ok(Function {
        loc,
        visibility,
        signature,
        acquires,
        name,
        body,
    })
}

// Parse a function parameter:
//      Parameter = <Var> ":" <Type>
fn parse_parameter<'input>(tokens: &mut Lexer<'input>) -> Result<(Var, Type), Error> {
    let v = parse_var(tokens)?;
    consume_token(tokens, Tok::Colon)?;
    let t = parse_type(tokens)?;
    Ok((v, t))
}

//**************************************************************************************************
// Structs
//**************************************************************************************************

// Parse a struct definition:
//      StructDefinition =
//          <DocComments> "resource"? "struct" <StructDefName> "{" Comma<FieldAnnot> "}"
//          | <DocComments> "native" "resource"? "struct" <StructDefName> ";"
//      StructDefName =
//          <Identifier> <OptionalTypeParameters>
fn parse_struct_definition<'input>(tokens: &mut Lexer<'input>) -> Result<StructDefinition, Error> {
    tokens.match_doc_comments();
    let start_loc = tokens.start_loc();

    // Record the source location of the "native" keyword (if there is one).
    let native_opt = consume_optional_token_with_loc(tokens, Tok::Native)?;

    // Record the source location of the "resource" keyword (if there is one).
    let resource_opt = consume_optional_token_with_loc(tokens, Tok::Resource)?;

    consume_token(tokens, Tok::Struct)?;

    // <StructDefName>
    let name = StructName(parse_identifier(tokens)?);
    let type_parameters = parse_optional_type_parameters(tokens)?;

    let fields = match native_opt {
        Some(loc) => {
            consume_token(tokens, Tok::Semicolon)?;
            StructFields::Native(loc)
        }
        _ => {
            let list = parse_comma_list(
                tokens,
                Tok::LBrace,
                Tok::RBrace,
                parse_field_annot,
                "a field",
            )?;
            StructFields::Defined(list)
        }
    };

    let loc = make_loc(tokens.file_name(), start_loc, tokens.previous_end_loc());
    Ok(StructDefinition {
        loc,
        resource_opt,
        name,
        type_parameters,
        fields,
    })
}

// Parse a field annotated with a type:
//      FieldAnnot = <DocComments> <Field> ":" <Type>
fn parse_field_annot<'input>(tokens: &mut Lexer<'input>) -> Result<(Field, Type), Error> {
    tokens.match_doc_comments();
    let f = parse_field(tokens)?;
    consume_token(tokens, Tok::Colon)?;
    let st = parse_type(tokens)?;
    Ok((f, st))
}

//**************************************************************************************************
// Constants
//**************************************************************************************************

// Parse a constant:
//      ConstantDecl = "const" <Identifier> ":" <Type> "=" <Exp> ";"
fn parse_constant<'input>(tokens: &mut Lexer<'input>) -> Result<Constant, Error> {
    tokens.match_doc_comments();
    let start_loc = tokens.start_loc();

    consume_token(tokens, Tok::Const)?;
    let name = ConstantName(parse_identifier(tokens)?);
    consume_token(tokens, Tok::Colon)?;
    let signature = parse_type(tokens)?;
    consume_token(tokens, Tok::Equal)?;
    let value = parse_exp(tokens)?;
    consume_token(tokens, Tok::Semicolon)?;
    let loc = make_loc(tokens.file_name(), start_loc, tokens.previous_end_loc());
    Ok(Constant {
        loc,
        name,
        signature,
        value,
    })
}

//**************************************************************************************************
// AddressBlock
//**************************************************************************************************

// Parse an address block:
//      AddressBlock =
//          "address" <Address> "{"
//              <Module>*
//          "}"
//
// Note that "address" is not a token.
fn parse_address_block<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<(Loc, Address, Vec<ModuleDefinition>), Error> {
    const UNEXPECTED_TOKEN: &str = "Invalid code unit. Expected 'address', 'module', or 'script'";
    if tokens.peek() != Tok::IdentifierValue {
        let start = tokens.start_loc();
        let end = start + tokens.content().len();
        let loc = make_loc(tokens.file_name(), start, end);
        return Err(vec![(
            loc,
            format!("{}. Got '{}'", UNEXPECTED_TOKEN, tokens.content()),
        )]);
    }
    let addr_name = parse_identifier(tokens)?;
    if addr_name.value != "address" {
        return Err(vec![(
            addr_name.loc,
            format!("{}. Got '{}'", UNEXPECTED_TOKEN, addr_name.value),
        )]);
    }
    let start_loc = tokens.start_loc();
    let addr = parse_address(tokens)?;
    let end_loc = tokens.previous_end_loc();
    let loc = make_loc(tokens.file_name(), start_loc, end_loc);

    consume_token(tokens, Tok::LBrace)?;
    let mut modules = vec![];
    while tokens.peek() != Tok::RBrace {
        modules.push(parse_module(tokens)?);
    }
    consume_token(tokens, Tok::RBrace)?;

    Ok((loc, addr, modules))
}

//**************************************************************************************************
// Modules
//**************************************************************************************************

// Parse a use declaration:
//      UseDecl =
//          "use" <ModuleIdent> <UseAlias> ";" |
//          "use" <ModuleIdent> :: <UseMember> ";" |
//          "use" <ModuleIdent> :: "{" Comma<UseMember> "}" ";"
fn parse_use_decl<'input>(tokens: &mut Lexer<'input>) -> Result<Use, Error> {
    consume_token(tokens, Tok::Use)?;
    let ident = parse_module_ident(tokens)?;
    let alias_opt = parse_use_alias(tokens)?;
    let use_ = match (&alias_opt, tokens.peek()) {
        (None, Tok::ColonColon) => {
            consume_token(tokens, Tok::ColonColon)?;
            let sub_uses = match tokens.peek() {
                Tok::LBrace => parse_comma_list(
                    tokens,
                    Tok::LBrace,
                    Tok::RBrace,
                    parse_use_member,
                    "a module member alias",
                )?,
                _ => vec![parse_use_member(tokens)?],
            };
            Use::Members(ident, sub_uses)
        }
        _ => Use::Module(ident, alias_opt.map(ModuleName)),
    };
    consume_token(tokens, Tok::Semicolon)?;
    Ok(use_)
}

// Parse an alias for a module member:
//      UseMember = <Identifier> <UseAlias>
fn parse_use_member<'input>(tokens: &mut Lexer<'input>) -> Result<(Name, Option<Name>), Error> {
    let member = parse_identifier(tokens)?;
    let alias_opt = parse_use_alias(tokens)?;
    Ok((member, alias_opt))
}

// Parse an 'as' use alias:
//      UseAlias = ("as" <Identifier>)?
fn parse_use_alias<'input>(tokens: &mut Lexer<'input>) -> Result<Option<Name>, Error> {
    Ok(if tokens.peek() == Tok::As {
        tokens.advance()?;
        Some(parse_identifier(tokens)?)
    } else {
        None
    })
}

// TODO rework parsing modifiers
fn is_struct_definition<'input>(tokens: &mut Lexer<'input>) -> Result<bool, Error> {
    let mut t = tokens.peek();
    if t == Tok::Native {
        t = tokens.lookahead()?;
    }
    Ok(t == Tok::Struct || t == Tok::Resource)
}

// Parse a module:
//      Module =
//          <DocComments> "module" <ModuleName> "{"
//              <UseDecl>*
//              ( <ConstantDecl> | <StructDefinition> | <FunctionDecl> | <Spec> )*
//          "}"
fn parse_module<'input>(tokens: &mut Lexer<'input>) -> Result<ModuleDefinition, Error> {
    tokens.match_doc_comments();
    let start_loc = tokens.start_loc();

    consume_token(tokens, Tok::Module)?;
    let name = parse_module_name(tokens)?;
    consume_token(tokens, Tok::LBrace)?;

    let mut members = vec![];
    while tokens.peek() != Tok::RBrace {
        members.push(match tokens.peek() {
            Tok::Spec => ModuleMember::Spec(parse_spec_block(tokens)?),
            Tok::Use => ModuleMember::Use(parse_use_decl(tokens)?),
            Tok::Const => ModuleMember::Constant(parse_constant(tokens)?),
            // TODO rework parsing modifiers
            _ if is_struct_definition(tokens)? => {
                ModuleMember::Struct(parse_struct_definition(tokens)?)
            }
            _ => ModuleMember::Function(parse_function_decl(tokens, /* allow_native */ true)?),
        })
    }
    consume_token(tokens, Tok::RBrace)?;

    let loc = make_loc(tokens.file_name(), start_loc, tokens.previous_end_loc());
    Ok(ModuleDefinition { loc, name, members })
}

//**************************************************************************************************
// Scripts
//**************************************************************************************************

// Parse a script:
//      Script =
//          "script" "{"
//              <UseDecl>*
//              <ConstantDecl>*
//              <MoveFunctionDecl>
//          "}"
fn parse_script<'input>(tokens: &mut Lexer<'input>) -> Result<Script, Error> {
    let start_loc = tokens.start_loc();

    consume_token(tokens, Tok::Script)?;
    consume_token(tokens, Tok::LBrace)?;

    let mut uses = vec![];
    while tokens.peek() == Tok::Use {
        uses.push(parse_use_decl(tokens)?);
    }
    let mut constants = vec![];
    while tokens.peek() == Tok::Const {
        constants.push(parse_constant(tokens)?);
    }
    let function = parse_function_decl(tokens, /* allow_native */ false)?;
    let mut specs = vec![];
    while tokens.peek() == Tok::Spec {
        specs.push(parse_spec_block(tokens)?)
    }

    if tokens.peek() != Tok::RBrace {
        let loc = current_token_loc(tokens);
        return Err(vec![(
            loc,
            "Unexpected characters after end of 'script' function".to_string(),
        )]);
    }
    consume_token(tokens, Tok::RBrace)?;

    let loc = make_loc(tokens.file_name(), start_loc, tokens.previous_end_loc());
    Ok(Script {
        loc,
        uses,
        constants,
        function,
        specs,
    })
}
//**************************************************************************************************
// Specification Blocks
//**************************************************************************************************

// Parse an optional specification block:
//     SpecBlockTarget =
//          "fun" <Identifier>
//        | "struct <Identifier>
//        | "module"
//        | "schema" <Identifier> <OptionalTypeParameters>
//        | <empty>
//     SpecBlock =
//        <DocComments> "spec" ( <SpecFunction> | <SpecBlockTarget> "{" SpecBlockMember* "}" )
fn parse_spec_block<'input>(tokens: &mut Lexer<'input>) -> Result<SpecBlock, Error> {
    tokens.match_doc_comments();
    let start_loc = tokens.start_loc();
    consume_token(tokens, Tok::Spec)?;
    let target_start_loc = tokens.start_loc();
    let target_end_loc = target_start_loc + tokens.content().len();
    if matches!(tokens.peek(), Tok::Define | Tok::Native) {
        // Special treatment of the 'spec [native] define ..` short form. This is mapped into an AST
        // which matches 'spec module { [native] define .. }`.
        let define = parse_spec_function(tokens)?;
        return Ok(spanned(
            tokens.file_name(),
            start_loc,
            tokens.previous_end_loc(),
            SpecBlock_ {
                target: spanned(
                    tokens.file_name(),
                    target_start_loc,
                    target_end_loc,
                    SpecBlockTarget_::Module,
                ),
                uses: vec![],
                members: vec![define],
            },
        ));
    }
    let target_ = match tokens.peek() {
        Tok::Fun => {
            tokens.advance()?;
            let name = FunctionName(parse_identifier(tokens)?);
            SpecBlockTarget_::Function(name)
        }
        Tok::Struct => {
            tokens.advance()?;
            let name = StructName(parse_identifier(tokens)?);
            SpecBlockTarget_::Structure(name)
        }
        Tok::Module => {
            tokens.advance()?;
            SpecBlockTarget_::Module
        }
        Tok::IdentifierValue if tokens.content() == "schema" => {
            tokens.advance()?;
            let name = parse_identifier(tokens)?;
            let type_parameters = parse_optional_type_parameters(tokens)?;
            SpecBlockTarget_::Schema(name, type_parameters)
        }
        Tok::LBrace => SpecBlockTarget_::Code,
        _ => {
            return Err(unexpected_token_error(
                tokens,
                "one of `module`, `struct`, `fun`, `schema`, or `{`",
            ));
        }
    };
    let target = spanned(
        tokens.file_name(),
        target_start_loc,
        match target_ {
            SpecBlockTarget_::Code => target_start_loc,
            _ => tokens.previous_end_loc(),
        },
        target_,
    );

    consume_token(tokens, Tok::LBrace)?;
    let mut uses = vec![];
    while tokens.peek() == Tok::Use {
        uses.push(parse_use_decl(tokens)?);
    }
    let mut members = vec![];
    while tokens.peek() != Tok::RBrace {
        members.push(parse_spec_block_member(tokens)?);
    }
    consume_token(tokens, Tok::RBrace)?;
    Ok(spanned(
        tokens.file_name(),
        start_loc,
        tokens.previous_end_loc(),
        SpecBlock_ {
            target,
            uses,
            members,
        },
    ))
}

// Parse a spec block member:
//    SpecBlockMember = <DocComments ( <Invariant> | <Condition> | <SpecFunction> | <SpecVariable>
//                                   | <SpecInclude> | <SpecApply> | <SpecPragma> | <SpecLet> )
fn parse_spec_block_member<'input>(tokens: &mut Lexer<'input>) -> Result<SpecBlockMember, Error> {
    tokens.match_doc_comments();
    match tokens.peek() {
        Tok::Invariant => parse_invariant(tokens),
        Tok::Let => parse_spec_let(tokens),
        Tok::Define | Tok::Native => parse_spec_function(tokens),
        Tok::IdentifierValue => match tokens.content() {
            "assert" | "assume" | "decreases" | "aborts_if" | "aborts_with" | "succeeds_if"
            | "modifies" | "ensures" | "requires" => parse_condition(tokens),
            "include" => parse_spec_include(tokens),
            "apply" => parse_spec_apply(tokens),
            "pragma" => parse_spec_pragma(tokens),
            "global" | "local" => parse_spec_variable(tokens),
            _ => {
                // local is optional but supported to be able to declare variables which are
                // named like the weak keywords above
                parse_spec_variable(tokens)
            }
        },
        _ => Err(unexpected_token_error(
            tokens,
            "one of `assert`, `assume`, `decreases`, `aborts_if`, `succeeds_if`, `modifies`, \
             `ensures`, `requires`, `include`, `apply`, `pragma`, `global`, or a name",
        )),
    }
}

// Parse a specification condition:
//    SpecCondition =
//        ("assert" | "assume" | "decreases" | "aborts_if" | "ensures" | "requires" )
//        <ConditionProperties> <Exp> ";"
//      | "aborts_with" <ConditionProperties> Comma<Exp> ";"
fn parse_condition<'input>(tokens: &mut Lexer<'input>) -> Result<SpecBlockMember, Error> {
    let start_loc = tokens.start_loc();
    let kind = match tokens.content() {
        "assert" => SpecConditionKind::Assert,
        "assume" => SpecConditionKind::Assume,
        "decreases" => SpecConditionKind::Decreases,
        "aborts_if" => SpecConditionKind::AbortsIf,
        "aborts_with" => SpecConditionKind::AbortsWith,
        "succeeds_if" => SpecConditionKind::SucceedsIf,
        "modifies" => SpecConditionKind::Modifies,
        "ensures" => SpecConditionKind::Ensures,
        "requires" => {
            if tokens.lookahead()? == Tok::Module {
                tokens.advance()?;
                SpecConditionKind::RequiresModule
            } else {
                SpecConditionKind::Requires
            }
        }
        _ => unreachable!(),
    };
    tokens.advance()?;
    let properties = parse_condition_properties(tokens)?;
    let exp = if kind == SpecConditionKind::AbortsWith || kind == SpecConditionKind::Modifies {
        // Use a dummy expression as a placeholder for this field.
        let loc = make_loc(tokens.file_name(), start_loc, start_loc + 1);
        sp(loc, Exp_::Value(sp(loc, Value_::Bool(false))))
    } else {
        parse_exp(tokens)?
    };
    let additional_exps = if kind == SpecConditionKind::AbortsIf
        && tokens.peek() == Tok::IdentifierValue
        && tokens.content() == "with"
    {
        tokens.advance()?;
        let codes = vec![parse_exp(tokens)?];
        consume_token(tokens, Tok::Semicolon)?;
        codes
    } else if kind == SpecConditionKind::AbortsWith || kind == SpecConditionKind::Modifies {
        parse_comma_list_after_start(
            tokens,
            tokens.start_loc(),
            tokens.peek(),
            Tok::Semicolon,
            parse_exp,
            "an aborts code or modifies target",
        )?
    } else {
        consume_token(tokens, Tok::Semicolon)?;
        vec![]
    };
    let end_loc = tokens.previous_end_loc();
    Ok(spanned(
        tokens.file_name(),
        start_loc,
        end_loc,
        SpecBlockMember_::Condition {
            kind,
            properties,
            exp,
            additional_exps,
        },
    ))
}

// Parse properties in a condition.
//   ConditionProperties = ( "[" Comma<SpecPragmaProperty> "]" )?
fn parse_condition_properties<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<Vec<PragmaProperty>, Error> {
    let properties = if tokens.peek() == Tok::LBracket {
        parse_comma_list(
            tokens,
            Tok::LBracket,
            Tok::RBracket,
            parse_spec_property,
            "a condition property",
        )?
    } else {
        vec![]
    };
    Ok(properties)
}

// Parse an invariant:
//     Invariant = "invariant" ( "update" | "pack" | "unpack" | "module" )?
//                 <ConditionProperties> <Exp> ";"
fn parse_invariant<'input>(tokens: &mut Lexer<'input>) -> Result<SpecBlockMember, Error> {
    let start_loc = tokens.start_loc();
    consume_token(tokens, Tok::Invariant)?;
    let kind = match tokens.peek() {
        Tok::IdentifierValue => {
            // The update/pack/unpack modifiers are 'weak' keywords. They are reserved
            // only when following an "invariant" token. One can use "invariant (update ...)" to
            // force interpretation as identifiers in expressions.
            match tokens.content() {
                "update" => {
                    tokens.advance()?;
                    SpecConditionKind::InvariantUpdate
                }
                "pack" => {
                    tokens.advance()?;
                    SpecConditionKind::InvariantPack
                }
                "unpack" => {
                    tokens.advance()?;
                    SpecConditionKind::InvariantUnpack
                }
                _ => SpecConditionKind::Invariant,
            }
        }
        Tok::Module => {
            tokens.advance()?;
            SpecConditionKind::InvariantModule
        }
        _ => SpecConditionKind::Invariant,
    };
    let properties = parse_condition_properties(tokens)?;
    let exp = parse_exp(tokens)?;
    consume_token(tokens, Tok::Semicolon)?;
    Ok(spanned(
        tokens.file_name(),
        start_loc,
        tokens.previous_end_loc(),
        SpecBlockMember_::Condition {
            kind,
            properties,
            exp,
            additional_exps: vec![],
        },
    ))
}

// Parse a specification function.
//     SpecFunction = "define" <SpecFunctionSignature> ( "{" <Sequence> "}" | ";" )
//                  | "native" "define" <SpecFunctionSignature> ";"
//     SpecFunctionSignature =
//         <Identifier> <OptionalTypeParameters> "(" Comma<Parameter> ")" ":" <Type>
fn parse_spec_function<'input>(tokens: &mut Lexer<'input>) -> Result<SpecBlockMember, Error> {
    let start_loc = tokens.start_loc();
    let native_opt = consume_optional_token_with_loc(tokens, Tok::Native)?;
    consume_token(tokens, Tok::Define)?;
    let name = FunctionName(parse_identifier(tokens)?);
    let type_parameters = parse_optional_type_parameters(tokens)?;
    // "(" Comma<Parameter> ")"
    let parameters = parse_comma_list(
        tokens,
        Tok::LParen,
        Tok::RParen,
        parse_parameter,
        "a function parameter",
    )?;

    // ":" <Type>)
    consume_token(tokens, Tok::Colon)?;
    let return_type = parse_type(tokens)?;

    let body_start_loc = tokens.start_loc();
    let no_body = tokens.peek() != Tok::LBrace;
    let (uninterpreted, body_) = if native_opt.is_some() || no_body {
        consume_token(tokens, Tok::Semicolon)?;
        (native_opt.is_none(), FunctionBody_::Native)
    } else {
        consume_token(tokens, Tok::LBrace)?;
        let seq = parse_sequence(tokens)?;
        (false, FunctionBody_::Defined(seq))
    };
    let body = spanned(
        tokens.file_name(),
        body_start_loc,
        tokens.previous_end_loc(),
        body_,
    );

    let signature = FunctionSignature {
        type_parameters,
        parameters,
        return_type,
    };

    Ok(spanned(
        tokens.file_name(),
        start_loc,
        tokens.previous_end_loc(),
        SpecBlockMember_::Function {
            signature,
            uninterpreted,
            name,
            body,
        },
    ))
}

// Parse a specification variable.
//     SpecVariable = ( "global" | "local" )? <Identifier> <OptionalTypeParameters> ":" <Type> ";"
fn parse_spec_variable<'input>(tokens: &mut Lexer<'input>) -> Result<SpecBlockMember, Error> {
    let start_loc = tokens.start_loc();
    let is_global = match tokens.content() {
        "global" => {
            consume_token(tokens, Tok::IdentifierValue)?;
            true
        }
        "local" => {
            consume_token(tokens, Tok::IdentifierValue)?;
            false
        }
        _ => false,
    };
    let name = parse_identifier(tokens)?;
    let type_parameters = parse_optional_type_parameters(tokens)?;
    consume_token(tokens, Tok::Colon)?;
    let type_ = parse_type(tokens)?;
    consume_token(tokens, Tok::Semicolon)?;
    Ok(spanned(
        tokens.file_name(),
        start_loc,
        tokens.previous_end_loc(),
        SpecBlockMember_::Variable {
            is_global,
            name,
            type_parameters,
            type_,
        },
    ))
}

// Parse a specification let.
//     SpecLet =  "let" <Identifier> "=" <Exp> ";"
fn parse_spec_let<'input>(tokens: &mut Lexer<'input>) -> Result<SpecBlockMember, Error> {
    let start_loc = tokens.start_loc();
    tokens.advance()?;
    let name = parse_identifier(tokens)?;
    consume_token(tokens, Tok::Equal)?;
    let def = parse_exp(tokens)?;
    consume_token(tokens, Tok::Semicolon)?;
    Ok(spanned(
        tokens.file_name(),
        start_loc,
        tokens.previous_end_loc(),
        SpecBlockMember_::Let { name, def },
    ))
}

// Parse a specification schema include.
//    SpecInclude = "include" <Exp>
fn parse_spec_include<'input>(tokens: &mut Lexer<'input>) -> Result<SpecBlockMember, Error> {
    let start_loc = tokens.start_loc();
    consume_identifier(tokens, "include")?;
    let properties = parse_condition_properties(tokens)?;
    let exp = parse_exp(tokens)?;
    consume_token(tokens, Tok::Semicolon)?;
    Ok(spanned(
        tokens.file_name(),
        start_loc,
        tokens.previous_end_loc(),
        SpecBlockMember_::Include { properties, exp },
    ))
}

// Parse a specification schema apply.
//    SpecApply = "apply" <Exp> "to" Comma<SpecApplyPattern>
//                                   ( "except" Comma<SpecApplyPattern> )? ";"
fn parse_spec_apply<'input>(tokens: &mut Lexer<'input>) -> Result<SpecBlockMember, Error> {
    let start_loc = tokens.start_loc();
    consume_identifier(tokens, "apply")?;
    let exp = parse_exp(tokens)?;
    consume_identifier(tokens, "to")?;
    let parse_patterns = |tokens: &mut Lexer<'input>| {
        parse_list(
            tokens,
            |tokens| {
                if tokens.peek() == Tok::Comma {
                    tokens.advance()?;
                    Ok(true)
                } else {
                    Ok(false)
                }
            },
            parse_spec_apply_pattern,
        )
    };
    let patterns = parse_patterns(tokens)?;
    let exclusion_patterns =
        if tokens.peek() == Tok::IdentifierValue && tokens.content() == "except" {
            tokens.advance()?;
            parse_patterns(tokens)?
        } else {
            vec![]
        };
    consume_token(tokens, Tok::Semicolon)?;
    Ok(spanned(
        tokens.file_name(),
        start_loc,
        tokens.previous_end_loc(),
        SpecBlockMember_::Apply {
            exp,
            patterns,
            exclusion_patterns,
        },
    ))
}

// Parse a function pattern:
//     SpecApplyPattern = <SpecApplyFragment>+ <OptionalTypeArgs>
fn parse_spec_apply_pattern<'input>(tokens: &mut Lexer<'input>) -> Result<SpecApplyPattern, Error> {
    let start_loc = tokens.start_loc();
    let public_opt = consume_optional_token_with_loc(tokens, Tok::Public)?;
    let visibility = if let Some(loc) = public_opt {
        Some(FunctionVisibility::Public(loc))
    } else if tokens.peek() == Tok::IdentifierValue && tokens.content() == "internal" {
        // Its not ideal right now that we do not have a loc here, but acceptable for what
        // we are doing with this in specs.
        tokens.advance()?;
        Some(FunctionVisibility::Internal)
    } else {
        None
    };
    let mut last_end = tokens.start_loc() + tokens.content().len();
    let name_pattern = parse_list(
        tokens,
        |tokens| {
            // We need name fragments followed by each other without space. So we do some
            // magic here similar as with `>>` based on token distance.
            let start_loc = tokens.start_loc();
            let adjacent = last_end == start_loc;
            last_end = start_loc + tokens.content().len();
            Ok(adjacent && [Tok::IdentifierValue, Tok::Star].contains(&tokens.peek()))
        },
        parse_spec_apply_fragment,
    )?;
    let type_parameters = parse_optional_type_parameters(tokens)?;
    Ok(spanned(
        tokens.file_name(),
        start_loc,
        tokens.previous_end_loc(),
        SpecApplyPattern_ {
            visibility,
            name_pattern,
            type_parameters,
        },
    ))
}

// Parse a name pattern fragment
//     SpecApplyFragment = <Identifier> | "*"
fn parse_spec_apply_fragment<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<SpecApplyFragment, Error> {
    let start_loc = tokens.start_loc();
    let fragment = match tokens.peek() {
        Tok::IdentifierValue => SpecApplyFragment_::NamePart(parse_identifier(tokens)?),
        Tok::Star => {
            tokens.advance()?;
            SpecApplyFragment_::Wildcard
        }
        _ => return Err(unexpected_token_error(tokens, "a name fragment or `*`")),
    };
    Ok(spanned(
        tokens.file_name(),
        start_loc,
        tokens.previous_end_loc(),
        fragment,
    ))
}

// Parse a specification pragma:
//    SpecPragma = "pragma" Comma<SpecPragmaProperty> ";"
fn parse_spec_pragma<'input>(tokens: &mut Lexer<'input>) -> Result<SpecBlockMember, Error> {
    let start_loc = tokens.start_loc();
    consume_identifier(tokens, "pragma")?;
    let properties = parse_comma_list_after_start(
        tokens,
        start_loc,
        Tok::IdentifierValue,
        Tok::Semicolon,
        parse_spec_property,
        "a pragma property",
    )?;
    Ok(spanned(
        tokens.file_name(),
        start_loc,
        tokens.previous_end_loc(),
        SpecBlockMember_::Pragma { properties },
    ))
}

// Parse a specification pragma property:
//    SpecPragmaProperty = <Identifier> ( "=" Value )?
fn parse_spec_property<'input>(tokens: &mut Lexer<'input>) -> Result<PragmaProperty, Error> {
    let start_loc = tokens.start_loc();
    let name = parse_identifier(tokens)?;
    let value = if tokens.peek() == Tok::Equal {
        tokens.advance()?;
        match tokens.peek() {
            Tok::True
            | Tok::False
            | Tok::U8Value
            | Tok::U64Value
            | Tok::U128Value
            | Tok::ByteStringValue
            | Tok::AddressValue => Some(parse_value(tokens)?),
            Tok::NumValue => {
                let i = parse_num(tokens)?;
                Some(spanned(
                    tokens.file_name(),
                    start_loc,
                    tokens.previous_end_loc(),
                    Value_::U128(i),
                ))
            }
            _ => return Err(unexpected_token_error(tokens, "a value")),
        }
    } else {
        None
    };
    Ok(spanned(
        tokens.file_name(),
        start_loc,
        tokens.previous_end_loc(),
        PragmaProperty_ { name, value },
    ))
}

//**************************************************************************************************
// File
//**************************************************************************************************

// Parse a file:
//      File =
//          (<AddressBlock> | <Module> | <Script>)*
fn parse_file<'input>(tokens: &mut Lexer<'input>) -> Result<Vec<Definition>, Error> {
    let mut defs = vec![];
    while tokens.peek() != Tok::EOF {
        defs.push(match tokens.peek() {
            Tok::Module => Definition::Module(parse_module(tokens)?),
            Tok::Script => Definition::Script(parse_script(tokens)?),
            _ => {
                let (loc, addr, modules) = parse_address_block(tokens)?;
                Definition::Address(loc, addr, modules)
            }
        })
    }
    Ok(defs)
}

/// Parse the `input` string as a file of Move source code and return the
/// result as either a pair of FileDefinition and doc comments or some Errors. The `file` name
/// is used to identify source locations in error messages.
pub fn parse_file_string(
    file: &'static str,
    input: &str,
    comment_map: BTreeMap<Span, String>,
) -> Result<(Vec<Definition>, BTreeMap<ByteIndex, String>), Errors> {
    let mut tokens = Lexer::new(input, file, comment_map);
    match tokens.advance() {
        Err(err) => Err(vec![err]),
        Ok(..) => Ok(()),
    }?;
    match parse_file(&mut tokens) {
        Err(err) => Err(vec![err]),
        Ok(def) => {
            let doc_comments = tokens.check_and_get_doc_comments()?;
            Ok((def, doc_comments))
        }
    }
}
