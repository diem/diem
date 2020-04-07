// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use proptest::sample::Index as PropIndex;
use vm::file_format::{
    CompiledModuleMut, Signature, SignatureToken, StructFieldInformation, TypeSignature,
};

pub struct SignatureRefMutation<'a> {
    module: &'a mut CompiledModuleMut,
    mutations: Vec<(PropIndex, PropIndex)>,
}

impl<'a> SignatureRefMutation<'a> {
    pub fn new(module: &'a mut CompiledModuleMut, mutations: Vec<(PropIndex, PropIndex)>) -> Self {
        Self { module, mutations }
    }

    pub fn apply(self) -> bool {
        if self.module.signatures.is_empty() {
            return false;
        }
        let module = self.module;
        let mut mutations = false;
        for (s_idx, t_idx) in self.mutations {
            let sig_idx = s_idx.index(module.signatures.len());
            let sig = &module.signatures[sig_idx];
            if sig.is_empty() {
                continue;
            }
            let token_idx = t_idx.index(sig.len());
            let new_sig = mutate_sig(sig, token_idx);
            module.signatures[sig_idx] = new_sig;
            mutations = true;
        }
        mutations
    }
}

/// Context for applying a list of `FieldRefMutation` instances.
pub struct FieldRefMutation<'a> {
    module: &'a mut CompiledModuleMut,
    mutations: Vec<(PropIndex, PropIndex)>,
}

impl<'a> FieldRefMutation<'a> {
    pub fn new(module: &'a mut CompiledModuleMut, mutations: Vec<(PropIndex, PropIndex)>) -> Self {
        Self { module, mutations }
    }

    #[inline]
    pub fn apply(self) -> bool {
        if self.module.struct_defs.is_empty() {
            return false;
        }
        let module = self.module;
        let mut mutations = false;
        for (s_idx, f_idx) in self.mutations {
            let struct_idx = s_idx.index(module.struct_defs.len());
            let struct_def = &mut module.struct_defs[struct_idx];
            if let StructFieldInformation::Declared(fields) = &mut struct_def.field_information {
                if fields.is_empty() {
                    continue;
                }
                let field_idx = f_idx.index(fields.len());
                let field_def = &mut fields[field_idx];
                let new_ty = mutate_field(&field_def.signature.0);
                fields[field_idx].signature = TypeSignature(new_ty);
                mutations = true;
            }
        }
        mutations
    }
}

fn mutate_sig(sig: &Signature, token_idx: usize) -> Signature {
    use SignatureToken::*;

    Signature(
        sig.0
            .iter()
            .enumerate()
            .map(|(idx, token)| {
                if idx == token_idx {
                    match &token {
                        Reference(_) | MutableReference(_) => Reference(Box::new(token.clone())),
                        _ => Reference(Box::new(Reference(Box::new(token.clone())))),
                    }
                } else {
                    token.clone()
                }
            })
            .collect(),
    )
}

fn mutate_field(token: &SignatureToken) -> SignatureToken {
    SignatureToken::Reference(Box::new(token.clone()))
}
