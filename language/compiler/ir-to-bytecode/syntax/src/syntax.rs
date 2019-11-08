// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use codespan::{ByteIndex, Span};
use std::fmt;
use std::str::FromStr;

use crate::ast::{
    parse_field, BinOp, Block, Block_, Builtin, Cmd, CopyableVal, CopyableVal_, Exp, Exp_, Field_,
    Fields, Function, FunctionBody, FunctionCall, FunctionCall_, FunctionName, FunctionVisibility,
    Function_, IfElse, ImportDefinition, Kind, LValue, LValue_, Loop, ModuleDefinition,
    ModuleIdent, ModuleName, Program, QualifiedModuleIdent, QualifiedStructIdent, Script,
    ScriptOrModule, Spanned, Statement, StructDefinition, StructDefinition_, StructName, Type,
    TypeVar, TypeVar_, UnaryOp, Var, Var_, While,
};
use crate::lexer::*;
use hex;
use libra_types::{account_address::AccountAddress, byte_array::ByteArray};

// FIXME: The following simplified version of ParseError copied from
// lalrpop-util should be replaced.

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParseError<L, E> {
    InvalidToken { location: L },
    User { error: E },
}

impl<L, E> fmt::Display for ParseError<L, E>
where
    L: fmt::Display,
    E: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::ParseError::*;
        match *self {
            User { ref error } => write!(f, "{}", error),
            InvalidToken { ref location } => write!(f, "Invalid token at {}", location),
        }
    }
}

fn spanned<T>(start: usize, end: usize, value: T) -> Spanned<T> {
    Spanned {
        value,
        span: Span::new(ByteIndex(start as u32), ByteIndex(end as u32)),
    }
}

fn consume_token<'input>(
    tokens: &mut Lexer<'input>,
    tok: Tok,
) -> Result<(), ParseError<usize, failure::Error>> {
    if tokens.peek() != tok {
        return Err(ParseError::InvalidToken {
            location: tokens.start_loc(),
        });
    }
    tokens.advance()?;
    Ok(())
}

fn parse_name<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<String, ParseError<usize, failure::Error>> {
    if tokens.peek() != Tok::NameValue {
        return Err(ParseError::InvalidToken {
            location: tokens.start_loc(),
        });
    }
    let name = tokens.content().to_string();
    tokens.advance()?;
    Ok(name)
}

fn parse_name_begin_ty<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<String, ParseError<usize, failure::Error>> {
    if tokens.peek() != Tok::NameBeginTyValue {
        return Err(ParseError::InvalidToken {
            location: tokens.start_loc(),
        });
    }
    let s = tokens.content();
    // The token includes a "<" at the end, so chop that off to get the name.
    let name = s[..s.len() - 1].to_string();
    tokens.advance()?;
    Ok(name)
}

fn parse_dot_name<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<String, ParseError<usize, failure::Error>> {
    if tokens.peek() != Tok::DotNameValue {
        return Err(ParseError::InvalidToken {
            location: tokens.start_loc(),
        });
    }
    let name = tokens.content().to_string();
    tokens.advance()?;
    Ok(name)
}

// AccountAddress: AccountAddress = {
//     < s: r"0[xX][0-9a-fA-F]+" > => { ... }
// };

fn parse_account_address<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<AccountAddress, ParseError<usize, failure::Error>> {
    if tokens.peek() != Tok::AccountAddressValue {
        return Err(ParseError::InvalidToken {
            location: tokens.start_loc(),
        });
    }
    let addr = AccountAddress::from_hex_literal(&tokens.content()).unwrap_or_else(|_| {
        panic!(
            "The address {:?} is of invalid length. Addresses are at most 32-bytes long",
            tokens.content()
        )
    });
    tokens.advance()?;
    Ok(addr)
}

// Var: Var = {
//     <n:Name> =>? Var::parse(n),
// };

fn parse_var<'input>(tokens: &mut Lexer<'input>) -> Result<Var, ParseError<usize, failure::Error>> {
    Var::parse(parse_name(tokens)?)
}

fn parse_var_<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<Var_, ParseError<usize, failure::Error>> {
    let start_loc = tokens.start_loc();
    let var = parse_var(tokens)?;
    let end_loc = tokens.previous_end_loc();
    Ok(spanned(start_loc, end_loc, var))
}

// Field: Field = {
//     <n:Name> =>? parse_field(n),
// };

fn parse_field_<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<Field_, ParseError<usize, failure::Error>> {
    let start_loc = tokens.start_loc();
    let f = parse_field(parse_name(tokens)?)?;
    let end_loc = tokens.previous_end_loc();
    Ok(spanned(start_loc, end_loc, f))
}

// CopyableVal: CopyableVal = {
//     AccountAddress => CopyableVal::Address(<>),
//     "true" => CopyableVal::Bool(true),
//     "false" => CopyableVal::Bool(false),
//     <i: U64> => CopyableVal::U64(i),
//     <buf: ByteArray> => CopyableVal::ByteArray(buf),
// }

fn parse_copyable_val_<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<CopyableVal_, ParseError<usize, failure::Error>> {
    let start_loc = tokens.start_loc();
    let val = match tokens.peek() {
        Tok::AccountAddressValue => {
            let addr = parse_account_address(tokens)?;
            CopyableVal::Address(addr)
        }
        Tok::True => {
            tokens.advance()?;
            CopyableVal::Bool(true)
        }
        Tok::False => {
            tokens.advance()?;
            CopyableVal::Bool(false)
        }
        Tok::U64Value => {
            let i = u64::from_str(tokens.content()).unwrap();
            tokens.advance()?;
            CopyableVal::U64(i)
        }
        Tok::ByteArrayValue => {
            let s = tokens.content();
            let buf = ByteArray::new(hex::decode(&s[2..s.len() - 1]).unwrap_or_else(|_| {
                panic!("The string {:?} is not a valid hex-encoded byte array", s)
            }));
            tokens.advance()?;
            CopyableVal::ByteArray(buf)
        }
        _ => {
            return Err(ParseError::InvalidToken {
                location: tokens.start_loc(),
            })
        }
    };
    let end_loc = tokens.previous_end_loc();
    Ok(spanned(start_loc, end_loc, val))
}

// Get the precedence of a binary operator. The minimum precedence value
// is 1, and larger values have higher precedence. For tokens that are not
// binary operators, this returns a value of zero so that they will be
// below the minimum value and will mark the end of the binary expression
// for the code in parse_rhs_of_binary_exp.
fn get_precedence(token: &Tok) -> u32 {
    match token {
        // Reserved minimum precedence value is 1 (specified in parse_exp_)
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

fn parse_exp_<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<Exp_, ParseError<usize, failure::Error>> {
    let lhs = parse_unary_exp_(tokens)?;
    parse_rhs_of_binary_exp(tokens, lhs, /* min_prec */ 1)
}

fn parse_rhs_of_binary_exp<'input>(
    tokens: &mut Lexer<'input>,
    lhs: Exp_,
    min_prec: u32,
) -> Result<Exp_, ParseError<usize, failure::Error>> {
    let mut result = lhs;
    let mut next_tok_prec = get_precedence(&tokens.peek());

    // Continue parsing binary expressions as long as they have they
    // specified minimum precedence.
    while next_tok_prec >= min_prec {
        let op_token = tokens.peek();
        tokens.advance()?;

        let mut rhs = parse_unary_exp_(tokens)?;

        // If the next token is another binary operator with a higher
        // precedence, then recursively parse that expression as the RHS.
        let this_prec = next_tok_prec;
        next_tok_prec = get_precedence(&tokens.peek());
        if this_prec < next_tok_prec {
            rhs = parse_rhs_of_binary_exp(tokens, rhs, this_prec + 1)?;
            next_tok_prec = get_precedence(&tokens.peek());
        }

        let op = match op_token {
            Tok::EqualEqual => BinOp::Eq,
            Tok::ExclaimEqual => BinOp::Neq,
            Tok::Less => BinOp::Lt,
            Tok::Greater => BinOp::Gt,
            Tok::LessEqual => BinOp::Le,
            Tok::GreaterEqual => BinOp::Ge,
            Tok::PipePipe => BinOp::Or,
            Tok::AmpAmp => BinOp::And,
            Tok::Caret => BinOp::Xor,
            Tok::Pipe => BinOp::BitOr,
            Tok::Amp => BinOp::BitAnd,
            Tok::Plus => BinOp::Add,
            Tok::Minus => BinOp::Sub,
            Tok::Star => BinOp::Mul,
            Tok::Slash => BinOp::Div,
            Tok::Percent => BinOp::Mod,
            _ => panic!("Unexpected token that is not a binary operator"),
        };
        let start_loc = result.span.start();
        let end_loc = tokens.previous_end_loc();
        let e = Exp::BinopExp(Box::new(result), op, Box::new(rhs));
        result = Spanned {
            span: Span::new(start_loc, ByteIndex(end_loc as u32)),
            value: e,
        };
    }

    Ok(result)
}

// QualifiedFunctionName : FunctionCall = {
//     <f: Builtin> => FunctionCall::Builtin(f),
//     <module_dot_name: DotName> <type_actuals: TypeActuals> =>? { ... }
// }

fn parse_qualified_function_name_<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<FunctionCall_, ParseError<usize, failure::Error>> {
    let start_loc = tokens.start_loc();
    let call = match tokens.peek() {
        Tok::Exists
        | Tok::BorrowGlobal
        | Tok::BorrowGlobalMut
        | Tok::GetTxnGasUnitPrice
        | Tok::GetTxnMaxGasUnits
        | Tok::GetTxnPublicKey
        | Tok::GetTxnSender
        | Tok::GetTxnSequenceNumber
        | Tok::MoveFrom
        | Tok::MoveToSender
        | Tok::GetGasRemaining
        | Tok::Freeze => {
            let f = parse_builtin(tokens)?;
            FunctionCall::Builtin(f)
        }
        Tok::DotNameValue => {
            let module_dot_name = parse_dot_name(tokens)?;
            let type_actuals = parse_type_actuals(tokens)?;
            let v: Vec<&str> = module_dot_name.split('.').collect();
            assert!(v.len() == 2);
            FunctionCall::ModuleFunctionCall {
                module: ModuleName::parse(v[0])?,
                name: FunctionName::parse(v[1])?,
                type_actuals,
            }
        }
        _ => {
            return Err(ParseError::InvalidToken {
                location: tokens.start_loc(),
            })
        }
    };
    let end_loc = tokens.previous_end_loc();
    Ok(spanned(start_loc, end_loc, call))
}

// UnaryExp : Exp = {
//     "!" <e: Sp<UnaryExp>> => Exp::UnaryExp(UnaryOp::Not, Box::new(e)),
//     "*" <e: Sp<UnaryExp>> => Exp::Dereference(Box::new(e)),
//     "&mut " <e: Sp<UnaryExp>> "." <f: Field> => { ... },
//     "&" <e: Sp<UnaryExp>> "." <f: Field> => { ... },
//     CallOrTerm,
// }

fn parse_borrow_field<'input>(
    tokens: &mut Lexer<'input>,
    mutable: bool,
) -> Result<Exp, ParseError<usize, failure::Error>> {
    // This could be either a field borrow (from UnaryExp) or
    // a borrow of a local variable (from Term). In the latter case,
    // only a simple name token is allowed, and it must not be
    // the start of a pack expression.
    let e = if tokens.peek() == Tok::NameValue {
        let start_loc = tokens.start_loc();
        let name = parse_name(tokens)?;
        let end_loc = tokens.previous_end_loc();
        if tokens.peek() != Tok::LBrace {
            let var = spanned(start_loc, end_loc, Var::parse(name)?);
            return Ok(Exp::BorrowLocal(mutable, var));
        }
        let type_actuals: Vec<Type> = vec![];
        spanned(start_loc, end_loc, parse_pack(tokens, &name, type_actuals)?)
    } else {
        parse_unary_exp_(tokens)?
    };
    consume_token(tokens, Tok::Period)?;
    let f = parse_field(parse_name(tokens)?)?;
    Ok(Exp::Borrow {
        is_mutable: mutable,
        exp: Box::new(e),
        field: f,
    })
}

fn parse_unary_exp<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<Exp, ParseError<usize, failure::Error>> {
    match tokens.peek() {
        Tok::Exclaim => {
            tokens.advance()?;
            let e = parse_unary_exp_(tokens)?;
            Ok(Exp::UnaryExp(UnaryOp::Not, Box::new(e)))
        }
        Tok::Star => {
            tokens.advance()?;
            let e = parse_unary_exp_(tokens)?;
            Ok(Exp::Dereference(Box::new(e)))
        }
        Tok::AmpMut => {
            tokens.advance()?;
            parse_borrow_field(tokens, true)
        }
        Tok::Amp => {
            tokens.advance()?;
            parse_borrow_field(tokens, false)
        }
        _ => parse_call_or_term(tokens),
    }
}

fn parse_unary_exp_<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<Exp_, ParseError<usize, failure::Error>> {
    let start_loc = tokens.start_loc();
    let e = parse_unary_exp(tokens)?;
    let end_loc = tokens.previous_end_loc();
    Ok(spanned(start_loc, end_loc, e))
}

// Call: Exp = {
//     <f: Sp<QualifiedFunctionName>> <exp: Sp<CallOrTerm>> => Exp::FunctionCall(f, Box::new(exp)),
// }

fn parse_call_<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<Exp_, ParseError<usize, failure::Error>> {
    let start_loc = tokens.start_loc();
    let f = parse_qualified_function_name_(tokens)?;
    let exp = parse_call_or_term_(tokens)?;
    let end_loc = tokens.previous_end_loc();
    Ok(spanned(
        start_loc,
        end_loc,
        Exp::FunctionCall(f, Box::new(exp)),
    ))
}

// CallOrTerm: Exp = {
//     <f: Sp<QualifiedFunctionName>> <exp: Sp<CallOrTerm>> => Exp::FunctionCall(f, Box::new(exp)),
//     Term,
// }

fn parse_call_or_term<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<Exp, ParseError<usize, failure::Error>> {
    match tokens.peek() {
        Tok::Exists
        | Tok::BorrowGlobal
        | Tok::BorrowGlobalMut
        | Tok::GetTxnGasUnitPrice
        | Tok::GetTxnMaxGasUnits
        | Tok::GetTxnPublicKey
        | Tok::GetTxnSender
        | Tok::GetTxnSequenceNumber
        | Tok::MoveFrom
        | Tok::MoveToSender
        | Tok::GetGasRemaining
        | Tok::Freeze
        | Tok::DotNameValue => {
            let f = parse_qualified_function_name_(tokens)?;
            let exp = parse_call_or_term_(tokens)?;
            Ok(Exp::FunctionCall(f, Box::new(exp)))
        }
        _ => parse_term(tokens),
    }
}

fn parse_call_or_term_<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<Exp_, ParseError<usize, failure::Error>> {
    let start_loc = tokens.start_loc();
    let v = parse_call_or_term(tokens)?;
    let end_loc = tokens.previous_end_loc();
    Ok(spanned(start_loc, end_loc, v))
}

// FieldExp: (Field_, Exp_) = {
//     <f: Sp<Field>> ":" <e: Sp<Exp>> => (f, e)
// }

fn parse_field_exp<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<(Field_, Exp_), ParseError<usize, failure::Error>> {
    let f = parse_field_(tokens)?;
    consume_token(tokens, Tok::Colon)?;
    let e = parse_exp_(tokens)?;
    Ok((f, e))
}

// Term: Exp = {
//     "move(" <v: Sp<Var>> ")" => Exp::Move(v),
//     "copy(" <v: Sp<Var>> ")" => Exp::Copy(v),
//     "&mut " <v: Sp<Var>> => Exp::BorrowLocal(true, v),
//     "&" <v: Sp<Var>> => Exp::BorrowLocal(false, v),
//     Sp<CopyableVal> => Exp::Value(<>),
//     <name_and_type_actuals: NameAndTypeActuals> "{" <fs:Comma<FieldExp>> "}" =>? { ... },
//     "(" <exps: Comma<Sp<Exp>>> ")" => Exp::ExprList(exps),
// }

fn parse_pack<'input>(
    tokens: &mut Lexer<'input>,
    name: &str,
    type_actuals: Vec<Type>,
) -> Result<Exp, ParseError<usize, failure::Error>> {
    consume_token(tokens, Tok::LBrace)?;
    let mut fs: Vec<(Field_, Exp_)> = vec![];
    while tokens.peek() != Tok::RBrace {
        fs.push(parse_field_exp(tokens)?);
        if tokens.peek() == Tok::RBrace {
            break;
        }
        consume_token(tokens, Tok::Comma)?;
    }
    tokens.advance()?; // consume the RBrace
    Ok(Exp::Pack(
        StructName::parse(name)?,
        type_actuals,
        fs.into_iter().collect::<Vec<_>>(),
    ))
}

fn parse_term<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<Exp, ParseError<usize, failure::Error>> {
    match tokens.peek() {
        Tok::Move => {
            tokens.advance()?;
            let v = parse_var_(tokens)?;
            consume_token(tokens, Tok::RParen)?;
            Ok(Exp::Move(v))
        }
        Tok::Copy => {
            tokens.advance()?;
            let v = parse_var_(tokens)?;
            consume_token(tokens, Tok::RParen)?;
            Ok(Exp::Copy(v))
        }
        Tok::AmpMut => {
            tokens.advance()?;
            let v = parse_var_(tokens)?;
            Ok(Exp::BorrowLocal(true, v))
        }
        Tok::Amp => {
            tokens.advance()?;
            let v = parse_var_(tokens)?;
            Ok(Exp::BorrowLocal(false, v))
        }
        Tok::AccountAddressValue | Tok::True | Tok::False | Tok::U64Value | Tok::ByteArrayValue => {
            Ok(Exp::Value(parse_copyable_val_(tokens)?))
        }
        Tok::NameValue | Tok::NameBeginTyValue => {
            let (name, type_actuals) = parse_name_and_type_actuals(tokens)?;
            parse_pack(tokens, &name, type_actuals)
        }
        Tok::LParen => {
            tokens.advance()?;
            let mut exps: Vec<Exp_> = vec![];
            while tokens.peek() != Tok::RParen {
                exps.push(parse_exp_(tokens)?);
                if tokens.peek() == Tok::RParen {
                    break;
                }
                consume_token(tokens, Tok::Comma)?;
            }
            tokens.advance()?; // consume the RParen
            Ok(Exp::ExprList(exps))
        }
        _ => Err(ParseError::InvalidToken {
            location: tokens.start_loc(),
        }),
    }
}

// StructName: StructName = {
//     <n: Name> =>? StructName::parse(n),
// }

fn parse_struct_name<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<StructName, ParseError<usize, failure::Error>> {
    StructName::parse(parse_name(tokens)?)
}

// QualifiedStructIdent : QualifiedStructIdent = {
//     <module_dot_struct: DotName> =>? { ... }
// }

fn parse_qualified_struct_ident<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<QualifiedStructIdent, ParseError<usize, failure::Error>> {
    let module_dot_struct = parse_dot_name(tokens)?;
    let v: Vec<&str> = module_dot_struct.split('.').collect();
    assert!(v.len() == 2);
    let m: ModuleName = ModuleName::parse(v[0])?;
    let n: StructName = StructName::parse(v[1])?;
    Ok(QualifiedStructIdent::new(m, n))
}

// ModuleName: ModuleName = {
//     <n: Name> =>? ModuleName::parse(n),
// }

fn parse_module_name<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<ModuleName, ParseError<usize, failure::Error>> {
    ModuleName::parse(parse_name(tokens)?)
}

// Builtin: Builtin = {
//     "exists<" <name_and_type_actuals: NameAndTypeActuals> ">" =>? { ... },
//     "borrow_global<" <name_and_type_actuals: NameAndTypeActuals> ">" =>? { ... },
//     "borrow_global_mut<" <name_and_type_actuals: NameAndTypeActuals> ">" =>? { ... },
//     "get_txn_gas_unit_price" => Builtin::GetTxnGasUnitPrice,
//     "get_txn_max_gas_units" => Builtin::GetTxnMaxGasUnits,
//     "get_txn_public_key" => Builtin::GetTxnPublicKey,
//     "get_txn_sender" => Builtin::GetTxnSender,
//     "get_txn_sequence_number" => Builtin::GetTxnSequenceNumber,
//     "move_from<" <name_and_type_actuals: NameAndTypeActuals> ">" =>? { ... },
//     "move_to_sender<" <name_and_type_actuals: NameAndTypeActuals> ">" =>? { ...},
//     "get_gas_remaining" => Builtin::GetGasRemaining,
//     "freeze" => Builtin::Freeze,
// }

fn parse_builtin<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<Builtin, ParseError<usize, failure::Error>> {
    match tokens.peek() {
        Tok::Exists => {
            tokens.advance()?;
            let (name, type_actuals) = parse_name_and_type_actuals(tokens)?;
            consume_token(tokens, Tok::Greater)?;
            Ok(Builtin::Exists(StructName::parse(name)?, type_actuals))
        }
        Tok::BorrowGlobal => {
            tokens.advance()?;
            let (name, type_actuals) = parse_name_and_type_actuals(tokens)?;
            consume_token(tokens, Tok::Greater)?;
            Ok(Builtin::BorrowGlobal(
                false,
                StructName::parse(name)?,
                type_actuals,
            ))
        }
        Tok::BorrowGlobalMut => {
            tokens.advance()?;
            let (name, type_actuals) = parse_name_and_type_actuals(tokens)?;
            consume_token(tokens, Tok::Greater)?;
            Ok(Builtin::BorrowGlobal(
                true,
                StructName::parse(name)?,
                type_actuals,
            ))
        }
        Tok::GetTxnGasUnitPrice => {
            tokens.advance()?;
            Ok(Builtin::GetTxnGasUnitPrice)
        }
        Tok::GetTxnMaxGasUnits => {
            tokens.advance()?;
            Ok(Builtin::GetTxnMaxGasUnits)
        }
        Tok::GetTxnPublicKey => {
            tokens.advance()?;
            Ok(Builtin::GetTxnPublicKey)
        }
        Tok::GetTxnSender => {
            tokens.advance()?;
            Ok(Builtin::GetTxnSender)
        }
        Tok::GetTxnSequenceNumber => {
            tokens.advance()?;
            Ok(Builtin::GetTxnSequenceNumber)
        }
        Tok::MoveFrom => {
            tokens.advance()?;
            let (name, type_actuals) = parse_name_and_type_actuals(tokens)?;
            consume_token(tokens, Tok::Greater)?;
            Ok(Builtin::MoveFrom(StructName::parse(name)?, type_actuals))
        }
        Tok::MoveToSender => {
            tokens.advance()?;
            let (name, type_actuals) = parse_name_and_type_actuals(tokens)?;
            consume_token(tokens, Tok::Greater)?;
            Ok(Builtin::MoveToSender(
                StructName::parse(name)?,
                type_actuals,
            ))
        }
        Tok::GetGasRemaining => {
            tokens.advance()?;
            Ok(Builtin::GetGasRemaining)
        }
        Tok::Freeze => {
            tokens.advance()?;
            Ok(Builtin::Freeze)
        }
        _ => Err(ParseError::InvalidToken {
            location: tokens.start_loc(),
        }),
    }
}

// LValue: LValue = {
//     <l:Sp<Var>> => LValue::Var(l),
//     "*" <e: Sp<Exp>> => LValue::Mutate(e),
//     "_" => LValue::Pop,
// }

fn parse_lvalue<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<LValue, ParseError<usize, failure::Error>> {
    match tokens.peek() {
        Tok::NameValue => {
            let l = parse_var_(tokens)?;
            Ok(LValue::Var(l))
        }
        Tok::Star => {
            tokens.advance()?;
            let e = parse_exp_(tokens)?;
            Ok(LValue::Mutate(e))
        }
        Tok::Underscore => {
            tokens.advance()?;
            Ok(LValue::Pop)
        }
        _ => Err(ParseError::InvalidToken {
            location: tokens.start_loc(),
        }),
    }
}

fn parse_lvalue_<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<LValue_, ParseError<usize, failure::Error>> {
    let start_loc = tokens.start_loc();
    let lv = parse_lvalue(tokens)?;
    let end_loc = tokens.previous_end_loc();
    Ok(spanned(start_loc, end_loc, lv))
}

// LValues: Vec<LValue_> = {
//     <l:Sp<LValue>> <v: ("," <Sp<LValue>>)*> => { ... }
// }

fn parse_lvalues<'input>(
    tokens: &mut Lexer<'input>,
    prefix: Option<LValue_>,
) -> Result<Vec<LValue_>, ParseError<usize, failure::Error>> {
    let l = if let Some(lv) = prefix {
        lv
    } else {
        parse_lvalue_(tokens)?
    };
    let mut lvalues = vec![l];
    while tokens.peek() == Tok::Comma {
        tokens.advance()?;
        lvalues.push(parse_lvalue_(tokens)?);
    }
    Ok(lvalues)
}

// FieldBindings: (Field_, Var_) = {
//     <f: Sp<Field>> ":" <v: Sp<Var>> => (f, v),
//     <f: Sp<Field>> => { ... }
// }

fn parse_field_bindings<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<(Field_, Var_), ParseError<usize, failure::Error>> {
    let f = parse_field_(tokens)?;
    if tokens.peek() == Tok::Colon {
        tokens.advance()?; // consume the colon
        let v = parse_var_(tokens)?;
        Ok((f, v))
    } else {
        Ok((
            f.clone(),
            Spanned {
                span: f.span,
                value: Var::new(f.value.name().into()),
            },
        ))
    }
}

// pub Cmd : Cmd = {
//     <lvalues: LValues> "=" <e: Sp<Exp>> => Cmd::Assign(lvalues, e),
//     <name_and_type_actuals: NameAndTypeActuals> "{" <bindings: Comma<FieldBindings>> "}" "=" <e: Sp<Exp>> =>? { ... },
//     "abort" <err: Sp<Exp>?> => { ... },
//     "return" <v: Comma<Sp<Exp>>> => Cmd::Return(Box::new(Spanned::no_loc(Exp::ExprList(v)))),
//     "continue" => Cmd::Continue,
//     "break" => Cmd::Break,
//     <Sp<Call>> => Cmd::Exp(Box::new(<>)),
//     "(" <Comma<Sp<Exp>>> ")" => Cmd::Exp(Box::new(Spanned::no_loc(Exp::ExprList(<>)))),
// }

fn parse_assign<'input>(
    tokens: &mut Lexer<'input>,
    prefix: Option<LValue_>,
) -> Result<Cmd, ParseError<usize, failure::Error>> {
    let lvalues = parse_lvalues(tokens, prefix)?;
    consume_token(tokens, Tok::Equal)?;
    let e = parse_exp_(tokens)?;
    Ok(Cmd::Assign(lvalues, e))
}

fn parse_unpack<'input>(
    tokens: &mut Lexer<'input>,
    name: &str,
    type_actuals: Vec<Type>,
) -> Result<Cmd, ParseError<usize, failure::Error>> {
    consume_token(tokens, Tok::LBrace)?;
    let mut bindings: Vec<(Field_, Var_)> = vec![];
    while tokens.peek() != Tok::RBrace {
        bindings.push(parse_field_bindings(tokens)?);
        if tokens.peek() == Tok::RBrace {
            break;
        }
        consume_token(tokens, Tok::Comma)?;
    }
    tokens.advance()?; // consume the RBrace
    consume_token(tokens, Tok::Equal)?;
    let e = parse_exp_(tokens)?;
    Ok(Cmd::Unpack(
        StructName::parse(name)?,
        type_actuals,
        bindings.into_iter().collect(),
        Box::new(e),
    ))
}

fn parse_cmd<'input>(tokens: &mut Lexer<'input>) -> Result<Cmd, ParseError<usize, failure::Error>> {
    match tokens.peek() {
        Tok::NameValue => {
            // This could be either an LValue for an assignment or
            // NameAndTypeActuals (with no type_actuals) for an unpack.
            let start_loc = tokens.start_loc();
            let name = parse_name(tokens)?;
            if tokens.peek() == Tok::LBrace {
                parse_unpack(tokens, &name, vec![])
            } else {
                // Construct the first LValue_ for the LValues vector.
                let var = Var::parse(name)?;
                let end_loc = tokens.previous_end_loc();
                let v = spanned(start_loc, end_loc, var);
                let lv = spanned(start_loc, end_loc, LValue::Var(v));
                parse_assign(tokens, Some(lv))
            }
        }
        Tok::Star | Tok::Underscore => parse_assign(tokens, None),
        Tok::NameBeginTyValue => {
            let (name, tys) = parse_name_and_type_actuals(tokens)?;
            parse_unpack(tokens, &name, tys)
        }
        Tok::Abort => {
            tokens.advance()?;
            let val = if tokens.peek() == Tok::Semicolon {
                None
            } else {
                Some(Box::new(parse_exp_(tokens)?))
            };
            Ok(Cmd::Abort(val))
        }
        Tok::Return => {
            tokens.advance()?;
            let mut v: Vec<Exp_> = vec![];
            while tokens.peek() != Tok::Semicolon {
                v.push(parse_exp_(tokens)?);
                if tokens.peek() == Tok::Semicolon {
                    break;
                }
                consume_token(tokens, Tok::Comma)?;
            }
            Ok(Cmd::Return(Box::new(Spanned::no_loc(Exp::ExprList(v)))))
        }
        Tok::Continue => {
            tokens.advance()?;
            Ok(Cmd::Continue)
        }
        Tok::Break => {
            tokens.advance()?;
            Ok(Cmd::Break)
        }
        Tok::Exists
        | Tok::BorrowGlobal
        | Tok::BorrowGlobalMut
        | Tok::GetTxnGasUnitPrice
        | Tok::GetTxnMaxGasUnits
        | Tok::GetTxnPublicKey
        | Tok::GetTxnSender
        | Tok::GetTxnSequenceNumber
        | Tok::MoveFrom
        | Tok::MoveToSender
        | Tok::GetGasRemaining
        | Tok::Freeze
        | Tok::DotNameValue => Ok(Cmd::Exp(Box::new(parse_call_(tokens)?))),
        Tok::LParen => {
            tokens.advance()?;
            let mut v: Vec<Exp_> = vec![];
            while tokens.peek() != Tok::RParen {
                v.push(parse_exp_(tokens)?);
                if tokens.peek() == Tok::RParen {
                    break;
                }
                consume_token(tokens, Tok::Comma)?;
            }
            tokens.advance()?; // consume the RParen
            Ok(Cmd::Exp(Box::new(Spanned::no_loc(Exp::ExprList(v)))))
        }
        _ => Err(ParseError::InvalidToken {
            location: tokens.start_loc(),
        }),
    }
}

// Statement : Statement = {
//     <cmd: Cmd_> ";" => Statement::CommandStatement(cmd),
//     "assert(" <e: Sp<Exp>> "," <err: Sp<Exp>> ")" => { ... },
//     <IfStatement>,
//     <WhileStatement>,
//     <LoopStatement>,
//     ";" => Statement::EmptyStatement,
// }

fn parse_statement<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<Statement, ParseError<usize, failure::Error>> {
    match tokens.peek() {
        Tok::Assert => {
            tokens.advance()?;
            let e = parse_exp_(tokens)?;
            consume_token(tokens, Tok::Comma)?;
            let err = parse_exp_(tokens)?;
            consume_token(tokens, Tok::RParen)?;
            let cond = {
                let span = e.span;
                Spanned {
                    span,
                    value: Exp::UnaryExp(UnaryOp::Not, Box::new(e)),
                }
            };
            let span = err.span;
            let stmt = {
                Statement::CommandStatement(Spanned {
                    span,
                    value: Cmd::Abort(Some(Box::new(err))),
                })
            };
            Ok(Statement::IfElseStatement(IfElse::if_block(
                cond,
                Spanned {
                    span,
                    value: Block::new(vec![stmt]),
                },
            )))
        }
        Tok::If => parse_if_statement(tokens),
        Tok::While => parse_while_statement(tokens),
        Tok::Loop => parse_loop_statement(tokens),
        Tok::Semicolon => {
            tokens.advance()?;
            Ok(Statement::EmptyStatement)
        }
        _ => {
            // Anything else should be parsed as a Cmd...
            let start_loc = tokens.start_loc();
            let c = parse_cmd(tokens)?;
            let end_loc = tokens.previous_end_loc();
            let cmd = spanned(start_loc, end_loc, c);
            consume_token(tokens, Tok::Semicolon)?;
            Ok(Statement::CommandStatement(cmd))
        }
    }
}

// IfStatement : Statement = {
//     "if" "(" <cond: Sp<Exp>> ")" <block: Sp<Block>> => { ... }
//     "if" "(" <cond: Sp<Exp>> ")" <if_block: Sp<Block>> "else" <else_block: Sp<Block>> => { ... }
// }

fn parse_if_statement<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<Statement, ParseError<usize, failure::Error>> {
    consume_token(tokens, Tok::If)?;
    consume_token(tokens, Tok::LParen)?;
    let cond = parse_exp_(tokens)?;
    consume_token(tokens, Tok::RParen)?;
    let if_block = parse_block_(tokens)?;
    if tokens.peek() == Tok::Else {
        tokens.advance()?;
        let else_block = parse_block_(tokens)?;
        Ok(Statement::IfElseStatement(IfElse::if_else(
            cond, if_block, else_block,
        )))
    } else {
        Ok(Statement::IfElseStatement(IfElse::if_block(cond, if_block)))
    }
}

// WhileStatement : Statement = {
//     "while" "(" <cond: Sp<Exp>> ")" <block: Sp<Block>> => { ... }
// }

fn parse_while_statement<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<Statement, ParseError<usize, failure::Error>> {
    consume_token(tokens, Tok::While)?;
    consume_token(tokens, Tok::LParen)?;
    let cond = parse_exp_(tokens)?;
    consume_token(tokens, Tok::RParen)?;
    let block = parse_block_(tokens)?;
    Ok(Statement::WhileStatement(While { cond, block }))
}

// LoopStatement : Statement = {
//     "loop" <block: Sp<Block>> => { ... }
// }

fn parse_loop_statement<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<Statement, ParseError<usize, failure::Error>> {
    consume_token(tokens, Tok::Loop)?;
    let block = parse_block_(tokens)?;
    Ok(Statement::LoopStatement(Loop { block }))
}

// Statements : Vec<Statement> = {
//     <Statement*>
// }

fn parse_statements<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<Vec<Statement>, ParseError<usize, failure::Error>> {
    let mut stmts: Vec<Statement> = vec![];
    // The Statements non-terminal in the grammar is always followed by a
    // closing brace, so continue parsing until we find one of those.
    while tokens.peek() != Tok::RBrace {
        stmts.push(parse_statement(tokens)?);
    }
    Ok(stmts)
}

// Block : Block = {
//     "{" <stmts: Statements> "}" => Block::new(stmts)
// }

fn parse_block_<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<Block_, ParseError<usize, failure::Error>> {
    let start_loc = tokens.start_loc();
    consume_token(tokens, Tok::LBrace)?;
    let stmts = parse_statements(tokens)?;
    consume_token(tokens, Tok::RBrace)?;
    let end_loc = tokens.previous_end_loc();
    Ok(spanned(start_loc, end_loc, Block::new(stmts)))
}

// Declaration: (Var_, Type) = {
//   "let" <v: Sp<Var>> ":" <t: Type> ";" => (v, t),
// }

fn parse_declaration<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<(Var_, Type), ParseError<usize, failure::Error>> {
    consume_token(tokens, Tok::Let)?;
    let v = parse_var_(tokens)?;
    consume_token(tokens, Tok::Colon)?;
    let t = parse_type(tokens)?;
    consume_token(tokens, Tok::Semicolon)?;
    Ok((v, t))
}

// Declarations: Vec<(Var_, Type)> = {
//     <Declaration*>
// }

fn parse_declarations<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<Vec<(Var_, Type)>, ParseError<usize, failure::Error>> {
    let mut decls: Vec<(Var_, Type)> = vec![];
    // Declarations always begin with the "let" token so continue parsing
    // them until we hit something else.
    while tokens.peek() == Tok::Let {
        decls.push(parse_declaration(tokens)?);
    }
    Ok(decls)
}

// FunctionBlock: (Vec<(Var_, Type)>, Block) = {
//     "{" <locals: Declarations> <stmts: Statements> "}" => (locals, Block::new(stmts))
// }

fn parse_function_block<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<(Vec<(Var_, Type)>, Block), ParseError<usize, failure::Error>> {
    consume_token(tokens, Tok::LBrace)?;
    let locals = parse_declarations(tokens)?;
    let stmts = parse_statements(tokens)?;
    consume_token(tokens, Tok::RBrace)?;
    Ok((locals, Block::new(stmts)))
}

// Kind: Kind = {
//     "resource" => Kind::Resource,
//     "unrestricted" => Kind::Unrestricted,
// }

fn parse_kind<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<Kind, ParseError<usize, failure::Error>> {
    let k = match tokens.peek() {
        Tok::Resource => Kind::Resource,
        Tok::Unrestricted => Kind::Unrestricted,
        _ => {
            return Err(ParseError::InvalidToken {
                location: tokens.start_loc(),
            })
        }
    };
    tokens.advance()?;
    Ok(k)
}

// Type: Type = {
//     "address" => Type::Address,
//     "u64" => Type::U64,
//     "bool" => Type::Bool,
//     "bytearray" => Type::ByteArray,
//     <s: QualifiedStructIdent> <tys: TypeActuals> => Type::Struct(s, tys),
//     "&" <t: Type> => Type::Reference(false, Box::new(t)),
//     "&mut " <t: Type> => Type::Reference(true, Box::new(t)),
//     <n: Name> =>? Ok(Type::TypeParameter(TypeVar::parse(n)?)),
// }

fn parse_type<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<Type, ParseError<usize, failure::Error>> {
    let t = match tokens.peek() {
        Tok::Address => {
            tokens.advance()?;
            Type::Address
        }
        Tok::U64 => {
            tokens.advance()?;
            Type::U64
        }
        Tok::Bool => {
            tokens.advance()?;
            Type::Bool
        }
        Tok::Bytearray => {
            tokens.advance()?;
            Type::ByteArray
        }
        Tok::DotNameValue => {
            let s = parse_qualified_struct_ident(tokens)?;
            let tys = parse_type_actuals(tokens)?;
            Type::Struct(s, tys)
        }
        Tok::Amp => {
            tokens.advance()?;
            Type::Reference(false, Box::new(parse_type(tokens)?))
        }
        Tok::AmpMut => {
            tokens.advance()?;
            Type::Reference(true, Box::new(parse_type(tokens)?))
        }
        Tok::NameValue => Type::TypeParameter(TypeVar::parse(parse_name(tokens)?)?),
        _ => {
            return Err(ParseError::InvalidToken {
                location: tokens.start_loc(),
            })
        }
    };
    Ok(t)
}

// TypeVar: TypeVar = {
//     <n: Name> =>? TypeVar::parse(n),
// }
// TypeVar_ = Sp<TypeVar>;

fn parse_type_var_<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<TypeVar_, ParseError<usize, failure::Error>> {
    let start_loc = tokens.start_loc();
    let type_var = TypeVar::parse(parse_name(tokens)?)?;
    let end_loc = tokens.previous_end_loc();
    Ok(spanned(start_loc, end_loc, type_var))
}

// TypeFormal: (TypeVar_, Kind) = {
//     <type_var: Sp<TypeVar>> <k: (":" <Kind>)?> =>? {
// }

fn parse_type_formal<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<(TypeVar_, Kind), ParseError<usize, failure::Error>> {
    let type_var = parse_type_var_(tokens)?;
    if tokens.peek() == Tok::Colon {
        tokens.advance()?; // consume the ":"
        let k = parse_kind(tokens)?;
        Ok((type_var, k))
    } else {
        Ok((type_var, Kind::All))
    }
}

// TypeActuals: Vec<Type> = {
//     <tys: ("<" <Comma<Type>> ">")?> => { ... }
// }

fn parse_type_actuals<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<Vec<Type>, ParseError<usize, failure::Error>> {
    let mut tys: Vec<Type> = vec![];
    if tokens.peek() == Tok::Less {
        tokens.advance()?; // consume the "<"
        while tokens.peek() != Tok::Greater {
            tys.push(parse_type(tokens)?);
            if tokens.peek() == Tok::Greater {
                break;
            }
            consume_token(tokens, Tok::Comma)?;
        }
        tokens.advance()?; // consume the ">"
    }
    Ok(tys)
}

// NameAndTypeFormals: (String, Vec<(TypeVar_, Kind)>) = {
//     <n: NameBeginTy> <k: Comma<TypeFormal>> ">" => (n, k),
//     <n: Name> => (n, vec![]),
// }

fn parse_name_and_type_formals<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<(String, Vec<(TypeVar_, Kind)>), ParseError<usize, failure::Error>> {
    let mut has_types = false;
    let n = if tokens.peek() == Tok::NameBeginTyValue {
        has_types = true;
        parse_name_begin_ty(tokens)?
    } else {
        parse_name(tokens)?
    };
    let mut k: Vec<(TypeVar_, Kind)> = vec![];
    if has_types {
        while tokens.peek() != Tok::Greater {
            k.push(parse_type_formal(tokens)?);
            if tokens.peek() == Tok::Greater {
                break;
            }
            consume_token(tokens, Tok::Comma)?;
        }
        tokens.advance()?; // consume the ">"
    }
    Ok((n, k))
}

// NameAndTypeActuals: (String, Vec<Type>) = {
//     <n: NameBeginTy> <tys: Comma<Type>> ">" => (n, tys),
//     <n: Name> => (n, vec![]),
// }

fn parse_name_and_type_actuals<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<(String, Vec<Type>), ParseError<usize, failure::Error>> {
    let mut has_types = false;
    let n = if tokens.peek() == Tok::NameBeginTyValue {
        has_types = true;
        parse_name_begin_ty(tokens)?
    } else {
        parse_name(tokens)?
    };
    let mut tys: Vec<Type> = vec![];
    if has_types {
        while tokens.peek() != Tok::Greater {
            tys.push(parse_type(tokens)?);
            if tokens.peek() == Tok::Greater {
                break;
            }
            consume_token(tokens, Tok::Comma)?;
        }
        tokens.advance()?; // consume the ">"
    }
    Ok((n, tys))
}

// ArgDecl : (Var_, Type) = {
//     <v: Sp<Var>> ":" <t: Type> ","? => (v, t)
// }

fn parse_arg_decl<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<(Var_, Type), ParseError<usize, failure::Error>> {
    let v = parse_var_(tokens)?;
    consume_token(tokens, Tok::Colon)?;
    let t = parse_type(tokens)?;
    if tokens.peek() == Tok::Comma {
        tokens.advance()?;
    }
    Ok((v, t))
}

// ReturnType: Vec<Type> = {
//     ":" <t: Type> <v: ("*" <Type>)*> => { ... }
// }

fn parse_return_type<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<Vec<Type>, ParseError<usize, failure::Error>> {
    consume_token(tokens, Tok::Colon)?;
    let t = parse_type(tokens)?;
    let mut v = vec![t];
    while tokens.peek() == Tok::Star {
        tokens.advance()?;
        v.push(parse_type(tokens)?);
    }
    Ok(v)
}

// AcquireList: Vec<StructName> = {
//     "acquires" <s: StructName> <al: ("," <StructName>)*> => { ... }
// }

fn parse_acquire_list<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<Vec<StructName>, ParseError<usize, failure::Error>> {
    consume_token(tokens, Tok::Acquires)?;
    let s = parse_struct_name(tokens)?;
    let mut al = vec![s];
    while tokens.peek() == Tok::Comma {
        tokens.advance()?;
        al.push(parse_struct_name(tokens)?);
    }
    Ok(al)
}

// FunctionDecl : (FunctionName, Function_) = {
//   <f: Sp<MoveFunctionDecl>> => (f.value.0, Spanned { span: f.span, value: f.value.1 }),
//   <f: Sp<NativeFunctionDecl>> => (f.value.0, Spanned { span: f.span, value: f.value.1 }),
// }

// MoveFunctionDecl : (FunctionName, Function) = {
//     <p: Public?> <name_and_type_formals: NameAndTypeFormals> "(" <args:
//     (ArgDecl)*> ")" <ret: ReturnType?>
//     <acquires: AcquireList?>
//     <locals_body: FunctionBlock> =>? { ... }
// }

// NativeFunctionDecl: (FunctionName, Function) = {
//     <nat: NativeTag> <p: Public?> <name_and_type_formals: NameAndTypeFormals>
//     "(" <args: (ArgDecl)*> ")" <ret: ReturnType?>
//         <acquires: AcquireList?>
//         ";" =>? { ... }
// }

fn parse_function_decl<'input>(
    tokens: &mut Lexer<'input>,
    is_native: bool,
    start_loc: usize,
) -> Result<(FunctionName, Function_), ParseError<usize, failure::Error>> {
    let is_public = if tokens.peek() == Tok::Public {
        tokens.advance()?;
        true
    } else {
        false
    };

    let (name, type_formals) = parse_name_and_type_formals(tokens)?;
    consume_token(tokens, Tok::LParen)?;
    let mut args: Vec<(Var_, Type)> = vec![];
    while tokens.peek() != Tok::RParen {
        args.push(parse_arg_decl(tokens)?);
    }
    tokens.advance()?; // consume the RParen

    let ret = if tokens.peek() == Tok::Colon {
        Some(parse_return_type(tokens)?)
    } else {
        None
    };

    let acquires = if tokens.peek() == Tok::Acquires {
        Some(parse_acquire_list(tokens)?)
    } else {
        None
    };

    let func_name = FunctionName::parse(name)?;
    let func = Function::new(
        if is_public {
            FunctionVisibility::Public
        } else {
            FunctionVisibility::Internal
        },
        args,
        ret.unwrap_or_else(|| vec![]),
        type_formals,
        acquires.unwrap_or_else(Vec::new),
        if is_native {
            consume_token(tokens, Tok::Semicolon)?;
            FunctionBody::Native
        } else {
            let (locals, body) = parse_function_block(tokens)?;
            FunctionBody::Move { locals, code: body }
        },
    );

    let end_loc = tokens.previous_end_loc();
    Ok((func_name, spanned(start_loc, end_loc, func)))
}

// FieldDecl : (Field_, Type) = {
//     <f: Sp<Field>> ":" <t: Type> ","? => (f, t)
// }

fn parse_field_decl<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<(Field_, Type), ParseError<usize, failure::Error>> {
    let f = parse_field_(tokens)?;
    consume_token(tokens, Tok::Colon)?;
    let t = parse_type(tokens)?;
    if tokens.peek() == Tok::Comma {
        tokens.advance()?;
    }
    Ok((f, t))
}

// Modules: Vec<ModuleDefinition> = {
//     "modules:" <c: Module*> "script:" => c,
// }

fn parse_modules<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<Vec<ModuleDefinition>, ParseError<usize, failure::Error>> {
    consume_token(tokens, Tok::Modules)?;
    let mut c: Vec<ModuleDefinition> = vec![];
    while tokens.peek() == Tok::Module {
        c.push(parse_module(tokens)?);
    }
    consume_token(tokens, Tok::Script)?;
    Ok(c)
}

// pub Program : Program = {
//     <m: Modules?> <s: Script> => { ... },
//     <m: Module> => { ... }
// }

fn parse_program<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<Program, ParseError<usize, failure::Error>> {
    if tokens.peek() == Tok::Module {
        let m = parse_module(tokens)?;
        let ret = Spanned {
            span: Span::default(),
            value: Cmd::Return(Box::new(Spanned::no_loc(Exp::ExprList(vec![])))),
        };
        let return_stmt = Statement::CommandStatement(ret);
        let body = FunctionBody::Move {
            locals: vec![],
            code: Block::new(vec![return_stmt]),
        };
        let main = Function::new(
            FunctionVisibility::Public,
            vec![],
            vec![],
            vec![],
            vec![],
            body,
        );
        Ok(Program::new(
            vec![m],
            Script::new(vec![], Spanned::no_loc(main)),
        ))
    } else {
        let modules = if tokens.peek() == Tok::Modules {
            parse_modules(tokens)?
        } else {
            vec![]
        };
        let s = parse_script(tokens)?;
        Ok(Program::new(modules, s))
    }
}

// pub Script : Script = {
//     <imports: (ImportDecl)*>
//     "main" "(" <args: (ArgDecl)*> ")" <locals_body: FunctionBlock> => { ... }
// }

fn parse_script<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<Script, ParseError<usize, failure::Error>> {
    let start_loc = tokens.start_loc();
    let mut imports: Vec<ImportDefinition> = vec![];
    while tokens.peek() == Tok::Import {
        imports.push(parse_import_decl(tokens)?);
    }
    consume_token(tokens, Tok::Main)?;
    consume_token(tokens, Tok::LParen)?;
    let mut args: Vec<(Var_, Type)> = vec![];
    while tokens.peek() != Tok::RParen {
        args.push(parse_arg_decl(tokens)?);
    }
    tokens.advance()?; // consume the RParen
    let (locals, body) = parse_function_block(tokens)?;
    let end_loc = tokens.previous_end_loc();
    let main = Function::new(
        FunctionVisibility::Public,
        args,
        vec![],
        vec![],
        vec![],
        FunctionBody::Move { locals, code: body },
    );
    let main = spanned(start_loc, end_loc, main);
    Ok(Script::new(imports, main))
}

// StructKind: bool = {
//     "struct" => false,
//     "resource" => true
// }
// StructDecl: StructDefinition_ = {
//     <is_nominal_resource: StructKind> <name_and_type_formals:
//     NameAndTypeFormals> "{" <data: (FieldDecl)*> "}" =>? { ... }
//     <native: NativeTag> <is_nominal_resource: StructKind>
//     <name_and_type_formals: NameAndTypeFormals> ";" =>? { ... }
// }

fn parse_struct_decl<'input>(
    tokens: &mut Lexer<'input>,
    is_native: bool,
    start_loc: usize,
) -> Result<StructDefinition_, ParseError<usize, failure::Error>> {
    let is_nominal_resource = match tokens.peek() {
        Tok::Struct => false,
        Tok::Resource => true,
        _ => {
            return Err(ParseError::InvalidToken {
                location: tokens.start_loc(),
            })
        }
    };
    tokens.advance()?;

    let (name, type_formals) = parse_name_and_type_formals(tokens)?;

    if is_native {
        consume_token(tokens, Tok::Semicolon)?;
        let end_loc = tokens.previous_end_loc();
        return Ok(spanned(
            start_loc,
            end_loc,
            StructDefinition::native(is_nominal_resource, name, type_formals)?,
        ));
    }

    consume_token(tokens, Tok::LBrace)?;
    let mut fields = Fields::new();
    while tokens.peek() != Tok::RBrace {
        fields.push(parse_field_decl(tokens)?);
    }
    tokens.advance()?; // consume the RBrace
    let end_loc = tokens.previous_end_loc();
    Ok(spanned(
        start_loc,
        end_loc,
        StructDefinition::move_declared(is_nominal_resource, name, type_formals, fields)?,
    ))
}

// QualifiedModuleIdent: QualifiedModuleIdent = {
//     <a: AccountAddress> "." <m: ModuleName> => QualifiedModuleIdent::new(m, a),
// }

fn parse_qualified_module_ident<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<QualifiedModuleIdent, ParseError<usize, failure::Error>> {
    let a = parse_account_address(tokens)?;
    consume_token(tokens, Tok::Period)?;
    let m = parse_module_name(tokens)?;
    Ok(QualifiedModuleIdent::new(m, a))
}

// ModuleIdent: ModuleIdent = {
//     <q: QualifiedModuleIdent> => ModuleIdent::Qualified(q),
//     <transaction_dot_module: DotName> =>? { ... }
// }

fn parse_module_ident<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<ModuleIdent, ParseError<usize, failure::Error>> {
    if tokens.peek() == Tok::AccountAddressValue {
        return Ok(ModuleIdent::Qualified(parse_qualified_module_ident(
            tokens,
        )?));
    }
    let transaction_dot_module = parse_dot_name(tokens)?;
    let v: Vec<&str> = transaction_dot_module.split('.').collect();
    assert!(v.len() == 2);
    let ident: String = v[0].to_string();
    if ident != "Transaction" {
        panic!("Ident = {} which is not Transaction", ident);
    }
    let m: ModuleName = ModuleName::parse(v[1])?;
    Ok(ModuleIdent::Transaction(m))
}

// ImportAlias: ModuleName = {
//     "as" <alias: ModuleName> => { ... }
// }

fn parse_import_alias<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<ModuleName, ParseError<usize, failure::Error>> {
    consume_token(tokens, Tok::As)?;
    let alias = parse_module_name(tokens)?;
    if alias.as_inner() == ModuleName::self_name() {
        panic!(
            "Invalid use of reserved module alias '{}'",
            ModuleName::self_name()
        );
    }
    Ok(alias)
}

// ImportDecl: ImportDefinition = {
//     "import" <ident: ModuleIdent> <alias: ImportAlias?> ";" => { ... }
// }

fn parse_import_decl<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<ImportDefinition, ParseError<usize, failure::Error>> {
    consume_token(tokens, Tok::Import)?;
    let ident = parse_module_ident(tokens)?;
    let alias = if tokens.peek() == Tok::As {
        Some(parse_import_alias(tokens)?)
    } else {
        None
    };
    consume_token(tokens, Tok::Semicolon)?;
    Ok(ImportDefinition::new(ident, alias))
}

// pub Module : ModuleDefinition = {
//     "module" <n: Name> "{"
//         <imports: (ImportDecl)*>
//         <structs: (StructDecl)*>
//         <functions: (FunctionDecl)*>
//     "}" =>? ModuleDefinition::new(n, imports, structs, functions),
// }

fn parse_module<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<ModuleDefinition, ParseError<usize, failure::Error>> {
    consume_token(tokens, Tok::Module)?;
    let name = parse_name(tokens)?;
    consume_token(tokens, Tok::LBrace)?;

    let mut imports: Vec<ImportDefinition> = vec![];
    while tokens.peek() == Tok::Import {
        imports.push(parse_import_decl(tokens)?);
    }

    // The "native" keyword can apply to either structs or functions,
    // so the parser needs to move past that token before it can determine
    // which kind of declaration it is handling.
    let mut start_loc = tokens.start_loc();
    let mut is_native = if tokens.peek() == Tok::Native {
        tokens.advance()?;
        true
    } else {
        false
    };

    let mut structs: Vec<StructDefinition_> = vec![];
    while tokens.peek() == Tok::Struct || tokens.peek() == Tok::Resource {
        structs.push(parse_struct_decl(tokens, is_native, start_loc)?);

        start_loc = tokens.start_loc();
        is_native = if tokens.peek() == Tok::Native {
            tokens.advance()?;
            true
        } else {
            false
        };
    }

    let mut functions: Vec<(FunctionName, Function_)> = vec![];
    while tokens.peek() != Tok::RBrace {
        functions.push(parse_function_decl(tokens, is_native, start_loc)?);

        start_loc = tokens.start_loc();
        is_native = if tokens.peek() == Tok::Native {
            tokens.advance()?;
            true
        } else {
            false
        };
    }
    // Make sure there was no "native" keyword before the RBrace.
    if is_native {
        return Err(ParseError::InvalidToken {
            location: tokens.start_loc(),
        });
    }
    tokens.advance()?; // consume the RBrace

    ModuleDefinition::new(name, imports, structs, functions)
}

// pub ScriptOrModule: ScriptOrModule = {
//     <s: Script> => ScriptOrModule::Script(s),
//     <m: Module> => ScriptOrModule::Module(m),
// }

fn parse_script_or_module<'input>(
    tokens: &mut Lexer<'input>,
) -> Result<ScriptOrModule, ParseError<usize, failure::Error>> {
    if tokens.peek() == Tok::Module {
        Ok(ScriptOrModule::Module(parse_module(tokens)?))
    } else {
        Ok(ScriptOrModule::Script(parse_script(tokens)?))
    }
}

pub fn parse_cmd_string<'input>(
    input: &'input str,
) -> Result<Cmd, ParseError<usize, failure::Error>> {
    let mut tokens = Lexer::new(input);
    tokens.advance()?;
    parse_cmd(&mut tokens)
}

pub fn parse_module_string<'input>(
    input: &'input str,
) -> Result<ModuleDefinition, ParseError<usize, failure::Error>> {
    let mut tokens = Lexer::new(input);
    tokens.advance()?;
    parse_module(&mut tokens)
}

pub fn parse_program_string<'input>(
    input: &'input str,
) -> Result<Program, ParseError<usize, failure::Error>> {
    let mut tokens = Lexer::new(input);
    tokens.advance()?;
    parse_program(&mut tokens)
}

pub fn parse_script_string<'input>(
    input: &'input str,
) -> Result<Script, ParseError<usize, failure::Error>> {
    let mut tokens = Lexer::new(input);
    tokens.advance()?;
    parse_script(&mut tokens)
}

pub fn parse_script_or_module_string<'input>(
    input: &'input str,
) -> Result<ScriptOrModule, ParseError<usize, failure::Error>> {
    let mut tokens = Lexer::new(input);
    tokens.advance()?;
    parse_script_or_module(&mut tokens)
}
