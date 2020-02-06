// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use codespan::{ByteIndex, Span};
use std::str::FromStr;

use crate::errors::*;
use crate::parser::ast::*;
use crate::parser::lexer::*;
use crate::shared::*;

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

fn make_loc(file: &'static str, start: usize, end: usize) -> Loc {
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
    if match_token(tokens, end_token)? {
        return Ok(vec![]);
    }
    let mut v = vec![];
    loop {
        if tokens.peek() == Tok::Comma || tokens.peek() == end_token {
            let current_loc = tokens.start_loc();
            let loc = make_loc(tokens.file_name(), current_loc, current_loc);
            return Err(vec![(loc, format!("Expected {}", item_description))]);
        }
        v.push(parse_list_item(tokens)?);
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
        if match_token(tokens, end_token)? {
            break Ok(v);
        }
    }
}

//**************************************************************************************************
// Names and Addresses
//**************************************************************************************************

// Parse a name:
//      Name = <NameValue>
fn parse_name<'input>(tokens: &mut Lexer<'input>) -> Result<Spanned<String>, Error> {
    if tokens.peek() != Tok::NameValue {
        return Err(unexpected_token_error(tokens, "a name value"));
    }
    let start_loc = tokens.start_loc();
    let name = tokens.content().to_string();
    tokens.advance()?;
    let end_loc = tokens.previous_end_loc();
    Ok(spanned(tokens.file_name(), start_loc, end_loc, name))
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
//      Var = <Name>
fn parse_var<'input>(tokens: &mut Lexer<'input>) -> Result<Var, Error> {
    Ok(Var(parse_name(tokens)?))
}

// Parse a field name:
//      Field = <Name>
fn parse_field<'input>(tokens: &mut Lexer<'input>) -> Result<Field, Error> {
    Ok(Field(parse_name(tokens)?))
}

// Parse a module name:
//      ModuleName = <Name>
fn parse_module_name<'input>(tokens: &mut Lexer<'input>) -> Result<ModuleName, Error> {
    Ok(ModuleName(parse_name(tokens)?))
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

// Parse a module access (either a struct or a function):
//      ModuleAccess =
//          <Name>
//          | <ModuleName> "::" <Name>
//          | <ModuleIdent> "::" <Name>
fn parse_module_access<'input, F: FnOnce() -> String>(
    tokens: &mut Lexer<'input>,
    item_description: F,
) -> Result<ModuleAccess, Error> {
    let start_loc = tokens.start_loc();
    let acc = match tokens.peek() {
        Tok::NameValue => {
            // Check if this is a ModuleName followed by "::".
            let m = parse_name(tokens)?;
            if match_token(tokens, Tok::ColonColon)? {
                let n = parse_name(tokens)?;
                ModuleAccess_::ModuleAccess(ModuleName(m), n)
            } else {
                ModuleAccess_::Name(m)
            }
        }

        Tok::AddressValue => {
            let m = parse_module_ident(tokens)?;
            consume_token(tokens, Tok::ColonColon)?;
            let n = parse_name(tokens)?;
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
        sp(f.loc(), Exp_::Name(f.0.clone()))
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
//          | <ModuleAccess> ("<" Comma<BaseType> ">")? "{" Comma<BindField> "}"
fn parse_bind<'input>(tokens: &mut Lexer<'input>) -> Result<Bind, Error> {
    let start_loc = tokens.start_loc();
    if tokens.peek() == Tok::NameValue {
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
    let ty_args = if tokens.peek() == Tok::Less {
        Some(parse_comma_list(
            tokens,
            Tok::Less,
            Tok::Greater,
            parse_base_type,
            "a type",
        )?)
    } else {
        None
    };
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

//**************************************************************************************************
// Values
//**************************************************************************************************

// Parse a value:
//      Value =
//          <Address>
//          | "true"
//          | "false"
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
        _ => unreachable!("parse_value called with invalid token"),
    };
    let end_loc = tokens.previous_end_loc();
    Ok(spanned(tokens.file_name(), start_loc, end_loc, val))
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
//      Sequence = (<SequenceItem> ";")* <Exp>? "}"
//
// Note that this does not include the opening brace of a block but it
// does consume the closing right brace.
fn parse_sequence<'input>(tokens: &mut Lexer<'input>) -> Result<Sequence, Error> {
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
    Ok((seq, last_semicolon_loc, Box::new(eopt)))
}

//**************************************************************************************************
// Expressions
//**************************************************************************************************

// Parse an expression term:
//      Term =
//          "move" <Var>
//          | "copy" <Var>
//          | "break"
//          | "continue"
//          | <Name>
//          | <Value>
//          | "(" Comma<Exp> ")"
//          | "(" <Exp> ":" <Type> ")"
//          | "{" <Sequence>
//          | <ModuleAccess> ("<" Comma<BaseType> ">")? "{" Comma<ExpField> "}"
//          | <ModuleAccess> ("<" Comma<BaseType> ">")? "(" Comma<Exp> ")"
//          | "::" <Name> ("<" Comma<BaseType> ">")? "(" Comma<Exp> ")"
fn parse_term<'input>(tokens: &mut Lexer<'input>) -> Result<Exp, Error> {
    let start_loc = tokens.start_loc();
    let term = match tokens.peek() {
        Tok::Move => {
            tokens.advance()?;
            Exp_::Move(parse_var(tokens)?)
        }

        Tok::Copy => {
            tokens.advance()?;
            Exp_::Copy(parse_var(tokens)?)
        }

        Tok::Break => {
            tokens.advance()?;
            Exp_::Break
        }

        Tok::Continue => {
            tokens.advance()?;
            Exp_::Continue
        }

        Tok::NameValue => {
            // Check if this is a ModuleAccess for a pack or call expression.
            match tokens.lookahead()? {
                Tok::ColonColon | Tok::LBrace | Tok::LParen => parse_pack_or_call(tokens)?,
                Tok::Less => {
                    // There's an ambiguity here. If there is no whitespace after the
                    // name, treat it as the start of a list of type arguments. Otherwise
                    // assume that the "<" is a boolean operator.
                    let next_start = tokens.lookahead_start_loc();
                    if next_start == start_loc + tokens.content().len() {
                        match parse_pack_or_call(tokens) {
                            Ok(x) => x,
                            Err(mut e) => {
                                let loc = make_loc(tokens.file_name(), next_start, next_start);
                                let msg =
                                    "Perhaps you need a blank space before this '<' operator?";
                                e.push((loc, msg.to_owned()));
                                return Err(e);
                            }
                        }
                    } else {
                        Exp_::Name(parse_name(tokens)?)
                    }
                }
                _ => Exp_::Name(parse_name(tokens)?),
            }
        }

        Tok::AddressValue => {
            // Check if this is a ModuleIdent (in a ModuleAccess).
            if tokens.lookahead()? == Tok::ColonColon {
                parse_pack_or_call(tokens)?
            } else {
                Exp_::Value(parse_value(tokens)?)
            }
        }

        Tok::True | Tok::False => Exp_::Value(parse_value(tokens)?),
        Tok::NumValue => {
            let i = match u128::from_str(tokens.content()) {
                Ok(i) => i,
                Err(_) => {
                    let end_loc = start_loc + tokens.content().len();
                    let loc = make_loc(tokens.file_name(), start_loc, end_loc);
                    let msg =
                        "Invalid number literal. The given literal is too large to fit into the \
                         largest number type 'u128'";
                    return Err(vec![(loc, msg.to_owned())]);
                }
            };
            tokens.advance()?;
            Exp_::InferredNum(i)
        }

        // "(" Comma<Exp> ")"
        // "(" <Exp> ":" <Type> ")"
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

        // "::" <Name> ("<" Comma<BaseType> ">")? "(" Comma<Exp> ")"
        Tok::ColonColon => {
            tokens.advance()?; // consume the "::"
            let n = parse_name(tokens)?;
            let tys = if tokens.peek() == Tok::Less {
                Some(parse_comma_list(
                    tokens,
                    Tok::Less,
                    Tok::Greater,
                    parse_base_type,
                    "a type",
                )?)
            } else {
                None
            };
            let rhs = parse_call_args(tokens)?;
            Exp_::GlobalCall(n, tys, rhs)
        }

        _ => {
            return Err(unexpected_token_error(tokens, "an expression term"));
        }
    };
    let end_loc = tokens.previous_end_loc();
    Ok(spanned(tokens.file_name(), start_loc, end_loc, term))
}

// Parse the subset of expression terms for pack and call operations.
// This is a helper function for parse_term.
fn parse_pack_or_call<'input>(tokens: &mut Lexer<'input>) -> Result<Exp_, Error> {
    let n = parse_module_access(tokens, || {
        panic!("parse_pack_or_call with something other than a NameValue or AddressValue token")
    })?;
    let tys = if tokens.peek() == Tok::Less {
        Some(parse_comma_list(
            tokens,
            Tok::Less,
            Tok::Greater,
            parse_base_type,
            "a type",
        )?)
    } else {
        None
    };
    match tokens.peek() {
        // <ModuleAccess> ("<" Comma<BaseType> ">")? "{" Comma<ExpField> "}"
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

        // <ModuleAccess> ("<" Comma<BaseType> ">")? "(" Comma<Exp> ")"
        Tok::LParen => {
            let rhs = parse_call_args(tokens)?;
            Ok(Exp_::Call(n, tys, rhs))
        }

        _ => {
            let expected = "either a brace-enclosed list of field expressions or \
                            a parenthesized list of arguments for a function call";
            Err(unexpected_token_error(tokens, expected))
        }
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

// Parse an expression:
//      Exp =
//          "if" "(" <Exp> ")" <Exp> ("else" <Exp>)?
//          | "while" "(" <Exp> ")" <Exp>
//          | "loop" <Exp>
//          | <UnaryExp> "=" <Exp>
//          | "return" <Exp>
//          | "abort" <Exp>
//          | <BinOpExp>
fn parse_exp<'input>(tokens: &mut Lexer<'input>) -> Result<Exp, Error> {
    let start_loc = tokens.start_loc();
    let exp = match tokens.peek() {
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
            let e = Box::new(parse_exp(tokens)?);
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
        Tok::PipePipe => 2,
        Tok::AmpAmp => 3,
        Tok::EqualEqual => 4,
        Tok::ExclaimEqual => 4,
        Tok::Less => 4,
        Tok::Greater => 4,
        Tok::LessEqual => 4,
        Tok::GreaterEqual => 4,
        Tok::Pipe => 5,
        Tok::Caret => 6,
        Tok::Amp => 7,
        Tok::Plus => 8,
        Tok::Minus => 8,
        Tok::Star => 9,
        Tok::Slash => 9,
        Tok::Percent => 9,
        _ => 0, // anything else is not a binary operator
    }
}

// Parse a binary operator expression:
//      BinOpExp =
//          <UnaryExp> <BinOp> <BinOpExp>
//      BinOp = (listed from lowest to highest precedence)
//          "||"
//          | "&&"
//          | "==" | "!=" | "<" | ">" | "<=" | ">="
//          | "|"
//          | "^"
//          | "&"
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
            Tok::Plus => BinOp_::Add,
            Tok::Minus => BinOp_::Sub,
            Tok::Star => BinOp_::Mul,
            Tok::Slash => BinOp_::Div,
            Tok::Percent => BinOp_::Mod,
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
//          | "-" <UnaryExp>
//          | "&mut" <UnaryExp>
//          | "&" <UnaryExp>
//          | "*" <UnaryExp>
//          | <DotChain>
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
        Tok::Minus => {
            tokens.advance()?;
            let op_end_loc = tokens.previous_end_loc();
            let op = spanned(tokens.file_name(), start_loc, op_end_loc, UnaryOp_::Neg);
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
        _ => {
            return parse_dot_chain(tokens);
        }
    };
    let end_loc = tokens.previous_end_loc();
    Ok(spanned(tokens.file_name(), start_loc, end_loc, exp))
}

// Parse an expression term optionally followed by a chain of dot accesses:
//      DotChain =
//          <DotChain> "." <Name>
//          | <Term>
fn parse_dot_chain<'input>(tokens: &mut Lexer<'input>) -> Result<Exp, Error> {
    let start_loc = tokens.start_loc();
    let mut lhs = parse_term(tokens)?;

    while match_token(tokens, Tok::Period)? {
        let n = parse_name(tokens)?;
        let exp = Exp_::Dot(Box::new(lhs), n);
        let end_loc = tokens.previous_end_loc();
        lhs = spanned(tokens.file_name(), start_loc, end_loc, exp);
    }
    Ok(lhs)
}

//**************************************************************************************************
// Types
//**************************************************************************************************

// Parse a base type:
//      BaseType = <ModuleAccess> ("<" Comma<BaseType> ">")?
fn parse_base_type<'input>(tokens: &mut Lexer<'input>) -> Result<SingleType, Error> {
    let start_loc = tokens.start_loc();
    let tn = parse_module_access(tokens, || "a type name".to_string())?;
    let tys = if tokens.peek() == Tok::Less {
        parse_comma_list(tokens, Tok::Less, Tok::Greater, parse_base_type, "a type")?
    } else {
        vec![]
    };
    let end_loc = tokens.previous_end_loc();
    let t = SingleType_::Apply(tn, tys);
    Ok(spanned(tokens.file_name(), start_loc, end_loc, t))
}

// Parse a SingleType:
//      SingleType =
//          <BaseType>
//          | "&" <BaseType>
//          | "&mut" <BaseType>
fn parse_single_type<'input>(tokens: &mut Lexer<'input>) -> Result<SingleType, Error> {
    let start_loc = tokens.start_loc();
    let (b, mutable) = match tokens.peek() {
        Tok::Amp => {
            tokens.advance()?;
            (parse_base_type(tokens)?, false)
        }
        Tok::AmpMut => {
            tokens.advance()?;
            (parse_base_type(tokens)?, true)
        }
        _ => {
            return parse_base_type(tokens);
        }
    };
    let end_loc = tokens.previous_end_loc();
    let t = SingleType_::Ref(mutable, Box::new(b));
    Ok(spanned(tokens.file_name(), start_loc, end_loc, t))
}

// Parse a type:
//      Type =
//          <SingleType>
//          | "(" Comma<SingleType> ")"
fn parse_type<'input>(tokens: &mut Lexer<'input>) -> Result<Type, Error> {
    let start_loc = tokens.start_loc();
    let mut ts = if tokens.peek() == Tok::LParen {
        parse_comma_list(
            tokens,
            Tok::LParen,
            Tok::RParen,
            parse_single_type,
            "a type",
        )?
    } else {
        vec![parse_single_type(tokens)?]
    };
    let t = match ts.len() {
        0 => Type_::Unit,
        1 => Type_::Single(ts.pop().unwrap()),
        _ => Type_::Multiple(ts),
    };
    let end_loc = tokens.previous_end_loc();
    Ok(spanned(tokens.file_name(), start_loc, end_loc, t))
}

// Parse a type parameter:
//      TypeParameter =
//          <Name> <Constraint>?
//      Constraint =
//          ":" "copyable"
//          | ":" "resource"
fn parse_type_parameter<'input>(tokens: &mut Lexer<'input>) -> Result<(Name, Kind), Error> {
    let n = parse_name(tokens)?;

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

//**************************************************************************************************
// Functions
//**************************************************************************************************

// Parse a function declaration:
//      FunctionDecl =
//          <NativeFunctionDecl>
//          | <MoveFunctionDecl>
//      NativeFunctionDecl =
//          "native" "public"? "fun"
//          <FunctionDefName> "(" Comma<Parameter> ")"
//          (":" <Type>)?
//          ("acquires" <ModuleAccess> ("," <ModuleAccess>)*)?
//          ";"
//      MoveFunctionDecl =
//          "public"? "fun"
//          <FunctionDefName> "(" Comma<Parameter> ")"
//          (":" <Type>)?
//          ("acquires" <ModuleAccess> ("," <ModuleAccess>)*)?
//          "{" <Sequence>
//      FunctionDefName =
//          <Name>
//          | <Name> "<" Comma<TypeParameter> ">"
//
// If the "allow_native" parameter is false, this will only accept Move
// functions.
fn parse_function_decl<'input>(
    tokens: &mut Lexer<'input>,
    allow_native: bool,
) -> Result<Function, Error> {
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

    // <Public>?
    let public_opt = consume_optional_token_with_loc(tokens, Tok::Public)?;
    let visibility = if let Some(loc) = public_opt {
        FunctionVisibility::Public(loc)
    } else {
        FunctionVisibility::Internal
    };

    // "fun" <FunctionDefName>
    consume_token(tokens, Tok::Fun)?;
    let name = FunctionName(parse_name(tokens)?);
    let type_parameters = if tokens.peek() == Tok::Less {
        parse_comma_list(
            tokens,
            Tok::Less,
            Tok::Greater,
            parse_type_parameter,
            "a type parameter",
        )?
    } else {
        vec![]
    };

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
        loop {
            acquires.push(parse_module_access(tokens, || {
                "a resource struct name".to_string()
            })?);
            if tokens.peek() == Tok::Semicolon || tokens.peek() == Tok::LBrace {
                break;
            }
            consume_token(tokens, Tok::Comma)?;
            if tokens.peek() == Tok::Semicolon || tokens.peek() == Tok::LBrace {
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

    Ok(Function {
        visibility,
        signature,
        acquires,
        name,
        body,
    })
}

// Parse a function parameter:
//      Parameter = <Var> ":" <SingleType>
fn parse_parameter<'input>(tokens: &mut Lexer<'input>) -> Result<(Var, SingleType), Error> {
    let v = parse_var(tokens)?;
    consume_token(tokens, Tok::Colon)?;
    let t = parse_single_type(tokens)?;
    Ok((v, t))
}

//**************************************************************************************************
// Structs
//**************************************************************************************************

// Parse a struct definition:
//      StructDefinition =
//          "resource"? "struct" <StructDefName> "{" Comma<FieldAnnot> "}"
//          | "native" "resource"? "struct" <StructDefName> ";"
//      StructDefName =
//          <Name>
//          | <Name> "<" Comma<TypeParameter> ">"
fn parse_struct_definition<'input>(tokens: &mut Lexer<'input>) -> Result<StructDefinition, Error> {
    // Record the source location of the "native" keyword (if there is one).
    let native_opt = consume_optional_token_with_loc(tokens, Tok::Native)?;

    // Record the source location of the "resource" keyword (if there is one).
    let resource_opt = consume_optional_token_with_loc(tokens, Tok::Resource)?;

    consume_token(tokens, Tok::Struct)?;

    // <StructDefName>
    let name = StructName(parse_name(tokens)?);
    let type_parameters = if tokens.peek() == Tok::Less {
        parse_comma_list(
            tokens,
            Tok::Less,
            Tok::Greater,
            parse_type_parameter,
            "a type parameter",
        )?
    } else {
        vec![]
    };

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

    Ok(StructDefinition {
        resource_opt,
        name,
        type_parameters,
        fields,
    })
}

// Parse a field annotated with a type:
//      FieldAnnot = <Field> ":" <SingleType>
fn parse_field_annot<'input>(tokens: &mut Lexer<'input>) -> Result<(Field, SingleType), Error> {
    let f = parse_field(tokens)?;
    consume_token(tokens, Tok::Colon)?;
    let st = parse_single_type(tokens)?;
    Ok((f, st))
}

//**************************************************************************************************
// Modules
//**************************************************************************************************

// Parse a use declaration:
//      UseDecl = "use" <ModuleIdent> ("as" <ModuleName>)? ";"
fn parse_use_decl<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<(ModuleIdent, Option<ModuleName>), Error> {
    consume_token(tokens, Tok::Use)?;
    let ident = parse_module_ident(tokens)?;
    let alias = if tokens.peek() == Tok::As {
        tokens.advance()?;
        Some(parse_module_name(tokens)?)
    } else {
        None
    };
    consume_token(tokens, Tok::Semicolon)?;
    Ok((ident, alias))
}

fn is_struct_definition<'input>(tokens: &mut Lexer<'input>) -> Result<bool, Error> {
    let mut t = tokens.peek();
    if t == Tok::Native {
        t = tokens.lookahead()?;
    }
    Ok(t == Tok::Struct || t == Tok::Resource)
}

// Parse a module:
//      Module =
//          "module" <ModuleName> "{"
//              <UseDecl>*
//              <StructDefinition>*
//              <FunctionDecl>*
//          "}"
fn parse_module<'input>(tokens: &mut Lexer<'input>) -> Result<ModuleDefinition, Error> {
    consume_token(tokens, Tok::Module)?;
    let name = parse_module_name(tokens)?;
    consume_token(tokens, Tok::LBrace)?;

    let mut uses = vec![];
    while tokens.peek() == Tok::Use {
        uses.push(parse_use_decl(tokens)?);
    }

    let mut structs = vec![];
    while is_struct_definition(tokens)? {
        structs.push(parse_struct_definition(tokens)?);
    }

    let mut functions = vec![];
    while tokens.peek() != Tok::RBrace {
        functions.push(parse_function_decl(tokens, /* allow_native */ true)?);
    }
    tokens.advance()?; // consume the RBrace

    Ok(ModuleDefinition {
        uses,
        name,
        structs,
        functions,
    })
}

//**************************************************************************************************
// File
//**************************************************************************************************

// Parse a file:
//      File =
//          (("address" <Address> ":") | <Module>)*
//          | <UseDecl>* <MoveFunctionDecl>
//
// Note that "address" is not a token.
fn parse_file<'input>(tokens: &mut Lexer<'input>) -> Result<FileDefinition, Error> {
    let f = if tokens.peek() == Tok::EOF
        || tokens.peek() == Tok::Module
        || tokens.peek() == Tok::NameValue
    {
        let mut v = vec![];
        while tokens.peek() != Tok::EOF {
            let m = if tokens.peek() == Tok::Module {
                ModuleOrAddress::Module(parse_module(tokens)?)
            } else {
                let addr_name = parse_name(tokens)?;
                if addr_name.value != "address" {
                    return Err(vec![(
                        addr_name.loc,
                        format!(
                            "Invalid address directive. Expected 'address' got '{}'",
                            addr_name.value
                        ),
                    )]);
                }
                let start_loc = tokens.start_loc();
                let addr = parse_address(tokens)?;
                let end_loc = tokens.previous_end_loc();
                consume_token(tokens, Tok::Colon)?;
                let loc = make_loc(tokens.file_name(), start_loc, end_loc);
                ModuleOrAddress::Address(loc, addr)
            };
            v.push(m);
        }
        FileDefinition::Modules(v)
    } else {
        let mut uses = vec![];
        while tokens.peek() == Tok::Use {
            uses.push(parse_use_decl(tokens)?);
        }
        let function = parse_function_decl(tokens, /* allow_native */ false)?;
        if tokens.peek() != Tok::EOF {
            let loc = current_token_loc(tokens);
            return Err(vec![(
                loc,
                "Unexpected characters after end of main function".to_string(),
            )]);
        }
        FileDefinition::Main(Main { uses, function })
    };
    Ok(f)
}

/// Parse the `input` string as a file of Move source code and return the
/// result as either a FileDefinition value or an Error. The `file` name
/// is used to identify source locations in error messages.
pub fn parse_file_string(file: &'static str, input: &str) -> Result<FileDefinition, Error> {
    let mut tokens = Lexer::new(input, file);
    tokens.advance()?;
    parse_file(&mut tokens)
}
