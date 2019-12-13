// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_proptest_helpers::{pick_slice_idxs, RepeatVec};
use libra_types::vm_error::{StatusCode, VMStatus};
use proptest::{
    prelude::*,
    sample::{select, Index as PropIndex},
};
use std::{collections::BTreeMap, iter};
use vm::{
    errors::append_err_info,
    file_format::{CompiledModuleMut, SignatureToken},
    internals::ModuleIndex,
    IndexKind, SignatureTokenKind,
};

/// Represents a mutation that wraps a signature token up in a double reference (or an array of
/// references.
#[derive(Clone, Debug)]
pub struct DoubleRefMutation {
    idx: PropIndex,
    kind: DoubleRefMutationKind,
}

impl DoubleRefMutation {
    pub fn strategy() -> impl Strategy<Value = Self> {
        (any::<PropIndex>(), DoubleRefMutationKind::strategy())
            .prop_map(|(idx, kind)| Self { idx, kind })
    }
}

impl AsRef<PropIndex> for DoubleRefMutation {
    #[inline]
    fn as_ref(&self) -> &PropIndex {
        &self.idx
    }
}

/// Context for applying a list of `DoubleRefMutation` instances.
pub struct ApplySignatureDoubleRefContext<'a> {
    module: &'a mut CompiledModuleMut,
    mutations: Vec<DoubleRefMutation>,
}

impl<'a> ApplySignatureDoubleRefContext<'a> {
    pub fn new(module: &'a mut CompiledModuleMut, mutations: Vec<DoubleRefMutation>) -> Self {
        Self { module, mutations }
    }

    pub fn apply(self) -> Vec<VMStatus> {
        // Apply double refs before field refs -- XXX is this correct?
        let sig_indexes = self.all_sig_indexes();
        let picked = sig_indexes.pick_uniform(&self.mutations);

        let mut errs = vec![];

        for (double_ref, (sig_idx, idx2)) in self.mutations.iter().zip(picked) {
            // When there's one level of indexing (e.g. Type), idx2 represents that level.
            // When there's two levels of indexing (e.g. FunctionArg), idx1 represents the outer
            // level (signature index) and idx2 the inner level (token index).
            let (token, kind, error_idx) = match sig_idx {
                SignatureIndex::Type => (
                    &mut self.module.type_signatures[idx2].0,
                    IndexKind::TypeSignature,
                    idx2,
                ),
                SignatureIndex::FunctionReturn(idx1) => (
                    &mut self.module.function_signatures[*idx1].return_types[idx2],
                    IndexKind::FunctionSignature,
                    *idx1,
                ),
                SignatureIndex::FunctionArg(idx1) => (
                    &mut self.module.function_signatures[*idx1].arg_types[idx2],
                    IndexKind::FunctionSignature,
                    *idx1,
                ),
                SignatureIndex::Locals(idx1) => (
                    &mut self.module.locals_signatures[*idx1].0[idx2],
                    IndexKind::LocalsSignature,
                    *idx1,
                ),
            };

            *token = double_ref.kind.wrap(token.clone());
            let msg = format!(
                "At index {} with kind {} with token {:#?}, with outer kind {} and inner kind {}",
                error_idx,
                kind,
                token.clone(),
                double_ref.kind.outer,
                double_ref.kind.inner
            );
            // If a locals signature is used by more than one functions and contains an error, it
            // should be reported multiple times. The following code emulates this behavior to match
            // the new implementation of the signature checker.
            // TODO: Revisit this and see if it's still required once we rework prop tests.
            match sig_idx {
                SignatureIndex::Locals(idx) => {
                    let n_references = self
                        .module
                        .function_defs
                        .iter()
                        .filter(|def| !def.is_native() && def.code.locals.0 as usize == *idx)
                        .count();
                    errs.extend(
                        iter::repeat_with(|| {
                            VMStatus::new(StatusCode::INVALID_SIGNATURE_TOKEN)
                                .with_message(msg.clone())
                        })
                        .take(n_references),
                    );
                }
                _ => {
                    errs.push(VMStatus::new(StatusCode::INVALID_SIGNATURE_TOKEN).with_message(msg));
                }
            }
        }

        errs
    }

    fn all_sig_indexes(&self) -> RepeatVec<SignatureIndex> {
        let mut res = RepeatVec::new();
        res.extend(SignatureIndex::Type, self.module.type_signatures.len());
        for (idx, sig) in self.module.function_signatures.iter().enumerate() {
            res.extend(SignatureIndex::FunctionReturn(idx), sig.return_types.len());
        }
        for (idx, sig) in self.module.function_signatures.iter().enumerate() {
            res.extend(SignatureIndex::FunctionArg(idx), sig.arg_types.len());
        }
        for (idx, sig) in self.module.locals_signatures.iter().enumerate() {
            res.extend(SignatureIndex::Locals(idx), sig.0.len());
        }
        res
    }
}

/// Represents a mutation that turns a field definition's type into a reference.
#[derive(Clone, Debug)]
pub struct FieldRefMutation {
    idx: PropIndex,
    is_mutable: bool,
}

impl FieldRefMutation {
    pub fn strategy() -> impl Strategy<Value = Self> {
        (any::<PropIndex>(), any::<bool>()).prop_map(|(idx, is_mutable)| Self { idx, is_mutable })
    }
}

impl AsRef<PropIndex> for FieldRefMutation {
    #[inline]
    fn as_ref(&self) -> &PropIndex {
        &self.idx
    }
}

/// Context for applying a list of `FieldRefMutation` instances.
pub struct ApplySignatureFieldRefContext<'a> {
    module: &'a mut CompiledModuleMut,
    mutations: Vec<FieldRefMutation>,
}

impl<'a> ApplySignatureFieldRefContext<'a> {
    pub fn new(module: &'a mut CompiledModuleMut, mutations: Vec<FieldRefMutation>) -> Self {
        Self { module, mutations }
    }

    #[inline]
    pub fn apply(self) -> Vec<VMStatus> {
        // One field definition might be associated with more than one signature, so collect all
        // the interesting ones in a map of type_sig_idx => field_def_idx.
        let mut interesting_idxs = BTreeMap::new();
        for (field_def_idx, field_def) in self.module.field_defs.iter().enumerate() {
            interesting_idxs
                .entry(field_def.signature)
                .or_insert_with(|| vec![])
                .push(field_def_idx);
        }
        // Convert into a Vec of pairs to allow pick_slice_idxs return vvalues to work.
        let interesting_idxs: Vec<_> = interesting_idxs.into_iter().collect();

        let picked = pick_slice_idxs(interesting_idxs.len(), &self.mutations);
        let mut errs = vec![];
        for (mutation, picked_idx) in self.mutations.iter().zip(picked) {
            let (type_sig_idx, field_def_idxs) = &interesting_idxs[picked_idx];
            let token = &mut self.module.type_signatures[type_sig_idx.into_index()].0;
            let (new_token, token_kind) = if mutation.is_mutable {
                (
                    SignatureToken::MutableReference(Box::new(token.clone())),
                    SignatureTokenKind::MutableReference,
                )
            } else {
                (
                    SignatureToken::Reference(Box::new(token.clone())),
                    SignatureTokenKind::Reference,
                )
            };

            *token = new_token;

            let msg = format!("with token {:#?} of kind {}", token.clone(), token_kind);
            let violation = VMStatus::new(StatusCode::INVALID_FIELD_DEF).with_message(msg);
            errs.extend(field_def_idxs.iter().map(|field_def_idx| {
                append_err_info(
                    violation.clone(),
                    IndexKind::FieldDefinition,
                    *field_def_idx,
                )
            }));
        }

        errs
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum SignatureIndex {
    Type,
    FunctionReturn(usize),
    FunctionArg(usize),
    Locals(usize),
}

#[derive(Clone, Debug)]
struct DoubleRefMutationKind {
    outer: SignatureTokenKind,
    inner: SignatureTokenKind,
}

impl DoubleRefMutationKind {
    fn strategy() -> impl Strategy<Value = Self> {
        (Self::outer_strategy(), Self::inner_strategy())
            .prop_map(|(outer, inner)| Self { outer, inner })
    }

    fn wrap(&self, token: SignatureToken) -> SignatureToken {
        let token = Self::wrap_one(token, self.inner);
        Self::wrap_one(token, self.outer)
    }

    fn wrap_one(token: SignatureToken, kind: SignatureTokenKind) -> SignatureToken {
        match kind {
            SignatureTokenKind::Reference => SignatureToken::Reference(Box::new(token)),
            SignatureTokenKind::MutableReference => {
                SignatureToken::MutableReference(Box::new(token))
            }
            SignatureTokenKind::Value => panic!("invalid wrapping kind: {}", kind),
        }
    }

    #[inline]
    fn outer_strategy() -> impl Strategy<Value = SignatureTokenKind> {
        static VALID_OUTERS: &[SignatureTokenKind] = &[
            SignatureTokenKind::Reference,
            SignatureTokenKind::MutableReference,
        ];
        select(VALID_OUTERS)
    }

    #[inline]
    fn inner_strategy() -> impl Strategy<Value = SignatureTokenKind> {
        static VALID_INNERS: &[SignatureTokenKind] = &[
            SignatureTokenKind::Reference,
            SignatureTokenKind::MutableReference,
        ];

        select(VALID_INNERS)
    }
}
