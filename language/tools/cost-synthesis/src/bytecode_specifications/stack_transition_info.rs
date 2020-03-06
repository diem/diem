// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Encodes the typed value-stack transitions for the VM bytecode.
//!
//! This file encodes the typed stack transitions for the bytecode according to
//! the rules that are outlined in the bytecode file format.
//! Each instructions type transition is encoded as
//!         [(source_tys_list, output_tys_list), ..., (source_tys_list, output_tys_list)]
//! We encode each instruction with the number of arguments (and the valid types
//! that these arguments can take), and the number of (type) outputs from that instruction.
use once_cell::sync::Lazy;
use std::{boxed::Box, u8::MAX};
use vm::file_format::{Bytecode, SignatureToken, StructHandleIndex, TypeSignature};

const MAX_INSTRUCTION_LEN: u8 = MAX;

/// Represents a type that a given position can be -- fixed, or variable.
///
/// We want to be able to represent that a given set of type positions can have multiple types
/// irrespective of the other types around it possibly (i.e. that the type can vary independently).
/// These are represented as `Variable` types. Likewise, we want to be able to also say that a type
/// is `Fixed`.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum SignatureTy {
    Fixed(TypeSignature),
    Variable(Vec<TypeSignature>, u8),
}

/// Holds the type details (input/output) for a bytecode instruction.
pub struct CallDetails {
    /// The types of the input arguments expected on the value stack.
    pub in_args: Vec<SignatureTy>,

    /// The types of the outputs placed at the top of the value stack after the execution of the
    /// instruction.
    pub out_args: Vec<SignatureTy>,
}

impl SignatureTy {
    /// This method returns the underlying type(s) of the `SignatureTy`. It discards the
    /// information on whether the types returned were `Fixed` or `Variable`.
    pub fn underlying(self) -> Vec<TypeSignature> {
        match self {
            SignatureTy::Fixed(t) => vec![t],
            SignatureTy::Variable(tys, _) => tys,
        }
    }

    /// A predicate returning whether the type is fixed.
    pub fn is_fixed(&self) -> bool {
        match self {
            SignatureTy::Fixed(_) => true,
            SignatureTy::Variable(_, _) => false,
        }
    }

    /// A predicate returning whether the type is variable.
    pub fn is_variable(&self) -> bool {
        !self.is_fixed()
    }
}

// The base types in our system.
//
// For any instruction that can take a stack value of any base type
// we represent it as a variable type over all base types for the bytecode.
static BASE_SIG_TOKENS: Lazy<Vec<SignatureToken>> = Lazy::new(|| {
    vec![
        SignatureToken::Bool,
        SignatureToken::U8,
        SignatureToken::U64,
        SignatureToken::U128,
        SignatureToken::Address,
        // Bogus struct handle index, but it's fine since we disregard this in the generation of
        // instruction arguments.
        SignatureToken::Struct(StructHandleIndex::new(0), vec![]),
    ]
});

static INTEGER_SIG_TOKENS: Lazy<Vec<SignatureToken>> = Lazy::new(|| {
    vec![
        SignatureToken::U8,
        SignatureToken::U64,
        SignatureToken::U128,
    ]
});

fn variable_ty_of_sig_tok(tok: Vec<SignatureToken>, len: u8) -> SignatureTy {
    let typs = tok.into_iter().map(TypeSignature).collect();
    SignatureTy::Variable(typs, len)
}

fn ty_of_sig_tok(tok: SignatureToken) -> SignatureTy {
    SignatureTy::Fixed(TypeSignature(tok))
}

fn simple_ref_of_sig_tok(tok: SignatureToken) -> SignatureTy {
    SignatureTy::Fixed(TypeSignature(SignatureToken::Reference(Box::new(tok))))
}

fn simple_ref_of_sig_toks(toks: Vec<SignatureToken>) -> SignatureTy {
    let len = toks.len() as u8;
    let types = toks
        .into_iter()
        .map(|tok| TypeSignature(SignatureToken::Reference(Box::new(tok))))
        .collect();
    SignatureTy::Variable(types, len)
}

fn ref_values(num: u64) -> Vec<SignatureTy> {
    (0..num)
        .map(|_| simple_ref_of_sig_toks(BASE_SIG_TOKENS.clone()))
        .collect()
}

fn ref_resources(num: u64) -> Vec<SignatureTy> {
    (0..num)
        .map(|_| simple_ref_of_sig_tok(SignatureToken::Struct(StructHandleIndex::new(0), vec![])))
        .collect()
}

fn bools(num: u64) -> Vec<SignatureTy> {
    (0..num)
        .map(|_| ty_of_sig_tok(SignatureToken::Bool))
        .collect()
}

fn u8s(num: u64) -> Vec<SignatureTy> {
    (0..num)
        .map(|_| ty_of_sig_tok(SignatureToken::U8))
        .collect()
}

fn u64s(num: u64) -> Vec<SignatureTy> {
    (0..num)
        .map(|_| ty_of_sig_tok(SignatureToken::U64))
        .collect()
}

fn u128s(num: u64) -> Vec<SignatureTy> {
    (0..num)
        .map(|_| ty_of_sig_tok(SignatureToken::U128))
        .collect()
}

fn simple_addrs(num: u64) -> Vec<SignatureTy> {
    (0..num)
        .map(|_| ty_of_sig_tok(SignatureToken::Address))
        .collect()
}

fn byte_arrays(num: u64) -> Vec<SignatureTy> {
    (0..num)
        .map(|_| ty_of_sig_tok(SignatureToken::Vector(Box::new(SignatureToken::U8))))
        .collect()
}

// Creates `num` independently variable types
fn values(num: u64) -> Vec<SignatureTy> {
    (0..num)
        .map(|_| variable_ty_of_sig_tok(BASE_SIG_TOKENS.clone(), BASE_SIG_TOKENS.len() as u8))
        .collect()
}

// creates `num` rows of a single type. Used for Eq and Neq
fn non_variable_values(num: u64) -> Vec<Vec<SignatureTy>> {
    BASE_SIG_TOKENS
        .iter()
        .map(|ty| (0..num).map(|_| ty_of_sig_tok(ty.clone())).collect())
        .collect()
}

fn integer_values(num: u64) -> Vec<Vec<SignatureTy>> {
    INTEGER_SIG_TOKENS
        .iter()
        .map(|ty| (0..num).map(|_| ty_of_sig_tok(ty.clone())).collect())
        .collect()
}

fn resources(num: u64) -> Vec<SignatureTy> {
    (0..num)
        .map(|_| ty_of_sig_tok(SignatureToken::Struct(StructHandleIndex::new(0), vec![])))
        .collect()
}

fn empty() -> Vec<SignatureTy> {
    vec![]
}

fn type_transitions(args: Vec<(Vec<SignatureTy>, Vec<SignatureTy>)>) -> Vec<CallDetails> {
    args.into_iter()
        .map(|(in_args, out_args)| CallDetails { in_args, out_args })
        .collect()
}

macro_rules! type_transition {
    (fixed: $e1:expr => $e2:expr) => {
        $e1.into_iter().flat_map(|vec| {
            type_transitions(vec![ (vec,$e2.clone())]).into_iter()
        }).collect()
    };
    ($($e1:expr => $e2:expr),+) => {
        type_transitions(vec![ $(($e1,$e2)),+ ])
    };
}

/// Given an instruction `op` return back the type-level stack only call details.
pub fn call_details(op: &Bytecode) -> Vec<CallDetails> {
    match op {
        Bytecode::Add
        | Bytecode::Sub
        | Bytecode::Mul
        | Bytecode::Mod
        | Bytecode::Div
        | Bytecode::BitOr
        | Bytecode::BitAnd
        | Bytecode::Xor => type_transition! {
            u8s(2) => u8s(1),
            u64s(2) => u64s(1),
            u128s(2) => u128s(1)
        },
        // TODO: rewrite in a more efficient way.
        Bytecode::Shl | Bytecode::Shr => type_transition! {
            vec![ty_of_sig_tok(SignatureToken::U8), ty_of_sig_tok(SignatureToken::U8)] => u8s(1),
            vec![ty_of_sig_tok(SignatureToken::U64), ty_of_sig_tok(SignatureToken::U8)] => u64s(1),
            vec![ty_of_sig_tok(SignatureToken::U128), ty_of_sig_tok(SignatureToken::U8)] => u128s(1)
        },
        Bytecode::Eq | Bytecode::Neq => type_transition! {
            fixed: non_variable_values(2) => bools(1)
        },
        Bytecode::Pop => type_transition! {
            values(1) => empty(),
            ref_values(1) => empty(),
            ref_resources(1) => empty()
        },
        Bytecode::LdU8(_) => type_transition! { empty() => u8s(1) },
        Bytecode::LdU64(_) => type_transition! { empty() => u64s(1) },
        Bytecode::LdU128(_) => type_transition! { empty() => u128s(1) },
        Bytecode::CastU8 => type_transition! { fixed: integer_values(1) => u8s(1) },
        Bytecode::CastU64 => type_transition! { fixed: integer_values(1) => u64s(1) },
        Bytecode::CastU128 => type_transition! { fixed: integer_values(1) => u128s(1) },
        Bytecode::LdAddr(_) => type_transition! { empty() => simple_addrs(1) },
        Bytecode::LdByteArray(_) => type_transition! { empty() => byte_arrays(1) },
        Bytecode::LdFalse | Bytecode::LdTrue => type_transition! { empty() => bools(1) },
        Bytecode::BrTrue(_) | Bytecode::BrFalse(_) => {
            type_transition! { bools(1) => empty() }
        }
        Bytecode::Abort => {
            type_transition! { u64s(1) => empty() }
        }
        Bytecode::Branch(_) => type_transition! { empty() => empty() },
        Bytecode::StLoc(_) => type_transition! { resources(1) => empty(), values(1) => empty() },
        Bytecode::CopyLoc(_) => type_transition! {
            empty() => values(1),
            empty() => ref_values(1),
            empty() => ref_resources(1)
        },
        Bytecode::MoveLoc(_) => type_transition! {
            empty() => values(1),
            empty() => resources(1),
            empty() => ref_values(1),
            empty() => ref_resources(1)
        },
        Bytecode::MutBorrowLoc(_)
        | Bytecode::ImmBorrowLoc(_)
        | Bytecode::ImmBorrowField(_)
        | Bytecode::MutBorrowField(_) => {
            type_transition! { empty() => ref_values(1), empty() => ref_resources(1) }
        }
        Bytecode::ReadRef => type_transition! { ref_values(1) => values(1) },
        Bytecode::WriteRef => {
            let mut input_tys = values(1);
            input_tys.append(&mut ref_values(1));
            type_transition! {
                input_tys => empty()
            }
        }
        Bytecode::Lt | Bytecode::Gt | Bytecode::Le | Bytecode::Ge => {
            type_transition! { fixed: integer_values(2) => bools(1) }
        }
        Bytecode::And | Bytecode::Or => type_transition! { bools(2) => bools(1) },
        Bytecode::Not => type_transition! { bools(1) => bools(1) },
        Bytecode::Ret => type_transition! {
            values(1) => empty(),
            resources(1) => empty(),
            ref_values(1) => empty(),
            ref_resources(1) => empty()
        },
        Bytecode::Pack(_, _) | Bytecode::Call(_, _) => {
            let possible_tys = BASE_SIG_TOKENS.clone();
            type_transition! {
                          vec![variable_ty_of_sig_tok(
                              possible_tys.clone(),
                              MAX_INSTRUCTION_LEN,
                          )] => vec![variable_ty_of_sig_tok(possible_tys, 1)]
            }
        }
        Bytecode::Unpack(_, _) => {
            let possible_tys = BASE_SIG_TOKENS.clone();
            type_transition! {
                vec![variable_ty_of_sig_tok(
                    possible_tys.clone(),
                    1,
            )] => vec![variable_ty_of_sig_tok(possible_tys, MAX_INSTRUCTION_LEN)]
            }
        }
        Bytecode::GetTxnGasUnitPrice
        | Bytecode::GetTxnSequenceNumber
        | Bytecode::GetTxnMaxGasUnits
        | Bytecode::GetGasRemaining => type_transition! { empty() => u64s(1) },
        Bytecode::GetTxnSenderAddress => type_transition! { empty() => simple_addrs(1) },
        Bytecode::Exists(_, _) => type_transition! { simple_addrs(1) => bools(1) },
        Bytecode::MutBorrowGlobal(_, _) | Bytecode::ImmBorrowGlobal(_, _) => {
            type_transition! { simple_addrs(1) => ref_values(1) }
        }
        Bytecode::MoveFrom(_, _) => type_transition! { simple_addrs(1) => values(1) },
        Bytecode::MoveToSender(_, _) => type_transition! { values(1) => empty() },
        Bytecode::GetTxnPublicKey => type_transition! { empty() => byte_arrays(1) },
        Bytecode::FreezeRef => type_transition! { ref_values(1) => ref_values(1) },
    }
}
