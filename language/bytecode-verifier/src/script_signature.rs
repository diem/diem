// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module implements a checker for verifying that a script (or script function when serving
//! as the entry point for script execution) has a valid signature, which entails
//! - All signer arguments are occur before non-signer arguments
//! - All types non-signer arguments have a type that is valid for constants
//! - Has an empty return type
use move_binary_format::{
    access::ModuleAccess,
    binary_views::BinaryIndexedView,
    errors::{Location, PartialVMError, PartialVMResult, VMResult},
    file_format::{
        CompiledModule, CompiledScript, SignatureIndex, SignatureToken, TableIndex, Visibility,
    },
    file_format_common::VERSION_1,
    IndexKind,
};
use move_core_types::{identifier::IdentStr, vm_status::StatusCode};

/// This function checks the extra requirements on the signature of the main function of a script.
pub fn verify_script(script: &CompiledScript) -> VMResult<()> {
    let resolver = &BinaryIndexedView::Script(script);
    let parameters = script.parameters;
    let return_type_opt = None;
    verify_main_signature_impl(resolver, parameters, return_type_opt)
        .map_err(|e| e.finish(Location::Script))
}

/// This function checks the extra requirements on the signature of the script visible function
/// when it serves as an entry point for script execution
pub fn verify_module_script_function(module: &CompiledModule, name: &IdentStr) -> VMResult<()> {
    let fdef_opt = module.function_defs().iter().enumerate().find(|(_, fdef)| {
        module.identifier_at(module.function_handle_at(fdef.function).name) == name
    });
    let (idx, fdef) = fdef_opt.ok_or_else(|| {
        PartialVMError::new(StatusCode::VERIFICATION_ERROR)
            .with_message("function not found in verify_module_script_function".to_string())
            .finish(Location::Module(module.self_id()))
    })?;

    match fdef.visibility {
        Visibility::Script => (),
        Visibility::Private | Visibility::Friend | Visibility::Public => {
            return Err(PartialVMError::new(
                StatusCode::EXECUTE_SCRIPT_FUNCTION_CALLED_ON_NON_SCRIPT_VISIBLE,
            )
            .at_index(IndexKind::FunctionDefinition, idx as TableIndex)
            .finish(Location::Module(module.self_id())))
        }
    }

    let resolver = &BinaryIndexedView::Module(module);
    let fhandle = module.function_handle_at(fdef.function);
    let parameters = fhandle.parameters;
    let return_type_opt = Some(fhandle.return_);
    verify_main_signature_impl(resolver, parameters, return_type_opt).map_err(|e| {
        e.at_index(IndexKind::FunctionDefinition, idx as TableIndex)
            .finish(Location::Module(module.self_id()))
    })
}

fn verify_main_signature_impl(
    resolver: &BinaryIndexedView,
    parameters: SignatureIndex,
    return_type_opt: Option<SignatureIndex>,
) -> PartialVMResult<()> {
    use SignatureToken as S;
    let arguments = &resolver.signature_at(parameters).0;
    // Check that all `signer` arguments occur before non-`signer` arguments
    // signer is a type that can only be populated by the Move VM. And its value is filled
    // based on the sender of the transaction
    let all_args_have_valid_type = if resolver.version() <= VERSION_1 {
        arguments
            .iter()
            .skip_while(|typ| matches!(typ, S::Reference(inner) if matches!(&**inner, S::Signer)))
            .all(|typ| typ.is_valid_for_constant())
    } else {
        arguments
            .iter()
            .skip_while(|typ| matches!(typ, S::Signer))
            .all(|typ| typ.is_valid_for_constant())
    };
    let has_valid_return_type = match return_type_opt {
        Some(idx) => resolver.signature_at(idx).0.is_empty(),
        None => true,
    };
    if !all_args_have_valid_type || !has_valid_return_type {
        Err(PartialVMError::new(
            StatusCode::INVALID_MAIN_FUNCTION_SIGNATURE,
        ))
    } else {
        Ok(())
    }
}
