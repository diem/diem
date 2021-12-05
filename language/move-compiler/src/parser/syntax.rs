// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

// In the informal grammar comments in this file, Comma<T> is shorthand for:
//      (<T> ",")* <T>?
// Note that this allows an optional trailing comma.

use move_command_line_common::files::FileHash;
use move_ir_types::location::*;
use move_symbol_pool::Symbol;

use crate::{
    diag,
    diagnostics::{Diagnostic, Diagnostics},
    parser::{ast::*, lexer::*},
    shared::*,
    MatchedFileCommentMap,
};

struct Context<'env, 'lexer, 'input> {
    env: &'env mut CompilationEnv,
    tokens: &'lexer mut Lexer<'input>,
}

impl<'env, 'lexer, 'input> Context<'env, 'lexer, 'input> {
    fn new(env: &'env mut CompilationEnv, tokens: &'lexer mut Lexer<'input>) -> Self {
        Self { env, tokens }
    }
}

//**************************************************************************************************
// Error Handling
//**************************************************************************************************

fn current_token_error_string(tokens: &Lexer) -> String {
    if tokens.peek() == Tok::EOF {
        "end-of-file".to_string()
    } else {
        format!("'{}'", tokens.content())
    }
}

fn unexpected_token_error(tokens: &Lexer, expected: &str) -> Diagnostic {
    unexpected_token_error_(tokens, tokens.start_loc(), expected)
}

fn unexpected_token_error_(
    tokens: &Lexer,
    expected_start_loc: usize,
    expected: &str,
) -> Diagnostic {
    let unexpected_loc = current_token_loc(tokens);
    let unexpected = current_token_error_string(tokens);
    let expected_loc = if expected_start_loc < tokens.start_loc() {
        make_loc(
            tokens.file_hash(),
            expected_start_loc,
            tokens.previous_end_loc(),
        )
    } else {
        unexpected_loc
    };
    diag!(
        Syntax::UnexpectedToken,
        (unexpected_loc, format!("Unexpected {}", unexpected)),
        (expected_loc, format!("Expected {}", expected)),
    )
}

fn add_type_args_ambiguity_label(loc: Loc, mut diag: Diagnostic) -> Diagnostic {
    const MSG: &str = "Perhaps you need a blank space before this '<' operator?";
    diag.add_secondary_label((loc, MSG));
    diag
}

//**************************************************************************************************
// Miscellaneous Utilities
//**************************************************************************************************

pub fn make_loc(file_hash: FileHash, start: usize, end: usize) -> Loc {
    Loc::new(file_hash, start as u32, end as u32)
}

fn current_token_loc(tokens: &Lexer) -> Loc {
    let start_loc = tokens.start_loc();
    make_loc(
        tokens.file_hash(),
        start_loc,
        start_loc + tokens.content().len(),
    )
}

fn spanned<T>(file_hash: FileHash, start: usize, end: usize, value: T) -> Spanned<T> {
    Spanned {
        loc: make_loc(file_hash, start, end),
        value,
    }
}

// Check for the specified token and consume it if it matches.
// Returns true if the token matches.
fn match_token(tokens: &mut Lexer, tok: Tok) -> Result<bool, Diagnostic> {
    if tokens.peek() == tok {
        tokens.advance()?;
        Ok(true)
    } else {
        Ok(false)
    }
}

// Check for the specified token and return an error if it does not match.
fn consume_token(tokens: &mut Lexer, tok: Tok) -> Result<(), Diagnostic> {
    consume_token_(tokens, tok, tokens.start_loc(), "")
}

fn consume_token_(
    tokens: &mut Lexer,
    tok: Tok,
    expected_start_loc: usize,
    expected_case: &str,
) -> Result<(), Diagnostic> {
    if tokens.peek() == tok {
        tokens.advance()?;
        Ok(())
    } else {
        let expected = format!("'{}'{}", tok, expected_case);
        Err(unexpected_token_error_(
            tokens,
            expected_start_loc,
            &expected,
        ))
    }
}

// let unexp_loc = current_token_loc(tokens);
// let unexp_msg = format!("Unexpected {}", current_token_error_string(tokens));

// let end_loc = tokens.previous_end_loc();
// let addr_loc = make_loc(tokens.file_hash(), start_loc, end_loc);
// let exp_msg = format!("Expected '::' {}", case);
// Err(vec![(unexp_loc, unexp_msg), (addr_loc, exp_msg)])

// Check for the identifier token with specified value and return an error if it does not match.
fn consume_identifier(tokens: &mut Lexer, value: &str) -> Result<(), Diagnostic> {
    if tokens.peek() == Tok::Identifier && tokens.content() == value {
        tokens.advance()
    } else {
        let expected = format!("'{}'", value);
        Err(unexpected_token_error(tokens, &expected))
    }
}

// If the next token is the specified kind, consume it and return
// its source location.
fn consume_optional_token_with_loc(
    tokens: &mut Lexer,
    tok: Tok,
) -> Result<Option<Loc>, Diagnostic> {
    if tokens.peek() == tok {
        let start_loc = tokens.start_loc();
        tokens.advance()?;
        let end_loc = tokens.previous_end_loc();
        Ok(Some(make_loc(tokens.file_hash(), start_loc, end_loc)))
    } else {
        Ok(None)
    }
}

// While parsing a list and expecting a ">" token to mark the end, replace
// a ">>" token with the expected ">". This handles the situation where there
// are nested type parameters that result in two adjacent ">" tokens, e.g.,
// "A<B<C>>".
fn adjust_token(tokens: &mut Lexer, end_token: Tok) {
    if tokens.peek() == Tok::GreaterGreater && end_token == Tok::Greater {
        tokens.replace_token(Tok::Greater, 1);
    }
}

// Parse a comma-separated list of items, including the specified starting and
// ending tokens.
fn parse_comma_list<F, R>(
    context: &mut Context,
    start_token: Tok,
    end_token: Tok,
    parse_list_item: F,
    item_description: &str,
) -> Result<Vec<R>, Diagnostic>
where
    F: Fn(&mut Context) -> Result<R, Diagnostic>,
{
    let start_loc = context.tokens.start_loc();
    consume_token(context.tokens, start_token)?;
    parse_comma_list_after_start(
        context,
        start_loc,
        start_token,
        end_token,
        parse_list_item,
        item_description,
    )
}

// Parse a comma-separated list of items, including the specified ending token, but
// assuming that the starting token has already been consumed.
fn parse_comma_list_after_start<F, R>(
    context: &mut Context,
    start_loc: usize,
    start_token: Tok,
    end_token: Tok,
    parse_list_item: F,
    item_description: &str,
) -> Result<Vec<R>, Diagnostic>
where
    F: Fn(&mut Context) -> Result<R, Diagnostic>,
{
    adjust_token(context.tokens, end_token);
    if match_token(context.tokens, end_token)? {
        return Ok(vec![]);
    }
    let mut v = vec![];
    loop {
        if context.tokens.peek() == Tok::Comma {
            let current_loc = context.tokens.start_loc();
            let loc = make_loc(context.tokens.file_hash(), current_loc, current_loc);
            return Err(diag!(
                Syntax::UnexpectedToken,
                (loc, format!("Expected {}", item_description))
            ));
        }
        v.push(parse_list_item(context)?);
        adjust_token(&mut context.tokens, end_token);
        if match_token(&mut context.tokens, end_token)? {
            break Ok(v);
        }
        if !match_token(&mut context.tokens, Tok::Comma)? {
            let current_loc = context.tokens.start_loc();
            let loc = make_loc(context.tokens.file_hash(), current_loc, current_loc);
            let loc2 = make_loc(context.tokens.file_hash(), start_loc, start_loc);
            return Err(diag!(
                Syntax::UnexpectedToken,
                (loc, format!("Expected '{}'", end_token)),
                (loc2, format!("To match this '{}'", start_token)),
            ));
        }
        adjust_token(context.tokens, end_token);
        if match_token(context.tokens, end_token)? {
            break Ok(v);
        }
    }
}

// Parse a list of items, without specified start and end tokens, and the separator determined by
// the passed function `parse_list_continue`.
fn parse_list<C, F, R>(
    context: &mut Context,
    mut parse_list_continue: C,
    parse_list_item: F,
) -> Result<Vec<R>, Diagnostic>
where
    C: FnMut(&mut Context) -> Result<bool, Diagnostic>,
    F: Fn(&mut Context) -> Result<R, Diagnostic>,
{
    let mut v = vec![];
    loop {
        v.push(parse_list_item(context)?);
        if !parse_list_continue(context)? {
            break Ok(v);
        }
    }
}

//**************************************************************************************************
// Identifiers, Addresses, and Names
//**************************************************************************************************

// Parse an identifier:
//      Identifier = <IdentifierValue>
fn parse_identifier(context: &mut Context) -> Result<Name, Diagnostic> {
    if context.tokens.peek() != Tok::Identifier {
        return Err(unexpected_token_error(context.tokens, "an identifier"));
    }
    let start_loc = context.tokens.start_loc();
    let id = context.tokens.content().into();
    context.tokens.advance()?;
    let end_loc = context.tokens.previous_end_loc();
    Ok(spanned(context.tokens.file_hash(), start_loc, end_loc, id))
}

// Parse a numerical address value
//     NumericalAddress = <Number>
fn parse_address_bytes(context: &mut Context) -> Result<Spanned<NumericalAddress>, Diagnostic> {
    let loc = current_token_loc(context.tokens);
    let addr_res = NumericalAddress::parse_str(context.tokens.content());
    consume_token(context.tokens, Tok::NumValue)?;
    let addr_ = match addr_res {
        Ok(addr_) => addr_,
        Err(msg) => {
            context
                .env
                .add_diag(diag!(Syntax::InvalidAddress, (loc, msg)));
            NumericalAddress::DEFAULT_ERROR_ADDRESS
        }
    };
    Ok(sp(loc, addr_))
}

// Parse the beginning of an access, either an address or an identifier:
//      LeadingNameAccess = <NumericalAddress> | <Identifier>
fn parse_leading_name_access(context: &mut Context) -> Result<LeadingNameAccess, Diagnostic> {
    parse_leading_name_access_(context, || "an address or an identifier")
}

// Parse the beginning of an access, either an address or an identifier with a specific description
fn parse_leading_name_access_<'a, F: FnOnce() -> &'a str>(
    context: &mut Context,
    item_description: F,
) -> Result<LeadingNameAccess, Diagnostic> {
    match context.tokens.peek() {
        Tok::Identifier => {
            let loc = current_token_loc(context.tokens);
            let n = parse_identifier(context)?;
            Ok(sp(loc, LeadingNameAccess_::Name(n)))
        }
        Tok::NumValue => {
            let sp!(loc, addr) = parse_address_bytes(context)?;
            Ok(sp(loc, LeadingNameAccess_::AnonymousAddress(addr)))
        }
        _ => Err(unexpected_token_error(context.tokens, item_description())),
    }
}

// Parse a variable name:
//      Var = <Identifier>
fn parse_var(context: &mut Context) -> Result<Var, Diagnostic> {
    Ok(Var(parse_identifier(context)?))
}

// Parse a field name:
//      Field = <Identifier>
fn parse_field(context: &mut Context) -> Result<Field, Diagnostic> {
    Ok(Field(parse_identifier(context)?))
}

// Parse a module name:
//      ModuleName = <Identifier>
fn parse_module_name(context: &mut Context) -> Result<ModuleName, Diagnostic> {
    Ok(ModuleName(parse_identifier(context)?))
}

// Parse a module identifier:
//      ModuleIdent = <LeadingNameAccess> "::" <ModuleName>
fn parse_module_ident(context: &mut Context) -> Result<ModuleIdent, Diagnostic> {
    let start_loc = context.tokens.start_loc();
    let address = parse_leading_name_access(context)?;

    consume_token_(
        context.tokens,
        Tok::ColonColon,
        start_loc,
        " after an address in a module identifier",
    )?;
    let module = parse_module_name(context)?;
    let end_loc = context.tokens.previous_end_loc();
    let loc = make_loc(context.tokens.file_hash(), start_loc, end_loc);
    Ok(sp(loc, ModuleIdent_ { address, module }))
}

// Parse a module access (a variable, struct type, or function):
//      NameAccessChain = <LeadingNameAccess> ( "::" <Identifier> ( "::" <Identifier> )? )?
fn parse_name_access_chain<'a, F: FnOnce() -> &'a str>(
    context: &mut Context,
    item_description: F,
) -> Result<NameAccessChain, Diagnostic> {
    let start_loc = context.tokens.start_loc();
    let access = parse_name_access_chain_(context, item_description)?;
    let end_loc = context.tokens.previous_end_loc();
    Ok(spanned(
        context.tokens.file_hash(),
        start_loc,
        end_loc,
        access,
    ))
}

// Parse a module access with a specific description
fn parse_name_access_chain_<'a, F: FnOnce() -> &'a str>(
    context: &mut Context,
    item_description: F,
) -> Result<NameAccessChain_, Diagnostic> {
    let start_loc = context.tokens.start_loc();
    let ln = parse_leading_name_access_(context, item_description)?;
    let ln = match ln {
        // A name by itself is a valid access chain
        sp!(_, LeadingNameAccess_::Name(n1)) if context.tokens.peek() != Tok::ColonColon => {
            return Ok(NameAccessChain_::One(n1))
        }
        ln => ln,
    };

    consume_token_(
        context.tokens,
        Tok::ColonColon,
        start_loc,
        " after an address in a module access chain",
    )?;
    let n2 = parse_identifier(context)?;
    if context.tokens.peek() != Tok::ColonColon {
        return Ok(NameAccessChain_::Two(ln, n2));
    }
    let ln_n2_loc = make_loc(
        context.tokens.file_hash(),
        start_loc,
        context.tokens.previous_end_loc(),
    );
    consume_token(context.tokens, Tok::ColonColon)?;
    let n3 = parse_identifier(context)?;
    Ok(NameAccessChain_::Three(sp(ln_n2_loc, (ln, n2)), n3))
}

//**************************************************************************************************
// Modifiers
//**************************************************************************************************

struct Modifiers {
    visibility: Option<Visibility>,
    native: Option<Loc>,
}

impl Modifiers {
    fn empty() -> Self {
        Self {
            visibility: None,
            native: None,
        }
    }
}

// Parse module member modifiers: visiblility and native.
// The modifiers are also used for script-functions
//      ModuleMemberModifiers = <ModuleMemberModifier>*
//      ModuleMemberModifier = <Visibility> | "native"
// ModuleMemberModifiers checks for uniqueness, meaning each individual ModuleMemberModifier can
// appear only once
fn parse_module_member_modifiers(context: &mut Context) -> Result<Modifiers, Diagnostic> {
    let mut mods = Modifiers::empty();
    loop {
        match context.tokens.peek() {
            Tok::Public => {
                let vis = parse_visibility(context)?;
                if let Some(prev_vis) = mods.visibility {
                    let msg = "Duplicate visibility modifier".to_string();
                    let prev_msg = "Visibility modifier previously given here".to_string();
                    context.env.add_diag(diag!(
                        Declarations::DuplicateItem,
                        (vis.loc().unwrap(), msg),
                        (prev_vis.loc().unwrap(), prev_msg),
                    ));
                }
                mods.visibility = Some(vis)
            }
            Tok::Native => {
                let loc = current_token_loc(context.tokens);
                context.tokens.advance()?;
                if let Some(prev_loc) = mods.native {
                    let msg = "Duplicate 'native' modifier".to_string();
                    let prev_msg = "'native' modifier previously given here".to_string();
                    context.env.add_diag(diag!(
                        Declarations::DuplicateItem,
                        (loc, msg),
                        (prev_loc, prev_msg)
                    ))
                }
                mods.native = Some(loc)
            }
            _ => break,
        }
    }
    Ok(mods)
}

// Parse a function visibility modifier:
//      Visibility = "public" ( "(" "script" | "friend" ")" )?
fn parse_visibility(context: &mut Context) -> Result<Visibility, Diagnostic> {
    let start_loc = context.tokens.start_loc();
    consume_token(context.tokens, Tok::Public)?;
    let sub_public_vis = if match_token(context.tokens, Tok::LParen)? {
        let sub_token = context.tokens.peek();
        context.tokens.advance()?;
        if sub_token != Tok::RParen {
            consume_token(context.tokens, Tok::RParen)?;
        }
        Some(sub_token)
    } else {
        None
    };
    let end_loc = context.tokens.previous_end_loc();
    // this loc will cover the span of 'public' or 'public(...)' in entirety
    let loc = make_loc(context.tokens.file_hash(), start_loc, end_loc);
    Ok(match sub_public_vis {
        None => Visibility::Public(loc),
        Some(Tok::Script) => Visibility::Script(loc),
        Some(Tok::Friend) => Visibility::Friend(loc),
        _ => {
            let msg = format!(
                "Invalid visibility modifier. Consider removing it or using one of '{}', '{}', or \
                 '{}'",
                Visibility::PUBLIC,
                Visibility::SCRIPT,
                Visibility::FRIEND
            );
            return Err(diag!(Syntax::UnexpectedToken, (loc, msg)));
        }
    })
}
// Parse an attribute value. Either a value literal or a module access
//      AttributeValue =
//          <Value>
//          | <NameAccessChain>
fn parse_attribute_value(context: &mut Context) -> Result<AttributeValue, Diagnostic> {
    if let Some(v) = maybe_parse_value(context)? {
        return Ok(sp(v.loc, AttributeValue_::Value(v)));
    }

    let ma = parse_name_access_chain(context, || "attribute name value")?;
    Ok(sp(ma.loc, AttributeValue_::ModuleAccess(ma)))
}

// Parse a single attribute
//      Attribute =
//          <Identifier>
//          | <Identifier> "=" <AttributeValue>
//          | <Identifier> "(" Comma<Attribute> ")"
fn parse_attribute(context: &mut Context) -> Result<Attribute, Diagnostic> {
    let start_loc = context.tokens.start_loc();
    let n = parse_identifier(context)?;
    let attr_ = match context.tokens.peek() {
        Tok::Equal => {
            context.tokens.advance()?;
            Attribute_::Assigned(n, Box::new(parse_attribute_value(context)?))
        }
        Tok::LParen => {
            let args_ = parse_comma_list(
                context,
                Tok::LParen,
                Tok::RParen,
                parse_attribute,
                "attribute",
            )?;
            let end_loc = context.tokens.previous_end_loc();
            Attribute_::Parameterized(
                n,
                spanned(context.tokens.file_hash(), start_loc, end_loc, args_),
            )
        }
        _ => Attribute_::Name(n),
    };
    let end_loc = context.tokens.previous_end_loc();
    Ok(spanned(
        context.tokens.file_hash(),
        start_loc,
        end_loc,
        attr_,
    ))
}

// Parse attributes. Used to annotate a variety of AST nodes
//      Attributes = ("#" "[" Comma<Attribute> "]")*
fn parse_attributes(context: &mut Context) -> Result<Vec<Attributes>, Diagnostic> {
    let mut attributes_vec = vec![];
    while let Tok::NumSign = context.tokens.peek() {
        let start_loc = context.tokens.start_loc();
        context.tokens.advance()?;
        let attributes_ = parse_comma_list(
            context,
            Tok::LBracket,
            Tok::RBracket,
            parse_attribute,
            "attribute",
        )?;
        let end_loc = context.tokens.previous_end_loc();
        attributes_vec.push(spanned(
            context.tokens.file_hash(),
            start_loc,
            end_loc,
            attributes_,
        ))
    }
    Ok(attributes_vec)
}

//**************************************************************************************************
// Fields and Bindings
//**************************************************************************************************

// Parse a field name optionally followed by a colon and an expression argument:
//      ExpField = <Field> <":" <Exp>>?
fn parse_exp_field(context: &mut Context) -> Result<(Field, Exp), Diagnostic> {
    let f = parse_field(context)?;
    let arg = if match_token(context.tokens, Tok::Colon)? {
        parse_exp(context)?
    } else {
        sp(
            f.loc(),
            Exp_::Name(sp(f.loc(), NameAccessChain_::One(f.0)), None),
        )
    };
    Ok((f, arg))
}

// Parse a field name optionally followed by a colon and a binding:
//      BindField = <Field> <":" <Bind>>?
//
// If the binding is not specified, the default is to use a variable
// with the same name as the field.
fn parse_bind_field(context: &mut Context) -> Result<(Field, Bind), Diagnostic> {
    let f = parse_field(context)?;
    let arg = if match_token(context.tokens, Tok::Colon)? {
        parse_bind(context)?
    } else {
        let v = Var(f.0);
        sp(v.loc(), Bind_::Var(v))
    };
    Ok((f, arg))
}

// Parse a binding:
//      Bind =
//          <Var>
//          | <NameAccessChain> <OptionalTypeArgs> "{" Comma<BindField> "}"
fn parse_bind(context: &mut Context) -> Result<Bind, Diagnostic> {
    let start_loc = context.tokens.start_loc();
    if context.tokens.peek() == Tok::Identifier {
        let next_tok = context.tokens.lookahead()?;
        if next_tok != Tok::LBrace && next_tok != Tok::Less && next_tok != Tok::ColonColon {
            let v = Bind_::Var(parse_var(context)?);
            let end_loc = context.tokens.previous_end_loc();
            return Ok(spanned(context.tokens.file_hash(), start_loc, end_loc, v));
        }
    }
    // The item description specified here should include the special case above for
    // variable names, because if the current context cannot be parsed as a struct name
    // it is possible that the user intention was to use a variable name.
    let ty = parse_name_access_chain(context, || "a variable or struct name")?;
    let ty_args = parse_optional_type_args(context)?;
    let args = parse_comma_list(
        context,
        Tok::LBrace,
        Tok::RBrace,
        parse_bind_field,
        "a field binding",
    )?;
    let end_loc = context.tokens.previous_end_loc();
    let unpack = Bind_::Unpack(Box::new(ty), ty_args, args);
    Ok(spanned(
        context.tokens.file_hash(),
        start_loc,
        end_loc,
        unpack,
    ))
}

// Parse a list of bindings, which can be zero, one, or more bindings:
//      BindList =
//          <Bind>
//          | "(" Comma<Bind> ")"
//
// The list is enclosed in parenthesis, except that the parenthesis are
// optional if there is a single Bind.
fn parse_bind_list(context: &mut Context) -> Result<BindList, Diagnostic> {
    let start_loc = context.tokens.start_loc();
    let b = if context.tokens.peek() != Tok::LParen {
        vec![parse_bind(context)?]
    } else {
        parse_comma_list(
            context,
            Tok::LParen,
            Tok::RParen,
            parse_bind,
            "a variable or structure binding",
        )?
    };
    let end_loc = context.tokens.previous_end_loc();
    Ok(spanned(context.tokens.file_hash(), start_loc, end_loc, b))
}

// Parse a list of bindings for lambda.
//      LambdaBindList =
//          "|" Comma<Bind> "|"
fn parse_lambda_bind_list(context: &mut Context) -> Result<BindList, Diagnostic> {
    let start_loc = context.tokens.start_loc();
    let b = parse_comma_list(
        context,
        Tok::Pipe,
        Tok::Pipe,
        parse_bind,
        "a variable or structure binding",
    )?;
    let end_loc = context.tokens.previous_end_loc();
    Ok(spanned(context.tokens.file_hash(), start_loc, end_loc, b))
}

//**************************************************************************************************
// Values
//**************************************************************************************************

// Parse a byte string:
//      ByteString = <ByteStringValue>
fn parse_byte_string(context: &mut Context) -> Result<Value_, Diagnostic> {
    if context.tokens.peek() != Tok::ByteStringValue {
        return Err(unexpected_token_error(
            context.tokens,
            "a byte string value",
        ));
    }
    let s = context.tokens.content();
    let text = Symbol::from(&s[2..s.len() - 1]);
    let value_ = if s.starts_with("x\"") {
        Value_::HexString(text)
    } else {
        assert!(s.starts_with("b\""));
        Value_::ByteString(text)
    };
    context.tokens.advance()?;
    Ok(value_)
}

// Parse a value:
//      Value =
//          "@" <LeadingAccessName>
//          | "true"
//          | "false"
//          | <Number>
//          | <NumberTyped>
//          | <ByteString>
fn maybe_parse_value(context: &mut Context) -> Result<Option<Value>, Diagnostic> {
    let start_loc = context.tokens.start_loc();
    let val = match context.tokens.peek() {
        Tok::AtSign => {
            context.tokens.advance()?;
            let addr = parse_leading_name_access(context)?;
            Value_::Address(addr)
        }
        Tok::True => {
            context.tokens.advance()?;
            Value_::Bool(true)
        }
        Tok::False => {
            context.tokens.advance()?;
            Value_::Bool(false)
        }
        Tok::NumValue => {
            //  If the number is followed by "::", parse it as the beginning of an address access
            if let Ok(Tok::ColonColon) = context.tokens.lookahead() {
                return Ok(None);
            }
            let num = context.tokens.content().into();
            context.tokens.advance()?;
            Value_::Num(num)
        }
        Tok::NumTypedValue => {
            let num = context.tokens.content().into();
            context.tokens.advance()?;
            Value_::Num(num)
        }

        Tok::ByteStringValue => parse_byte_string(context)?,
        _ => return Ok(None),
    };
    let end_loc = context.tokens.previous_end_loc();
    Ok(Some(spanned(
        context.tokens.file_hash(),
        start_loc,
        end_loc,
        val,
    )))
}

fn parse_value(context: &mut Context) -> Result<Value, Diagnostic> {
    Ok(maybe_parse_value(context)?.expect("parse_value called with invalid token"))
}

//**************************************************************************************************
// Sequences
//**************************************************************************************************

// Parse a sequence item:
//      SequenceItem =
//          <Exp>
//          | "let" <BindList> (":" <Type>)? ("=" <Exp>)?
fn parse_sequence_item(context: &mut Context) -> Result<SequenceItem, Diagnostic> {
    let start_loc = context.tokens.start_loc();
    let item = if match_token(context.tokens, Tok::Let)? {
        let b = parse_bind_list(context)?;
        let ty_opt = if match_token(context.tokens, Tok::Colon)? {
            Some(parse_type(context)?)
        } else {
            None
        };
        if match_token(context.tokens, Tok::Equal)? {
            let e = parse_exp(context)?;
            SequenceItem_::Bind(b, ty_opt, Box::new(e))
        } else {
            SequenceItem_::Declare(b, ty_opt)
        }
    } else {
        let e = parse_exp(context)?;
        SequenceItem_::Seq(Box::new(e))
    };
    let end_loc = context.tokens.previous_end_loc();
    Ok(spanned(
        context.tokens.file_hash(),
        start_loc,
        end_loc,
        item,
    ))
}

// Parse a sequence:
//      Sequence = <UseDecl>* (<SequenceItem> ";")* <Exp>? "}"
//
// Note that this does not include the opening brace of a block but it
// does consume the closing right brace.
fn parse_sequence(context: &mut Context) -> Result<Sequence, Diagnostic> {
    let mut uses = vec![];
    while context.tokens.peek() == Tok::Use {
        uses.push(parse_use_decl(vec![], context)?);
    }

    let mut seq: Vec<SequenceItem> = vec![];
    let mut last_semicolon_loc = None;
    let mut eopt = None;
    while context.tokens.peek() != Tok::RBrace {
        let item = parse_sequence_item(context)?;
        if context.tokens.peek() == Tok::RBrace {
            // If the sequence ends with an expression that is not
            // followed by a semicolon, split out that expression
            // from the rest of the SequenceItems.
            match item.value {
                SequenceItem_::Seq(e) => {
                    eopt = Some(Spanned {
                        loc: item.loc,
                        value: e.value,
                    });
                }
                _ => return Err(unexpected_token_error(context.tokens, "';'")),
            }
            break;
        }
        seq.push(item);
        last_semicolon_loc = Some(current_token_loc(context.tokens));
        consume_token(context.tokens, Tok::Semicolon)?;
    }
    context.tokens.advance()?; // consume the RBrace
    Ok((uses, seq, last_semicolon_loc, Box::new(eopt)))
}

//**************************************************************************************************
// Expressions
//**************************************************************************************************

// Parse an expression term:
//      Term =
//          "break"
//          | "continue"
//          | "vector" ("<" Comma<Type> ">")? "[" Comma<Exp> "]"
//          | <Value>
//          | "(" Comma<Exp> ")"
//          | "(" <Exp> ":" <Type> ")"
//          | "(" <Exp> "as" <Type> ")"
//          | "{" <Sequence>
fn parse_term(context: &mut Context) -> Result<Exp, Diagnostic> {
    const VECTOR_IDENT: &str = "vector";

    let start_loc = context.tokens.start_loc();
    let term = match context.tokens.peek() {
        Tok::Break => {
            context.tokens.advance()?;
            Exp_::Break
        }

        Tok::Continue => {
            context.tokens.advance()?;
            Exp_::Continue
        }

        Tok::Identifier if context.tokens.content() == VECTOR_IDENT => {
            consume_identifier(context.tokens, VECTOR_IDENT)?;
            let vec_end_loc = context.tokens.previous_end_loc();
            let vec_loc = make_loc(context.tokens.file_hash(), start_loc, vec_end_loc);
            let targs_start_loc = context.tokens.start_loc();
            let tys_opt = parse_optional_type_args(context).map_err(|diag| {
                let targ_loc =
                    make_loc(context.tokens.file_hash(), targs_start_loc, targs_start_loc);
                add_type_args_ambiguity_label(targ_loc, diag)
            })?;
            let args_start_loc = context.tokens.start_loc();
            let args_ = parse_comma_list(
                context,
                Tok::LBracket,
                Tok::RBracket,
                parse_exp,
                "a vector argument expression",
            )?;
            let args_end_loc = context.tokens.previous_end_loc();
            let args = spanned(
                context.tokens.file_hash(),
                args_start_loc,
                args_end_loc,
                args_,
            );
            Exp_::Vector(vec_loc, tys_opt, args)
        }

        Tok::Identifier => parse_name_exp(context)?,

        Tok::NumValue => {
            // Check if this is a ModuleIdent (in a ModuleAccess).
            if context.tokens.lookahead()? == Tok::ColonColon {
                parse_name_exp(context)?
            } else {
                Exp_::Value(parse_value(context)?)
            }
        }

        Tok::AtSign | Tok::True | Tok::False | Tok::NumTypedValue | Tok::ByteStringValue => {
            Exp_::Value(parse_value(context)?)
        }

        // "(" Comma<Exp> ")"
        // "(" <Exp> ":" <Type> ")"
        // "(" <Exp> "as" <Type> ")"
        Tok::LParen => {
            let list_loc = context.tokens.start_loc();
            context.tokens.advance()?; // consume the LParen
            if match_token(context.tokens, Tok::RParen)? {
                Exp_::Unit
            } else {
                // If there is a single expression inside the parens,
                // then it may be followed by a colon and a type annotation.
                let e = parse_exp(context)?;
                if match_token(context.tokens, Tok::Colon)? {
                    let ty = parse_type(context)?;
                    consume_token(context.tokens, Tok::RParen)?;
                    Exp_::Annotate(Box::new(e), ty)
                } else if match_token(context.tokens, Tok::As)? {
                    let ty = parse_type(context)?;
                    consume_token(context.tokens, Tok::RParen)?;
                    Exp_::Cast(Box::new(e), ty)
                } else {
                    if context.tokens.peek() != Tok::RParen {
                        consume_token(context.tokens, Tok::Comma)?;
                    }
                    let mut es = parse_comma_list_after_start(
                        context,
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
            context.tokens.advance()?; // consume the LBrace
            Exp_::Block(parse_sequence(context)?)
        }

        Tok::Spec => {
            let spec_block = parse_spec_block(vec![], context)?;
            Exp_::Spec(spec_block)
        }

        _ => {
            return Err(unexpected_token_error(context.tokens, "an expression term"));
        }
    };
    let end_loc = context.tokens.previous_end_loc();
    Ok(spanned(
        context.tokens.file_hash(),
        start_loc,
        end_loc,
        term,
    ))
}

// Parse a pack, call, or other reference to a name:
//      NameExp =
//          <NameAccessChain> <OptionalTypeArgs> "{" Comma<ExpField> "}"
//          | <NameAccessChain> <OptionalTypeArgs> "(" Comma<Exp> ")"
//          | <NameAccessChain> "!" "(" Comma<Exp> ")"
//          | <NameAccessChain> <OptionalTypeArgs>
fn parse_name_exp(context: &mut Context) -> Result<Exp_, Diagnostic> {
    let n = parse_name_access_chain(context, || {
        panic!("parse_name_exp with something other than a ModuleAccess")
    })?;

    // There's an ambiguity if the name is followed by a "<". If there is no whitespace
    // after the name, treat it as the start of a list of type arguments. Otherwise
    // assume that the "<" is a boolean operator.
    let mut tys = None;
    let start_loc = context.tokens.start_loc();
    if context.tokens.peek() == Tok::Exclaim {
        context.tokens.advance()?;
        let is_macro = true;
        let rhs = parse_call_args(context)?;
        return Ok(Exp_::Call(n, is_macro, tys, rhs));
    }

    if context.tokens.peek() == Tok::Less && n.loc.end() as usize == start_loc {
        let loc = make_loc(context.tokens.file_hash(), start_loc, start_loc);
        tys = parse_optional_type_args(context)
            .map_err(|diag| add_type_args_ambiguity_label(loc, diag))?;
    }

    match context.tokens.peek() {
        // Pack: "{" Comma<ExpField> "}"
        Tok::LBrace => {
            let fs = parse_comma_list(
                context,
                Tok::LBrace,
                Tok::RBrace,
                parse_exp_field,
                "a field expression",
            )?;
            Ok(Exp_::Pack(n, tys, fs))
        }

        // Call: "(" Comma<Exp> ")"
        Tok::Exclaim | Tok::LParen => {
            let is_macro = false;
            let rhs = parse_call_args(context)?;
            Ok(Exp_::Call(n, is_macro, tys, rhs))
        }

        // Other name reference...
        _ => Ok(Exp_::Name(n, tys)),
    }
}

// Parse the arguments to a call: "(" Comma<Exp> ")"
fn parse_call_args(context: &mut Context) -> Result<Spanned<Vec<Exp>>, Diagnostic> {
    let start_loc = context.tokens.start_loc();
    let args = parse_comma_list(
        context,
        Tok::LParen,
        Tok::RParen,
        parse_exp,
        "a call argument expression",
    )?;
    let end_loc = context.tokens.previous_end_loc();
    Ok(spanned(
        context.tokens.file_hash(),
        start_loc,
        end_loc,
        args,
    ))
}

// Return true if the current token is one that might occur after an Exp.
// This is needed, for example, to check for the optional Exp argument to
// a return (where "return" is itself an Exp).
fn at_end_of_exp(context: &mut Context) -> bool {
    matches!(
        context.tokens.peek(),
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
//          | "while" "(" <Exp> ")" <Exp> (SpecBlock)?
//          | "loop" <Exp>
//          | "return" <Exp>?
//          | "abort" <Exp>
//          | <BinOpExp>
//          | <UnaryExp> "=" <Exp>
fn parse_exp(context: &mut Context) -> Result<Exp, Diagnostic> {
    let start_loc = context.tokens.start_loc();
    let exp = match context.tokens.peek() {
        Tok::Pipe => {
            let bindings = parse_lambda_bind_list(context)?;
            let body = Box::new(parse_exp(context)?);
            Exp_::Lambda(bindings, body)
        }
        Tok::Identifier if is_quant(context) => parse_quant(context)?,
        Tok::If => {
            context.tokens.advance()?;
            consume_token(context.tokens, Tok::LParen)?;
            let eb = Box::new(parse_exp(context)?);
            consume_token(context.tokens, Tok::RParen)?;
            let et = Box::new(parse_exp(context)?);
            let ef = if match_token(context.tokens, Tok::Else)? {
                Some(Box::new(parse_exp(context)?))
            } else {
                None
            };
            Exp_::IfElse(eb, et, ef)
        }
        Tok::While => {
            context.tokens.advance()?;
            consume_token(context.tokens, Tok::LParen)?;
            let econd = parse_exp(context)?;
            consume_token(context.tokens, Tok::RParen)?;
            let eloop = Box::new(parse_exp(context)?);
            let econd = if context.tokens.peek() == Tok::Spec {
                // Parse a loop invariant. Also validate that only `invariant`
                // properties are contained in the spec block. This is
                // transformed into `while ({spec { .. }; cond) body`.
                let spec = parse_spec_block(vec![], context)?;
                for member in &spec.value.members {
                    match member.value {
                        SpecBlockMember_::Condition {
                            kind: sp!(_, SpecConditionKind_::Invariant(..)),
                            ..
                        } => {
                            // Ok
                        }
                        _ => {
                            return Err(diag!(
                                Syntax::InvalidSpecBlockMember,
                                (member.loc, "only 'invariant' allowed here")
                            ))
                        }
                    }
                }
                sp(
                    econd.loc,
                    Exp_::Block((
                        vec![],
                        vec![sp(
                            spec.loc,
                            SequenceItem_::Seq(Box::new(sp(spec.loc, Exp_::Spec(spec)))),
                        )],
                        None,
                        Box::new(Some(econd)),
                    )),
                )
            } else {
                econd
            };
            Exp_::While(Box::new(econd), eloop)
        }
        Tok::Loop => {
            context.tokens.advance()?;
            let eloop = Box::new(parse_exp(context)?);
            Exp_::Loop(eloop)
        }
        Tok::Return => {
            context.tokens.advance()?;
            let e = if at_end_of_exp(context) {
                None
            } else {
                Some(Box::new(parse_exp(context)?))
            };
            Exp_::Return(e)
        }
        Tok::Abort => {
            context.tokens.advance()?;
            let e = Box::new(parse_exp(context)?);
            Exp_::Abort(e)
        }
        _ => {
            // This could be either an assignment or a binary operator
            // expression.
            let lhs = parse_unary_exp(context)?;
            if context.tokens.peek() != Tok::Equal {
                return parse_binop_exp(context, lhs, /* min_prec */ 1);
            }
            context.tokens.advance()?; // consume the "="
            let rhs = Box::new(parse_exp(context)?);
            Exp_::Assign(Box::new(lhs), rhs)
        }
    };
    let end_loc = context.tokens.previous_end_loc();
    Ok(spanned(context.tokens.file_hash(), start_loc, end_loc, exp))
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
        Tok::LessEqualEqualGreater => 2,
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
fn parse_binop_exp(context: &mut Context, lhs: Exp, min_prec: u32) -> Result<Exp, Diagnostic> {
    let mut result = lhs;
    let mut next_tok_prec = get_precedence(context.tokens.peek());

    while next_tok_prec >= min_prec {
        // Parse the operator.
        let op_start_loc = context.tokens.start_loc();
        let op_token = context.tokens.peek();
        context.tokens.advance()?;
        let op_end_loc = context.tokens.previous_end_loc();

        let mut rhs = parse_unary_exp(context)?;

        // If the next token is another binary operator with a higher
        // precedence, then recursively parse that expression as the RHS.
        let this_prec = next_tok_prec;
        next_tok_prec = get_precedence(context.tokens.peek());
        if this_prec < next_tok_prec {
            rhs = parse_binop_exp(context, rhs, this_prec + 1)?;
            next_tok_prec = get_precedence(context.tokens.peek());
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
            Tok::LessEqualEqualGreater => BinOp_::Iff,
            _ => panic!("Unexpected token that is not a binary operator"),
        };
        let sp_op = spanned(context.tokens.file_hash(), op_start_loc, op_end_loc, op);

        let start_loc = result.loc.start() as usize;
        let end_loc = context.tokens.previous_end_loc();
        let e = Exp_::BinopExp(Box::new(result), sp_op, Box::new(rhs));
        result = spanned(context.tokens.file_hash(), start_loc, end_loc, e);
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
fn parse_unary_exp(context: &mut Context) -> Result<Exp, Diagnostic> {
    let start_loc = context.tokens.start_loc();
    let exp = match context.tokens.peek() {
        Tok::Exclaim => {
            context.tokens.advance()?;
            let op_end_loc = context.tokens.previous_end_loc();
            let op = spanned(
                context.tokens.file_hash(),
                start_loc,
                op_end_loc,
                UnaryOp_::Not,
            );
            let e = parse_unary_exp(context)?;
            Exp_::UnaryExp(op, Box::new(e))
        }
        Tok::AmpMut => {
            context.tokens.advance()?;
            let e = parse_unary_exp(context)?;
            Exp_::Borrow(true, Box::new(e))
        }
        Tok::Amp => {
            context.tokens.advance()?;
            let e = parse_unary_exp(context)?;
            Exp_::Borrow(false, Box::new(e))
        }
        Tok::Star => {
            context.tokens.advance()?;
            let e = parse_unary_exp(context)?;
            Exp_::Dereference(Box::new(e))
        }
        Tok::Move => {
            context.tokens.advance()?;
            Exp_::Move(parse_var(context)?)
        }
        Tok::Copy => {
            context.tokens.advance()?;
            Exp_::Copy(parse_var(context)?)
        }
        _ => {
            return parse_dot_or_index_chain(context);
        }
    };
    let end_loc = context.tokens.previous_end_loc();
    Ok(spanned(context.tokens.file_hash(), start_loc, end_loc, exp))
}

// Parse an expression term optionally followed by a chain of dot or index accesses:
//      DotOrIndexChain =
//          <DotOrIndexChain> "." <Identifier>
//          | <DotOrIndexChain> "[" <Exp> "]"                      spec only
//          | <Term>
fn parse_dot_or_index_chain(context: &mut Context) -> Result<Exp, Diagnostic> {
    let start_loc = context.tokens.start_loc();
    let mut lhs = parse_term(context)?;
    loop {
        let exp = match context.tokens.peek() {
            Tok::Period => {
                context.tokens.advance()?;
                let n = parse_identifier(context)?;
                Exp_::Dot(Box::new(lhs), n)
            }
            Tok::LBracket => {
                context.tokens.advance()?;
                let index = parse_exp(context)?;
                let exp = Exp_::Index(Box::new(lhs), Box::new(index));
                consume_token(context.tokens, Tok::RBracket)?;
                exp
            }
            _ => break,
        };
        let end_loc = context.tokens.previous_end_loc();
        lhs = spanned(context.tokens.file_hash(), start_loc, end_loc, exp);
    }
    Ok(lhs)
}

// Lookahead to determine whether this is a quantifier. This matches
//
//      ( "exists" | "forall" | "choose" | "min" )
//          <Identifier> ( ":" | <Identifier> ) ...
//
// as a sequence to identify a quantifier. While the <Identifier> after
// the exists/forall would by syntactically sufficient (Move does not
// have affixed identifiers in expressions), we add another token
// of lookahead to keep the result more precise in the presence of
// syntax errors.
fn is_quant(context: &mut Context) -> bool {
    if !matches!(context.tokens.content(), "exists" | "forall" | "choose") {
        return false;
    }
    match context.tokens.lookahead2() {
        Err(_) => false,
        Ok((tok1, tok2)) => tok1 == Tok::Identifier && matches!(tok2, Tok::Colon | Tok::Identifier),
    }
}

// Parses a quantifier expressions, assuming is_quant(context) is true.
//
//   <Quantifier> =
//       ( "forall" | "exists" ) <QuantifierBindings> ({ (<Exp>)* })* ("where" <Exp>)? ":" Exp
//     | ( "choose" [ "min" ] ) <QuantifierBind> "where" <Exp>
//   <QuantifierBindings> = <QuantifierBind> ("," <QuantifierBind>)*
//   <QuantifierBind> = <Identifier> ":" <Type> | <Identifier> "in" <Exp>
//
fn parse_quant(context: &mut Context) -> Result<Exp_, Diagnostic> {
    let start_loc = context.tokens.start_loc();
    let kind = match context.tokens.content() {
        "exists" => {
            context.tokens.advance()?;
            QuantKind_::Exists
        }
        "forall" => {
            context.tokens.advance()?;
            QuantKind_::Forall
        }
        "choose" => {
            context.tokens.advance()?;
            match context.tokens.peek() {
                Tok::Identifier if context.tokens.content() == "min" => {
                    context.tokens.advance()?;
                    QuantKind_::ChooseMin
                }
                _ => QuantKind_::Choose,
            }
        }
        _ => unreachable!(),
    };
    let spanned_kind = spanned(
        context.tokens.file_hash(),
        start_loc,
        context.tokens.previous_end_loc(),
        kind,
    );

    if matches!(kind, QuantKind_::Choose | QuantKind_::ChooseMin) {
        let binding = parse_quant_binding(context)?;
        consume_identifier(context.tokens, "where")?;
        let body = parse_exp(context)?;
        return Ok(Exp_::Quant(
            spanned_kind,
            Spanned {
                loc: binding.loc,
                value: vec![binding],
            },
            vec![],
            None,
            Box::new(body),
        ));
    }

    let bindings_start_loc = context.tokens.start_loc();
    let binds_with_range_list = parse_list(
        context,
        |context| {
            if context.tokens.peek() == Tok::Comma {
                context.tokens.advance()?;
                Ok(true)
            } else {
                Ok(false)
            }
        },
        parse_quant_binding,
    )?;
    let binds_with_range_list = spanned(
        context.tokens.file_hash(),
        bindings_start_loc,
        context.tokens.previous_end_loc(),
        binds_with_range_list,
    );

    let triggers = if context.tokens.peek() == Tok::LBrace {
        parse_list(
            context,
            |context| {
                if context.tokens.peek() == Tok::LBrace {
                    Ok(true)
                } else {
                    Ok(false)
                }
            },
            |context| {
                parse_comma_list(
                    context,
                    Tok::LBrace,
                    Tok::RBrace,
                    parse_exp,
                    "a trigger expresssion",
                )
            },
        )?
    } else {
        Vec::new()
    };

    let condition = match context.tokens.peek() {
        Tok::Identifier if context.tokens.content() == "where" => {
            context.tokens.advance()?;
            Some(Box::new(parse_exp(context)?))
        }
        _ => None,
    };
    consume_token(context.tokens, Tok::Colon)?;
    let body = parse_exp(context)?;

    Ok(Exp_::Quant(
        spanned_kind,
        binds_with_range_list,
        triggers,
        condition,
        Box::new(body),
    ))
}

// Parses one quantifier binding.
fn parse_quant_binding(context: &mut Context) -> Result<Spanned<(Bind, Exp)>, Diagnostic> {
    let start_loc = context.tokens.start_loc();
    let ident = parse_identifier(context)?;
    let bind = spanned(
        context.tokens.file_hash(),
        start_loc,
        context.tokens.previous_end_loc(),
        Bind_::Var(Var(ident)),
    );
    let range = if context.tokens.peek() == Tok::Colon {
        // This is a quantifier over the full domain of a type.
        // Built `domain<ty>()` expression.
        context.tokens.advance()?;
        let ty = parse_type(context)?;
        make_builtin_call(ty.loc, Symbol::from("$spec_domain"), Some(vec![ty]), vec![])
    } else {
        // This is a quantifier over a value, like a vector or a range.
        consume_identifier(context.tokens, "in")?;
        parse_exp(context)?
    };
    let end_loc = context.tokens.previous_end_loc();
    Ok(spanned(
        context.tokens.file_hash(),
        start_loc,
        end_loc,
        (bind, range),
    ))
}

fn make_builtin_call(loc: Loc, name: Symbol, type_args: Option<Vec<Type>>, args: Vec<Exp>) -> Exp {
    let maccess = sp(loc, NameAccessChain_::One(sp(loc, name)));
    sp(loc, Exp_::Call(maccess, false, type_args, sp(loc, args)))
}

//**************************************************************************************************
// Types
//**************************************************************************************************

// Parse a Type:
//      Type =
//          <NameAccessChain> ("<" Comma<Type> ">")?
//          | "&" <Type>
//          | "&mut" <Type>
//          | "|" Comma<Type> "|" Type   (spec only)
//          | "(" Comma<Type> ")"
fn parse_type(context: &mut Context) -> Result<Type, Diagnostic> {
    let start_loc = context.tokens.start_loc();
    let t = match context.tokens.peek() {
        Tok::LParen => {
            let mut ts = parse_comma_list(context, Tok::LParen, Tok::RParen, parse_type, "a type")?;
            match ts.len() {
                0 => Type_::Unit,
                1 => ts.pop().unwrap().value,
                _ => Type_::Multiple(ts),
            }
        }
        Tok::Amp => {
            context.tokens.advance()?;
            let t = parse_type(context)?;
            Type_::Ref(false, Box::new(t))
        }
        Tok::AmpMut => {
            context.tokens.advance()?;
            let t = parse_type(context)?;
            Type_::Ref(true, Box::new(t))
        }
        Tok::Pipe => {
            let args = parse_comma_list(context, Tok::Pipe, Tok::Pipe, parse_type, "a type")?;
            let result = if is_start_of_type(context) {
                parse_type(context)?
            } else {
                spanned(
                    context.tokens.file_hash(),
                    start_loc,
                    context.tokens.start_loc(),
                    Type_::Unit,
                )
            };
            return Ok(spanned(
                context.tokens.file_hash(),
                start_loc,
                context.tokens.previous_end_loc(),
                Type_::Fun(args, Box::new(result)),
            ));
        }
        _ => {
            let tn = parse_name_access_chain(context, || "a type name")?;
            let tys = if context.tokens.peek() == Tok::Less {
                parse_comma_list(context, Tok::Less, Tok::Greater, parse_type, "a type")?
            } else {
                vec![]
            };
            Type_::Apply(Box::new(tn), tys)
        }
    };
    let end_loc = context.tokens.previous_end_loc();
    Ok(spanned(context.tokens.file_hash(), start_loc, end_loc, t))
}

/// Checks whether the next tokens looks like the start of a type. Must be aligned
/// with `parse_type`.
fn is_start_of_type(context: &mut Context) -> bool {
    matches!(
        context.tokens.peek(),
        Tok::LParen | Tok::Amp | Tok::AmpMut | Tok::Pipe | Tok::Identifier
    )
}

// Parse an optional list of type arguments.
//    OptionalTypeArgs = "<" Comma<Type> ">" | <empty>
fn parse_optional_type_args(context: &mut Context) -> Result<Option<Vec<Type>>, Diagnostic> {
    if context.tokens.peek() == Tok::Less {
        Ok(Some(parse_comma_list(
            context,
            Tok::Less,
            Tok::Greater,
            parse_type,
            "a type",
        )?))
    } else {
        Ok(None)
    }
}

fn token_to_ability(token: Tok, content: &str) -> Option<Ability_> {
    match (token, content) {
        (Tok::Copy, _) => Some(Ability_::Copy),
        (Tok::Identifier, Ability_::DROP) => Some(Ability_::Drop),
        (Tok::Identifier, Ability_::STORE) => Some(Ability_::Store),
        (Tok::Identifier, Ability_::KEY) => Some(Ability_::Key),
        _ => None,
    }
}

// Parse a type ability
//      Ability =
//          <Copy>
//          | "drop"
//          | "store"
//          | "key"
fn parse_ability(context: &mut Context) -> Result<Ability, Diagnostic> {
    let loc = current_token_loc(context.tokens);
    match token_to_ability(context.tokens.peek(), context.tokens.content()) {
        Some(ability) => {
            context.tokens.advance()?;
            Ok(sp(loc, ability))
        }
        None => {
            let msg = format!(
                "Unexpected {}. Expected a type ability, one of: 'copy', 'drop', 'store', or 'key'",
                current_token_error_string(context.tokens)
            );
            Err(diag!(Syntax::UnexpectedToken, (loc, msg),))
        }
    }
}

// Parse a type parameter:
//      TypeParameter =
//          <Identifier> <Constraint>?
//      Constraint =
//          ":" <Ability> (+ <Ability>)*
fn parse_type_parameter(context: &mut Context) -> Result<(Name, Vec<Ability>), Diagnostic> {
    let n = parse_identifier(context)?;

    let ability_constraints = if match_token(context.tokens, Tok::Colon)? {
        parse_list(
            context,
            |context| match context.tokens.peek() {
                Tok::Plus => {
                    context.tokens.advance()?;
                    Ok(true)
                }
                Tok::Greater | Tok::Comma => Ok(false),
                _ => Err(unexpected_token_error(
                    context.tokens,
                    &format!(
                        "one of: '{}', '{}', or '{}'",
                        Tok::Plus,
                        Tok::Greater,
                        Tok::Comma
                    ),
                )),
            },
            parse_ability,
        )?
    } else {
        vec![]
    };
    Ok((n, ability_constraints))
}

// Parse type parameter with optional phantom declaration:
//   TypeParameterWithPhantomDecl = "phantom"? <TypeParameter>
fn parse_type_parameter_with_phantom_decl(
    context: &mut Context,
) -> Result<StructTypeParameter, Diagnostic> {
    let is_phantom =
        if context.tokens.peek() == Tok::Identifier && context.tokens.content() == "phantom" {
            context.tokens.advance()?;
            true
        } else {
            false
        };
    let (name, constraints) = parse_type_parameter(context)?;
    Ok(StructTypeParameter {
        is_phantom,
        name,
        constraints,
    })
}

// Parse optional type parameter list.
//    OptionalTypeParameters = "<" Comma<TypeParameter> ">" | <empty>
fn parse_optional_type_parameters(
    context: &mut Context,
) -> Result<Vec<(Name, Vec<Ability>)>, Diagnostic> {
    if context.tokens.peek() == Tok::Less {
        parse_comma_list(
            context,
            Tok::Less,
            Tok::Greater,
            parse_type_parameter,
            "a type parameter",
        )
    } else {
        Ok(vec![])
    }
}

// Parse optional struct type parameters:
//    StructTypeParameter = "<" Comma<TypeParameterWithPhantomDecl> ">" | <empty>
fn parse_struct_type_parameters(
    context: &mut Context,
) -> Result<Vec<StructTypeParameter>, Diagnostic> {
    if context.tokens.peek() == Tok::Less {
        parse_comma_list(
            context,
            Tok::Less,
            Tok::Greater,
            parse_type_parameter_with_phantom_decl,
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
//          ( "macro" | "fun" )
//          <FunctionDefName> "(" Comma<Parameter> ")"
//          (":" <Type>)?
//          ("acquires" <NameAccessChain> ("," <NameAccessChain>)*)?
//          ("{" <Sequence> "}" | ";")
//
fn parse_function_decl(
    attributes: Vec<Attributes>,
    start_loc: usize,
    modifiers: Modifiers,
    context: &mut Context,
) -> Result<Function, Diagnostic> {
    let Modifiers { visibility, native } = modifiers;

    // "macro" | "fun" <FunctionDefName>
    let is_macro = if context.tokens.peek() == Tok::Macro {
        context.tokens.advance()?;
        true
    } else {
        consume_token(context.tokens, Tok::Fun)?;
        false
    };
    let name = FunctionName(parse_identifier(context)?);
    let type_parameters = parse_optional_type_parameters(context)?;

    // "(" Comma<Parameter> ")"
    let parameters = parse_comma_list(
        context,
        Tok::LParen,
        Tok::RParen,
        parse_parameter,
        "a function parameter",
    )?;

    // (":" <Type>)?
    let return_type = if match_token(context.tokens, Tok::Colon)? {
        parse_type(context)?
    } else {
        sp(name.loc(), Type_::Unit)
    };

    // ("acquires" (<NameAccessChain> ",")* <NameAccessChain> ","?
    let mut acquires = vec![];
    if match_token(context.tokens, Tok::Acquires)? {
        let follows_acquire = |tok| matches!(tok, Tok::Semicolon | Tok::LBrace);
        loop {
            acquires.push(parse_name_access_chain(context, || {
                "a resource struct name"
            })?);
            if follows_acquire(context.tokens.peek()) {
                break;
            }
            consume_token(context.tokens, Tok::Comma)?;
            if follows_acquire(context.tokens.peek()) {
                break;
            }
        }
    }

    let body = match native {
        Some(loc) => {
            consume_token(context.tokens, Tok::Semicolon)?;
            sp(loc, FunctionBody_::Native)
        }
        _ => {
            let start_loc = context.tokens.start_loc();
            consume_token(context.tokens, Tok::LBrace)?;
            let seq = parse_sequence(context)?;
            let end_loc = context.tokens.previous_end_loc();
            sp(
                make_loc(context.tokens.file_hash(), start_loc, end_loc),
                FunctionBody_::Defined(seq),
            )
        }
    };

    let signature = FunctionSignature {
        type_parameters,
        parameters,
        return_type,
    };

    let loc = make_loc(
        context.tokens.file_hash(),
        start_loc,
        context.tokens.previous_end_loc(),
    );
    Ok(Function {
        attributes,
        loc,
        visibility: visibility.unwrap_or(Visibility::Internal),
        signature,
        acquires,
        is_macro,
        name,
        body,
    })
}

// Parse a function parameter:
//      Parameter = <Var> ":" <Type>
fn parse_parameter(context: &mut Context) -> Result<(Var, Type), Diagnostic> {
    let v = parse_var(context)?;
    consume_token(context.tokens, Tok::Colon)?;
    let t = parse_type(context)?;
    Ok((v, t))
}

//**************************************************************************************************
// Structs
//**************************************************************************************************

// Parse a struct definition:
//      StructDecl =
//          "struct" <StructDefName> ("has" <Ability> (, <Ability>)+)?
//          ("{" Comma<FieldAnnot> "}" | ";")
//      StructDefName =
//          <Identifier> <OptionalTypeParameters>
fn parse_struct_decl(
    attributes: Vec<Attributes>,
    start_loc: usize,
    modifiers: Modifiers,
    context: &mut Context,
) -> Result<StructDefinition, Diagnostic> {
    let Modifiers { visibility, native } = modifiers;
    if let Some(vis) = visibility {
        let msg = format!(
            "Invalid struct declaration. Structs cannot have visibility modifiers as they are \
             always '{}'",
            Visibility::PUBLIC
        );
        context
            .env
            .add_diag(diag!(Syntax::InvalidModifier, (vis.loc().unwrap(), msg)));
    }

    consume_token(context.tokens, Tok::Struct)?;

    // <StructDefName>
    let name = StructName(parse_identifier(context)?);
    let type_parameters = parse_struct_type_parameters(context)?;

    let abilities = if context.tokens.peek() == Tok::Identifier && context.tokens.content() == "has"
    {
        context.tokens.advance()?;
        parse_list(
            context,
            |context| match context.tokens.peek() {
                Tok::Comma => {
                    context.tokens.advance()?;
                    Ok(true)
                }
                Tok::LBrace | Tok::Semicolon => Ok(false),
                _ => Err(unexpected_token_error(
                    context.tokens,
                    &format!(
                        "one of: '{}', '{}', or '{}'",
                        Tok::Comma,
                        Tok::LBrace,
                        Tok::Semicolon
                    ),
                )),
            },
            parse_ability,
        )?
    } else {
        vec![]
    };

    let fields = match native {
        Some(loc) => {
            consume_token(context.tokens, Tok::Semicolon)?;
            StructFields::Native(loc)
        }
        _ => {
            let list = parse_comma_list(
                context,
                Tok::LBrace,
                Tok::RBrace,
                parse_field_annot,
                "a field",
            )?;
            StructFields::Defined(list)
        }
    };

    let loc = make_loc(
        context.tokens.file_hash(),
        start_loc,
        context.tokens.previous_end_loc(),
    );
    Ok(StructDefinition {
        attributes,
        loc,
        abilities,
        name,
        type_parameters,
        fields,
    })
}

// Parse a field annotated with a type:
//      FieldAnnot = <DocComments> <Field> ":" <Type>
fn parse_field_annot(context: &mut Context) -> Result<(Field, Type), Diagnostic> {
    context.tokens.match_doc_comments();
    let f = parse_field(context)?;
    consume_token(context.tokens, Tok::Colon)?;
    let st = parse_type(context)?;
    Ok((f, st))
}

//**************************************************************************************************
// Constants
//**************************************************************************************************

// Parse a constant:
//      ConstantDecl = "const" <Identifier> ":" <Type> "=" <Exp> ";"
fn parse_constant_decl(
    attributes: Vec<Attributes>,
    start_loc: usize,
    modifiers: Modifiers,
    context: &mut Context,
) -> Result<Constant, Diagnostic> {
    let Modifiers { visibility, native } = modifiers;
    if let Some(vis) = visibility {
        let msg = "Invalid constant declaration. Constants cannot have visibility modifiers as \
                   they are always internal";
        context
            .env
            .add_diag(diag!(Syntax::InvalidModifier, (vis.loc().unwrap(), msg)));
    }
    if let Some(loc) = native {
        let msg = "Invalid constant declaration. 'native' constants are not supported";
        context
            .env
            .add_diag(diag!(Syntax::InvalidModifier, (loc, msg)));
    }
    consume_token(context.tokens, Tok::Const)?;
    let name = ConstantName(parse_identifier(context)?);
    consume_token(context.tokens, Tok::Colon)?;
    let signature = parse_type(context)?;
    consume_token(context.tokens, Tok::Equal)?;
    let value = parse_exp(context)?;
    consume_token(context.tokens, Tok::Semicolon)?;
    let loc = make_loc(
        context.tokens.file_hash(),
        start_loc,
        context.tokens.previous_end_loc(),
    );
    Ok(Constant {
        attributes,
        loc,
        signature,
        name,
        value,
    })
}

//**************************************************************************************************
// AddressBlock
//**************************************************************************************************

// Parse an address block:
//      AddressBlock =
//          "address" <LeadingNameAccess> "{" (<Attributes> <Module>)* "}"
//
// Note that "address" is not a token.
fn parse_address_block(
    attributes: Vec<Attributes>,
    context: &mut Context,
) -> Result<AddressDefinition, Diagnostic> {
    const UNEXPECTED_TOKEN: &str = "Invalid code unit. Expected 'address', 'module', or 'script'";
    if context.tokens.peek() != Tok::Identifier {
        let start = context.tokens.start_loc();
        let end = start + context.tokens.content().len();
        let loc = make_loc(context.tokens.file_hash(), start, end);
        let msg = format!(
            "{}. Got {}",
            UNEXPECTED_TOKEN,
            current_token_error_string(context.tokens)
        );
        return Err(diag!(Syntax::UnexpectedToken, (loc, msg)));
    }
    let addr_name = parse_identifier(context)?;
    if addr_name.value.as_str() != "address" {
        let msg = format!("{}. Got '{}'", UNEXPECTED_TOKEN, addr_name.value);
        return Err(diag!(Syntax::UnexpectedToken, (addr_name.loc, msg)));
    }
    let start_loc = context.tokens.start_loc();
    let addr = parse_leading_name_access(context)?;
    let end_loc = context.tokens.previous_end_loc();
    let loc = make_loc(context.tokens.file_hash(), start_loc, end_loc);

    let modules = match context.tokens.peek() {
        Tok::LBrace => {
            context.tokens.advance()?;
            let mut modules = vec![];
            while context.tokens.peek() != Tok::RBrace {
                let attributes = parse_attributes(context)?;
                modules.push(parse_module(attributes, context)?);
            }
            consume_token(context.tokens, Tok::RBrace)?;
            modules
        }
        _ => return Err(unexpected_token_error(context.tokens, "'{'")),
    };

    Ok(AddressDefinition {
        attributes,
        loc,
        addr,
        modules,
    })
}

//**************************************************************************************************
// Friends
//**************************************************************************************************

// Parse a friend declaration:
//      FriendDecl =
//          "friend" <NameAccessChain> ";"
fn parse_friend_decl(
    attributes: Vec<Attributes>,
    context: &mut Context,
) -> Result<FriendDecl, Diagnostic> {
    let start_loc = context.tokens.start_loc();
    consume_token(context.tokens, Tok::Friend)?;
    let friend = parse_name_access_chain(context, || "a friend declaration")?;
    consume_token(context.tokens, Tok::Semicolon)?;
    let loc = make_loc(
        context.tokens.file_hash(),
        start_loc,
        context.tokens.previous_end_loc(),
    );
    Ok(FriendDecl {
        attributes,
        loc,
        friend,
    })
}

//**************************************************************************************************
// Modules
//**************************************************************************************************

// Parse a use declaration:
//      UseDecl =
//          "use" <ModuleIdent> <UseAlias> ";" |
//          "use" <ModuleIdent> :: <UseMember> ";" |
//          "use" <ModuleIdent> :: "{" Comma<UseMember> "}" ";"
fn parse_use_decl(
    attributes: Vec<Attributes>,
    context: &mut Context,
) -> Result<UseDecl, Diagnostic> {
    consume_token(context.tokens, Tok::Use)?;
    let ident = parse_module_ident(context)?;
    let alias_opt = parse_use_alias(context)?;
    let use_ = match (&alias_opt, context.tokens.peek()) {
        (None, Tok::ColonColon) => {
            consume_token(context.tokens, Tok::ColonColon)?;
            let sub_uses = match context.tokens.peek() {
                Tok::LBrace => parse_comma_list(
                    context,
                    Tok::LBrace,
                    Tok::RBrace,
                    parse_use_member,
                    "a module member alias",
                )?,
                _ => vec![parse_use_member(context)?],
            };
            Use::Members(ident, sub_uses)
        }
        _ => Use::Module(ident, alias_opt.map(ModuleName)),
    };
    consume_token(context.tokens, Tok::Semicolon)?;
    Ok(UseDecl { attributes, use_ })
}

// Parse an alias for a module member:
//      UseMember = <Identifier> <UseAlias>
fn parse_use_member(context: &mut Context) -> Result<(Name, Option<Name>), Diagnostic> {
    let member = parse_identifier(context)?;
    let alias_opt = parse_use_alias(context)?;
    Ok((member, alias_opt))
}

// Parse an 'as' use alias:
//      UseAlias = ("as" <Identifier>)?
fn parse_use_alias(context: &mut Context) -> Result<Option<Name>, Diagnostic> {
    Ok(if context.tokens.peek() == Tok::As {
        context.tokens.advance()?;
        Some(parse_identifier(context)?)
    } else {
        None
    })
}

// Parse a module:
//      Module =
//          <DocComments> ( "spec" | "module") (<LeadingNameAccess>::)?<ModuleName> "{"
//              ( <Attributes>
//                  ( <UseDecl> | <FriendDecl> | <SpecBlock> |
//                    <DocComments> <ModuleMemberModifiers>
//                        (<ConstantDecl> | <StructDecl> | <FunctionDecl>) )
//                  )
//              )*
//          "}"
fn parse_module(
    attributes: Vec<Attributes>,
    context: &mut Context,
) -> Result<ModuleDefinition, Diagnostic> {
    context.tokens.match_doc_comments();
    let start_loc = context.tokens.start_loc();

    let is_spec_module = if context.tokens.peek() == Tok::Spec {
        context.tokens.advance()?;
        true
    } else {
        consume_token(context.tokens, Tok::Module)?;
        false
    };
    let sp!(n1_loc, n1_) = parse_leading_name_access(context)?;
    let (address, name) = match (n1_, context.tokens.peek()) {
        (addr_ @ LeadingNameAccess_::AnonymousAddress(_), _)
        | (addr_ @ LeadingNameAccess_::Name(_), Tok::ColonColon) => {
            let addr = sp(n1_loc, addr_);
            consume_token(context.tokens, Tok::ColonColon)?;
            let name = parse_module_name(context)?;
            (Some(addr), name)
        }
        (LeadingNameAccess_::Name(name), _) => (None, ModuleName(name)),
    };
    consume_token(context.tokens, Tok::LBrace)?;

    let mut members = vec![];
    while context.tokens.peek() != Tok::RBrace {
        members.push({
            let attributes = parse_attributes(context)?;
            match context.tokens.peek() {
                // Top-level specification constructs
                Tok::Invariant => {
                    context.tokens.match_doc_comments();
                    ModuleMember::Spec(singleton_module_spec_block(
                        context,
                        context.tokens.start_loc(),
                        attributes,
                        parse_invariant,
                    )?)
                }
                Tok::Spec => {
                    match context.tokens.lookahead() {
                        Ok(Tok::Fun) | Ok(Tok::Native) => {
                            context.tokens.match_doc_comments();
                            let start_loc = context.tokens.start_loc();
                            context.tokens.advance()?;
                            // Add an extra check for better error message
                            // if old syntax is used
                            if context.tokens.lookahead2() == Ok((Tok::Identifier, Tok::LBrace)) {
                                return Err(unexpected_token_error(
                                    context.tokens,
                                    "only 'spec', drop the 'fun' keyword",
                                ));
                            }
                            ModuleMember::Spec(singleton_module_spec_block(
                                context,
                                start_loc,
                                attributes,
                                parse_spec_function,
                            )?)
                        }
                        _ => {
                            // Regular spec block
                            ModuleMember::Spec(parse_spec_block(attributes, context)?)
                        }
                    }
                }
                // Regular move constructs
                Tok::Use => ModuleMember::Use(parse_use_decl(attributes, context)?),
                Tok::Friend => ModuleMember::Friend(parse_friend_decl(attributes, context)?),
                _ => {
                    context.tokens.match_doc_comments();
                    let start_loc = context.tokens.start_loc();
                    let modifiers = parse_module_member_modifiers(context)?;
                    match context.tokens.peek() {
                        Tok::Const => ModuleMember::Constant(parse_constant_decl(
                            attributes, start_loc, modifiers, context,
                        )?),
                        Tok::Fun | Tok::Macro => ModuleMember::Function(parse_function_decl(
                            attributes, start_loc, modifiers, context,
                        )?),
                        Tok::Struct => ModuleMember::Struct(parse_struct_decl(
                            attributes, start_loc, modifiers, context,
                        )?),
                        _ => {
                            return Err(unexpected_token_error(
                                context.tokens,
                                &format!(
                                    "a module member: '{}', '{}', '{}', '{}', '{}', '{}', or '{}'",
                                    Tok::Spec,
                                    Tok::Use,
                                    Tok::Friend,
                                    Tok::Const,
                                    Tok::Fun,
                                    Tok::Macro,
                                    Tok::Struct
                                ),
                            ))
                        }
                    }
                }
            }
        })
    }
    consume_token(context.tokens, Tok::RBrace)?;
    let loc = make_loc(
        context.tokens.file_hash(),
        start_loc,
        context.tokens.previous_end_loc(),
    );
    Ok(ModuleDefinition {
        attributes,
        loc,
        address,
        name,
        is_spec_module,
        members,
    })
}

//**************************************************************************************************
// Scripts
//**************************************************************************************************

// Parse a script:
//      Script =
//          "script" "{"
//              (<Attributes> <UseDecl>)*
//              (<Attributes> <ConstantDecl>)*
//              <Attributes> <DocComments> <ModuleMemberModifiers> <FunctionDecl>
//              (<Attributes> <SpecBlock>)*
//          "}"
fn parse_script(
    script_attributes: Vec<Attributes>,
    context: &mut Context,
) -> Result<Script, Diagnostic> {
    let start_loc = context.tokens.start_loc();

    consume_token(context.tokens, Tok::Script)?;
    consume_token(context.tokens, Tok::LBrace)?;

    let mut uses = vec![];
    let mut next_item_attributes = parse_attributes(context)?;
    while context.tokens.peek() == Tok::Use {
        uses.push(parse_use_decl(next_item_attributes, context)?);
        next_item_attributes = parse_attributes(context)?;
    }
    let mut constants = vec![];
    while context.tokens.peek() == Tok::Const {
        let start_loc = context.tokens.start_loc();
        constants.push(parse_constant_decl(
            next_item_attributes,
            start_loc,
            Modifiers::empty(),
            context,
        )?);
        next_item_attributes = parse_attributes(context)?;
    }

    context.tokens.match_doc_comments(); // match doc comments to script function
    let function_start_loc = context.tokens.start_loc();
    let modifiers = parse_module_member_modifiers(context)?;
    // don't need to check native modifier, it is checked later
    let function =
        parse_function_decl(next_item_attributes, function_start_loc, modifiers, context)?;

    let mut specs = vec![];
    while context.tokens.peek() == Tok::NumSign || context.tokens.peek() == Tok::Spec {
        let attributes = parse_attributes(context)?;
        specs.push(parse_spec_block(attributes, context)?);
    }

    if context.tokens.peek() != Tok::RBrace {
        let loc = current_token_loc(context.tokens);
        let msg = "Unexpected characters after end of 'script' function";
        return Err(diag!(Syntax::UnexpectedToken, (loc, msg)));
    }
    consume_token(context.tokens, Tok::RBrace)?;

    let loc = make_loc(
        context.tokens.file_hash(),
        start_loc,
        context.tokens.previous_end_loc(),
    );
    Ok(Script {
        attributes: script_attributes,
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
fn parse_spec_block(
    attributes: Vec<Attributes>,
    context: &mut Context,
) -> Result<SpecBlock, Diagnostic> {
    context.tokens.match_doc_comments();
    let start_loc = context.tokens.start_loc();
    consume_token(context.tokens, Tok::Spec)?;
    let target_start_loc = context.tokens.start_loc();
    let target_ = match context.tokens.peek() {
        Tok::Fun => {
            return Err(unexpected_token_error(
                context.tokens,
                "only 'spec', drop the 'fun' keyword",
            ));
        }
        Tok::Struct => {
            return Err(unexpected_token_error(
                context.tokens,
                "only 'spec', drop the 'struct' keyword",
            ));
        }
        Tok::Module => {
            context.tokens.advance()?;
            SpecBlockTarget_::Module
        }
        Tok::Identifier if context.tokens.content() == "schema" => {
            context.tokens.advance()?;
            let name = parse_identifier(context)?;
            let type_parameters = parse_optional_type_parameters(context)?;
            SpecBlockTarget_::Schema(name, type_parameters)
        }
        Tok::Identifier => {
            let name = parse_identifier(context)?;
            let signature = parse_spec_target_signature_opt(&name.loc, context)?;
            SpecBlockTarget_::Member(name, signature)
        }
        Tok::LBrace => SpecBlockTarget_::Code,
        _ => {
            return Err(unexpected_token_error(
                context.tokens,
                "one of `module`, `struct`, `fun`, `schema`, or `{`",
            ));
        }
    };
    let target = spanned(
        context.tokens.file_hash(),
        target_start_loc,
        match target_ {
            SpecBlockTarget_::Code => target_start_loc,
            _ => context.tokens.previous_end_loc(),
        },
        target_,
    );

    consume_token(context.tokens, Tok::LBrace)?;
    let mut uses = vec![];
    while context.tokens.peek() == Tok::Use {
        uses.push(parse_use_decl(vec![], context)?);
    }
    let mut members = vec![];
    while context.tokens.peek() != Tok::RBrace {
        members.push(parse_spec_block_member(context)?);
    }
    consume_token(context.tokens, Tok::RBrace)?;
    Ok(spanned(
        context.tokens.file_hash(),
        start_loc,
        context.tokens.previous_end_loc(),
        SpecBlock_ {
            attributes,
            target,
            uses,
            members,
        },
    ))
}

fn parse_spec_target_signature_opt(
    loc: &Loc,
    context: &mut Context,
) -> Result<Option<Box<FunctionSignature>>, Diagnostic> {
    match context.tokens.peek() {
        Tok::Less | Tok::LParen => {
            let type_parameters = parse_optional_type_parameters(context)?;
            // "(" Comma<Parameter> ")"
            let parameters = parse_comma_list(
                context,
                Tok::LParen,
                Tok::RParen,
                parse_parameter,
                "a function parameter",
            )?;
            // (":" <Type>)?
            let return_type = if match_token(context.tokens, Tok::Colon)? {
                parse_type(context)?
            } else {
                sp(*loc, Type_::Unit)
            };
            Ok(Some(Box::new(FunctionSignature {
                type_parameters,
                parameters,
                return_type,
            })))
        }
        _ => Ok(None),
    }
}

// Parse a spec block member:
//    SpecBlockMember = <DocComments> ( <Invariant> | <Condition> | <SpecFunction> | <SpecVariable>
//                                   | <SpecInclude> | <SpecApply> | <SpecPragma> | <SpecLet>
//                                   | <SpecUpdate> | <SpecAxiom> )
fn parse_spec_block_member(context: &mut Context) -> Result<SpecBlockMember, Diagnostic> {
    context.tokens.match_doc_comments();
    match context.tokens.peek() {
        Tok::Invariant => parse_invariant(context),
        Tok::Let => parse_spec_let(context),
        Tok::Fun | Tok::Native => parse_spec_function(context),
        Tok::Identifier => match context.tokens.content() {
            "assert" | "assume" | "decreases" | "aborts_if" | "aborts_with" | "succeeds_if"
            | "modifies" | "emits" | "ensures" | "requires" => parse_condition(context),
            "axiom" => parse_axiom(context),
            "include" => parse_spec_include(context),
            "apply" => parse_spec_apply(context),
            "pragma" => parse_spec_pragma(context),
            "global" | "local" => parse_spec_variable(context),
            "update" => parse_spec_update(context),
            _ => {
                // local is optional but supported to be able to declare variables which are
                // named like the weak keywords above
                parse_spec_variable(context)
            }
        },
        _ => Err(unexpected_token_error(
            context.tokens,
            "one of `assert`, `assume`, `decreases`, `aborts_if`, `aborts_with`, `succeeds_if`, \
             `modifies`, `emits`, `ensures`, `requires`, `include`, `apply`, `pragma`, `global`, \
             or a name",
        )),
    }
}

// Parse a specification condition:
//    SpecCondition =
//        ("assert" | "assume" | "ensures" | "requires" ) <ConditionProperties> <Exp> ";"
//      | "aborts_if" <ConditionProperties> <Exp> ["with" <Exp>] ";"
//      | "aborts_with" <ConditionProperties> <Exp> [Comma <Exp>]* ";"
//      | "decreases" <ConditionProperties> <Exp> ";"
//      | "emits" <ConditionProperties> <Exp> "to" <Exp> [If <Exp>] ";"
fn parse_condition(context: &mut Context) -> Result<SpecBlockMember, Diagnostic> {
    let start_loc = context.tokens.start_loc();
    let kind_ = match context.tokens.content() {
        "assert" => SpecConditionKind_::Assert,
        "assume" => SpecConditionKind_::Assume,
        "decreases" => SpecConditionKind_::Decreases,
        "aborts_if" => SpecConditionKind_::AbortsIf,
        "aborts_with" => SpecConditionKind_::AbortsWith,
        "succeeds_if" => SpecConditionKind_::SucceedsIf,
        "modifies" => SpecConditionKind_::Modifies,
        "emits" => SpecConditionKind_::Emits,
        "ensures" => SpecConditionKind_::Ensures,
        "requires" => SpecConditionKind_::Requires,
        _ => unreachable!(),
    };
    context.tokens.advance()?;
    let kind = spanned(
        context.tokens.file_hash(),
        start_loc,
        context.tokens.previous_end_loc(),
        kind_.clone(),
    );
    let properties = parse_condition_properties(context)?;
    let exp = if kind_ == SpecConditionKind_::AbortsWith || kind_ == SpecConditionKind_::Modifies {
        // Use a dummy expression as a placeholder for this field.
        let loc = make_loc(context.tokens.file_hash(), start_loc, start_loc + 1);
        sp(loc, Exp_::Value(sp(loc, Value_::Bool(false))))
    } else {
        parse_exp(context)?
    };
    let additional_exps = if kind_ == SpecConditionKind_::AbortsIf
        && context.tokens.peek() == Tok::Identifier
        && context.tokens.content() == "with"
    {
        context.tokens.advance()?;
        let codes = vec![parse_exp(context)?];
        consume_token(context.tokens, Tok::Semicolon)?;
        codes
    } else if kind_ == SpecConditionKind_::AbortsWith || kind_ == SpecConditionKind_::Modifies {
        parse_comma_list_after_start(
            context,
            context.tokens.start_loc(),
            context.tokens.peek(),
            Tok::Semicolon,
            parse_exp,
            "an aborts code or modifies target",
        )?
    } else if kind_ == SpecConditionKind_::Emits {
        consume_identifier(context.tokens, "to")?;
        let mut additional_exps = vec![parse_exp(context)?];
        if match_token(context.tokens, Tok::If)? {
            additional_exps.push(parse_exp(context)?);
        }
        consume_token(context.tokens, Tok::Semicolon)?;
        additional_exps
    } else {
        consume_token(context.tokens, Tok::Semicolon)?;
        vec![]
    };
    let end_loc = context.tokens.previous_end_loc();
    Ok(spanned(
        context.tokens.file_hash(),
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
fn parse_condition_properties(context: &mut Context) -> Result<Vec<PragmaProperty>, Diagnostic> {
    let properties = if context.tokens.peek() == Tok::LBracket {
        parse_comma_list(
            context,
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

// Parse an axiom:
//     a = "axiom" <OptionalTypeParameters> <ConditionProperties> <Exp> ";"
fn parse_axiom(context: &mut Context) -> Result<SpecBlockMember, Diagnostic> {
    let start_loc = context.tokens.start_loc();
    consume_identifier(context.tokens, "axiom")?;
    let type_parameters = parse_optional_type_parameters(context)?;
    let kind = spanned(
        context.tokens.file_hash(),
        start_loc,
        context.tokens.previous_end_loc(),
        SpecConditionKind_::Axiom(type_parameters),
    );
    let properties = parse_condition_properties(context)?;
    let exp = parse_exp(context)?;
    consume_token(context.tokens, Tok::Semicolon)?;
    Ok(spanned(
        context.tokens.file_hash(),
        start_loc,
        context.tokens.previous_end_loc(),
        SpecBlockMember_::Condition {
            kind,
            properties,
            exp,
            additional_exps: vec![],
        },
    ))
}

// Parse an invariant:
//     Invariant = "invariant" <OptionalTypeParameters> [ "update" ] <ConditionProperties> <Exp> ";"
fn parse_invariant(context: &mut Context) -> Result<SpecBlockMember, Diagnostic> {
    let start_loc = context.tokens.start_loc();
    consume_token(context.tokens, Tok::Invariant)?;
    let type_parameters = parse_optional_type_parameters(context)?;
    let kind_ = match context.tokens.peek() {
        Tok::Identifier if context.tokens.content() == "update" => {
            context.tokens.advance()?;
            SpecConditionKind_::InvariantUpdate(type_parameters)
        }
        _ => SpecConditionKind_::Invariant(type_parameters),
    };
    let kind = spanned(
        context.tokens.file_hash(),
        start_loc,
        context.tokens.previous_end_loc(),
        kind_,
    );
    let properties = parse_condition_properties(context)?;
    let exp = parse_exp(context)?;
    consume_token(context.tokens, Tok::Semicolon)?;
    Ok(spanned(
        context.tokens.file_hash(),
        start_loc,
        context.tokens.previous_end_loc(),
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
fn parse_spec_function(context: &mut Context) -> Result<SpecBlockMember, Diagnostic> {
    let start_loc = context.tokens.start_loc();
    let native_opt = consume_optional_token_with_loc(context.tokens, Tok::Native)?;
    consume_token(context.tokens, Tok::Fun)?;
    let name = FunctionName(parse_identifier(context)?);
    let type_parameters = parse_optional_type_parameters(context)?;
    // "(" Comma<Parameter> ")"
    let parameters = parse_comma_list(
        context,
        Tok::LParen,
        Tok::RParen,
        parse_parameter,
        "a function parameter",
    )?;

    // ":" <Type>)
    consume_token(context.tokens, Tok::Colon)?;
    let return_type = parse_type(context)?;

    let body_start_loc = context.tokens.start_loc();
    let no_body = context.tokens.peek() != Tok::LBrace;
    let (uninterpreted, body_) = if native_opt.is_some() || no_body {
        consume_token(context.tokens, Tok::Semicolon)?;
        (native_opt.is_none(), FunctionBody_::Native)
    } else {
        consume_token(context.tokens, Tok::LBrace)?;
        let seq = parse_sequence(context)?;
        (false, FunctionBody_::Defined(seq))
    };
    let body = spanned(
        context.tokens.file_hash(),
        body_start_loc,
        context.tokens.previous_end_loc(),
        body_,
    );

    let signature = FunctionSignature {
        type_parameters,
        parameters,
        return_type,
    };

    Ok(spanned(
        context.tokens.file_hash(),
        start_loc,
        context.tokens.previous_end_loc(),
        SpecBlockMember_::Function {
            signature,
            uninterpreted,
            name,
            body,
        },
    ))
}

// Parse a specification variable.
//     SpecVariable = ( "global" | "local" )?
//                    <Identifier> <OptionalTypeParameters>
//                    ":" <Type>
//                    [ "=" Exp ]  // global only
//                    ";"
fn parse_spec_variable(context: &mut Context) -> Result<SpecBlockMember, Diagnostic> {
    let start_loc = context.tokens.start_loc();
    let is_global = match context.tokens.content() {
        "global" => {
            consume_token(context.tokens, Tok::Identifier)?;
            true
        }
        "local" => {
            consume_token(context.tokens, Tok::Identifier)?;
            false
        }
        _ => false,
    };
    let name = parse_identifier(context)?;
    let type_parameters = parse_optional_type_parameters(context)?;
    consume_token(context.tokens, Tok::Colon)?;
    let type_ = parse_type(context)?;
    let init = if is_global && context.tokens.peek() == Tok::Equal {
        context.tokens.advance()?;
        Some(parse_exp(context)?)
    } else {
        None
    };

    consume_token(context.tokens, Tok::Semicolon)?;
    Ok(spanned(
        context.tokens.file_hash(),
        start_loc,
        context.tokens.previous_end_loc(),
        SpecBlockMember_::Variable {
            is_global,
            name,
            type_parameters,
            type_,
            init,
        },
    ))
}

// Parse a specification update.
//     SpecUpdate = "update" <Exp> = <Exp> ";"
fn parse_spec_update(context: &mut Context) -> Result<SpecBlockMember, Diagnostic> {
    let start_loc = context.tokens.start_loc();
    consume_token(context.tokens, Tok::Identifier)?;
    let lhs = parse_unary_exp(context)?;
    consume_token(context.tokens, Tok::Equal)?;
    let rhs = parse_exp(context)?;
    consume_token(context.tokens, Tok::Semicolon)?;
    Ok(spanned(
        context.tokens.file_hash(),
        start_loc,
        context.tokens.previous_end_loc(),
        SpecBlockMember_::Update { lhs, rhs },
    ))
}

// Parse a specification let.
//     SpecLet =  "let" [ "post" ] <Identifier> "=" <Exp> ";"
fn parse_spec_let(context: &mut Context) -> Result<SpecBlockMember, Diagnostic> {
    let start_loc = context.tokens.start_loc();
    context.tokens.advance()?;
    let post_state =
        if context.tokens.peek() == Tok::Identifier && context.tokens.content() == "post" {
            context.tokens.advance()?;
            true
        } else {
            false
        };
    let name = parse_identifier(context)?;
    consume_token(context.tokens, Tok::Equal)?;
    let def = parse_exp(context)?;
    consume_token(context.tokens, Tok::Semicolon)?;
    Ok(spanned(
        context.tokens.file_hash(),
        start_loc,
        context.tokens.previous_end_loc(),
        SpecBlockMember_::Let {
            name,
            post_state,
            def,
        },
    ))
}

// Parse a specification schema include.
//    SpecInclude = "include" <Exp>
fn parse_spec_include(context: &mut Context) -> Result<SpecBlockMember, Diagnostic> {
    let start_loc = context.tokens.start_loc();
    consume_identifier(context.tokens, "include")?;
    let properties = parse_condition_properties(context)?;
    let exp = parse_exp(context)?;
    consume_token(context.tokens, Tok::Semicolon)?;
    Ok(spanned(
        context.tokens.file_hash(),
        start_loc,
        context.tokens.previous_end_loc(),
        SpecBlockMember_::Include { properties, exp },
    ))
}

// Parse a specification schema apply.
//    SpecApply = "apply" <Exp> "to" Comma<SpecApplyPattern>
//                                   ( "except" Comma<SpecApplyPattern> )? ";"
fn parse_spec_apply(context: &mut Context) -> Result<SpecBlockMember, Diagnostic> {
    let start_loc = context.tokens.start_loc();
    consume_identifier(context.tokens, "apply")?;
    let exp = parse_exp(context)?;
    consume_identifier(context.tokens, "to")?;
    let parse_patterns = |context: &mut Context| {
        parse_list(
            context,
            |context| {
                if context.tokens.peek() == Tok::Comma {
                    context.tokens.advance()?;
                    Ok(true)
                } else {
                    Ok(false)
                }
            },
            parse_spec_apply_pattern,
        )
    };
    let patterns = parse_patterns(context)?;
    let exclusion_patterns =
        if context.tokens.peek() == Tok::Identifier && context.tokens.content() == "except" {
            context.tokens.advance()?;
            parse_patterns(context)?
        } else {
            vec![]
        };
    consume_token(context.tokens, Tok::Semicolon)?;
    Ok(spanned(
        context.tokens.file_hash(),
        start_loc,
        context.tokens.previous_end_loc(),
        SpecBlockMember_::Apply {
            exp,
            patterns,
            exclusion_patterns,
        },
    ))
}

// Parse a function pattern:
//     SpecApplyPattern = <SpecApplyFragment>+ <OptionalTypeArgs>
fn parse_spec_apply_pattern(context: &mut Context) -> Result<SpecApplyPattern, Diagnostic> {
    let start_loc = context.tokens.start_loc();
    // TODO: update the visibility parsing in the spec as well
    let public_opt = consume_optional_token_with_loc(context.tokens, Tok::Public)?;
    let visibility = if let Some(loc) = public_opt {
        Some(Visibility::Public(loc))
    } else if context.tokens.peek() == Tok::Identifier && context.tokens.content() == "internal" {
        // Its not ideal right now that we do not have a loc here, but acceptable for what
        // we are doing with this in specs.
        context.tokens.advance()?;
        Some(Visibility::Internal)
    } else {
        None
    };
    let mut last_end = context.tokens.start_loc() + context.tokens.content().len();
    let name_pattern = parse_list(
        context,
        |context| {
            // We need name fragments followed by each other without space. So we do some
            // magic here similar as with `>>` based on token distance.
            let start_loc = context.tokens.start_loc();
            let adjacent = last_end == start_loc;
            last_end = start_loc + context.tokens.content().len();
            Ok(adjacent && [Tok::Identifier, Tok::Star].contains(&context.tokens.peek()))
        },
        parse_spec_apply_fragment,
    )?;
    let type_parameters = parse_optional_type_parameters(context)?;
    Ok(spanned(
        context.tokens.file_hash(),
        start_loc,
        context.tokens.previous_end_loc(),
        SpecApplyPattern_ {
            visibility,
            name_pattern,
            type_parameters,
        },
    ))
}

// Parse a name pattern fragment
//     SpecApplyFragment = <Identifier> | "*"
fn parse_spec_apply_fragment(context: &mut Context) -> Result<SpecApplyFragment, Diagnostic> {
    let start_loc = context.tokens.start_loc();
    let fragment = match context.tokens.peek() {
        Tok::Identifier => SpecApplyFragment_::NamePart(parse_identifier(context)?),
        Tok::Star => {
            context.tokens.advance()?;
            SpecApplyFragment_::Wildcard
        }
        _ => {
            return Err(unexpected_token_error(
                context.tokens,
                "a name fragment or `*`",
            ))
        }
    };
    Ok(spanned(
        context.tokens.file_hash(),
        start_loc,
        context.tokens.previous_end_loc(),
        fragment,
    ))
}

// Parse a specification pragma:
//    SpecPragma = "pragma" Comma<SpecPragmaProperty> ";"
fn parse_spec_pragma(context: &mut Context) -> Result<SpecBlockMember, Diagnostic> {
    let start_loc = context.tokens.start_loc();
    consume_identifier(context.tokens, "pragma")?;
    let properties = parse_comma_list_after_start(
        context,
        start_loc,
        Tok::Identifier,
        Tok::Semicolon,
        parse_spec_property,
        "a pragma property",
    )?;
    Ok(spanned(
        context.tokens.file_hash(),
        start_loc,
        context.tokens.previous_end_loc(),
        SpecBlockMember_::Pragma { properties },
    ))
}

// Parse a specification pragma property:
//    SpecPragmaProperty = <Identifier> ( "=" <Value> | <NameAccessChain> )?
fn parse_spec_property(context: &mut Context) -> Result<PragmaProperty, Diagnostic> {
    let start_loc = context.tokens.start_loc();
    let name = match consume_optional_token_with_loc(context.tokens, Tok::Friend)? {
        // special treatment for `pragma friend = ...` as friend is a keyword
        // TODO: this might violate the assumption that a keyword can never be a name.
        Some(loc) => Name::new(loc, Symbol::from("friend")),
        None => parse_identifier(context)?,
    };
    let value = if context.tokens.peek() == Tok::Equal {
        context.tokens.advance()?;
        match context.tokens.peek() {
            Tok::AtSign | Tok::True | Tok::False | Tok::NumTypedValue | Tok::ByteStringValue => {
                Some(PragmaValue::Literal(parse_value(context)?))
            }
            Tok::NumValue
                if !context
                    .tokens
                    .lookahead()
                    .map(|tok| tok == Tok::ColonColon)
                    .unwrap_or(false) =>
            {
                Some(PragmaValue::Literal(parse_value(context)?))
            }
            _ => {
                // Parse as a module access for a possibly qualified identifier
                Some(PragmaValue::Ident(parse_name_access_chain(
                    context,
                    || "an identifier as pragma value",
                )?))
            }
        }
    } else {
        None
    };
    Ok(spanned(
        context.tokens.file_hash(),
        start_loc,
        context.tokens.previous_end_loc(),
        PragmaProperty_ { name, value },
    ))
}

/// Creates a module spec block for a single member.
fn singleton_module_spec_block(
    context: &mut Context,
    start_loc: usize,
    attributes: Vec<Attributes>,
    member_parser: impl Fn(&mut Context) -> Result<SpecBlockMember, Diagnostic>,
) -> Result<SpecBlock, Diagnostic> {
    let member = member_parser(context)?;
    let end_loc = context.tokens.previous_end_loc();
    Ok(spanned(
        context.tokens.file_hash(),
        start_loc,
        end_loc,
        SpecBlock_ {
            attributes,
            target: spanned(
                context.tokens.file_hash(),
                start_loc,
                start_loc,
                SpecBlockTarget_::Module,
            ),
            uses: vec![],
            members: vec![member],
        },
    ))
}

//**************************************************************************************************
// File
//**************************************************************************************************

// Parse a file:
//      File =
//          (<Attributes> (<AddressBlock> | <Module> | <Script>))*
fn parse_file(context: &mut Context) -> Result<Vec<Definition>, Diagnostic> {
    let mut defs = vec![];
    while context.tokens.peek() != Tok::EOF {
        let attributes = parse_attributes(context)?;
        defs.push(match context.tokens.peek() {
            Tok::Spec | Tok::Module => Definition::Module(parse_module(attributes, context)?),
            Tok::Script => Definition::Script(parse_script(attributes, context)?),
            _ => Definition::Address(parse_address_block(attributes, context)?),
        })
    }
    Ok(defs)
}

/// Parse the `input` string as a file of Move source code and return the
/// result as either a pair of FileDefinition and doc comments or some Diagnostics. The `file` name
/// is used to identify source locations in error messages.
pub fn parse_file_string(
    env: &mut CompilationEnv,
    file_hash: FileHash,
    input: &str,
) -> Result<(Vec<Definition>, MatchedFileCommentMap), Diagnostics> {
    let mut tokens = Lexer::new(input, file_hash);
    match tokens.advance() {
        Err(err) => Err(Diagnostics::from(vec![err])),
        Ok(..) => Ok(()),
    }?;
    match parse_file(&mut Context::new(env, &mut tokens)) {
        Err(err) => Err(Diagnostics::from(vec![err])),
        Ok(def) => Ok((def, tokens.check_and_get_doc_comments(env))),
    }
}
