// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module implements a checker for verifying that a non-resource struct does not
//! have resource fields inside it.
use diem_types::vm_status::StatusCode;
use vm::{
    access::ModuleAccess,
    errors::{verification_error, Location, PartialVMResult, VMResult},
    file_format::{CompiledModule, Kind, SignatureToken, StructFieldInformation, TableIndex},
    IndexKind,
};

pub struct ResourceTransitiveChecker<'a> {
    module: &'a CompiledModule,
}

impl<'a> ResourceTransitiveChecker<'a> {
    pub fn verify_module(module: &'a CompiledModule) -> VMResult<()> {
        Self::verify_module_impl(module).map_err(|e| e.finish(Location::Module(module.self_id())))
    }

    fn verify_module_impl(module: &'a CompiledModule) -> PartialVMResult<()> {
        let checker = Self { module };
        for (idx, struct_def) in checker.module.struct_defs().iter().enumerate() {
            let sh = checker.module.struct_handle_at(struct_def.struct_handle);
            if sh.is_nominal_resource {
                continue;
            }
            let fields = match &struct_def.field_information {
                StructFieldInformation::Native => continue,
                StructFieldInformation::Declared(fields) => fields,
            };
            for field in fields {
                if checker.contains_nominal_resource(&field.signature.0, &sh.type_parameters) {
                    return Err(verification_error(
                        StatusCode::INVALID_RESOURCE_FIELD,
                        IndexKind::StructDefinition,
                        idx as TableIndex,
                    ));
                }
            }
        }
        Ok(())
    }

    /// Determines if the given signature token contains a nominal resource.
    /// More specifically, a signature token contains a nominal resource if
    ///   1) it is a type variable explicitly marked as resource kind.
    ///   2) it is a signer, which is always a resource type
    ///   3) it is a struct that
    ///       a) is marked as resource.
    ///       b) has a type actual which is a nominal resource.
    fn contains_nominal_resource(&self, token: &SignatureToken, type_parameters: &[Kind]) -> bool {
        match token {
            SignatureToken::Signer => true,
            SignatureToken::Struct(sh_idx) => {
                let sh = self.module.struct_handle_at(*sh_idx);
                sh.is_nominal_resource
            }
            SignatureToken::StructInstantiation(sh_idx, type_arguments) => {
                let sh = self.module.struct_handle_at(*sh_idx);
                if sh.is_nominal_resource {
                    return true;
                }
                for token in type_arguments {
                    if self.contains_nominal_resource(token, type_parameters) {
                        return true;
                    }
                }
                false
            }
            SignatureToken::Vector(ty) => self.contains_nominal_resource(ty, type_parameters),
            SignatureToken::Reference(_)
            | SignatureToken::MutableReference(_)
            | SignatureToken::Bool
            | SignatureToken::U8
            | SignatureToken::U64
            | SignatureToken::U128
            | SignatureToken::Address
            | SignatureToken::TypeParameter(_) => false,
        }
    }
}
