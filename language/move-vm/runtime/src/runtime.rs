// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{interpreter::Interpreter, interpreter_context::InterpreterContext, loader::Loader};
use bytecode_verifier::VerifiedModule;
use libra_logger::prelude::*;
use libra_types::{
    language_storage::{ModuleId, TypeTag},
    vm_error::{StatusCode, VMStatus},
};
use move_core_types::{gas_schedule::CostTable, identifier::IdentStr};
use move_vm_types::values::Value;
use vm::{
    access::ModuleAccess,
    errors::{verification_error, vm_error, Location, VMResult},
    file_format::Signature,
    transaction_metadata::TransactionMetadata,
    CompiledModule, IndexKind,
};

/// An instantiation of the MoveVM.
pub(crate) struct VMRuntime {
    loader: Loader,
}

impl VMRuntime {
    pub fn new() -> Self {
        VMRuntime {
            loader: Loader::new(),
        }
    }

    pub(crate) fn publish_module(
        &self,
        module: Vec<u8>,
        context: &mut dyn InterpreterContext,
        txn_data: &TransactionMetadata,
    ) -> VMResult<()> {
        let compiled_module = match CompiledModule::deserialize(&module) {
            Ok(module) => module,
            Err(err) => {
                warn!("[VM] module deserialization failed {:?}", err);
                return Err(err);
            }
        };

        // Make sure the module's self address matches the transaction sender. The self address is
        // where the module will actually be published. If we did not check this, the sender could
        // publish a module under anyone's account.
        if compiled_module.address() != &txn_data.sender {
            return Err(verification_error(
                IndexKind::AddressIdentifier,
                CompiledModule::IMPLEMENTED_MODULE_INDEX as usize,
                StatusCode::MODULE_ADDRESS_DOES_NOT_MATCH_SENDER,
            ));
        }

        // Make sure that there is not already a module with this name published
        // under the transaction sender's account.
        let module_id = compiled_module.self_id();
        if context.exists_module(&module_id) {
            return Err(vm_error(
                Location::default(),
                StatusCode::DUPLICATE_MODULE_NAME,
            ));
        };

        match VerifiedModule::new(compiled_module) {
            Ok(ver_module) => ver_module,
            Err((_, mut errors)) => {
                let err = if errors.is_empty() {
                    VMStatus::new(StatusCode::VERIFIER_INVARIANT_VIOLATION)
                } else {
                    errors.remove(0)
                };

                return Err(err);
            }
        };

        context.publish_module(module_id, module)
    }

    pub fn execute_script(
        &self,
        context: &mut dyn InterpreterContext,
        txn_data: &TransactionMetadata,
        gas_schedule: &CostTable,
        script: Vec<u8>,
        ty_args: Vec<TypeTag>,
        args: Vec<Value>,
    ) -> VMResult<()> {
        let mut type_params = vec![];
        for ty in &ty_args {
            type_params.push(self.loader.load_type(ty, context)?);
        }
        let main = self.loader.load_script(&script, context)?;

        self.loader
            .verify_ty_args(main.type_parameters(), &type_params)?;
        verify_args(main.parameters(), &args)?;

        Interpreter::entrypoint(
            context,
            &self.loader,
            txn_data,
            gas_schedule,
            main,
            type_params,
            args,
        )
    }

    pub fn execute_function(
        &self,
        context: &mut dyn InterpreterContext,
        txn_data: &TransactionMetadata,
        gas_schedule: &CostTable,
        module: &ModuleId,
        function_name: &IdentStr,
        ty_args: Vec<TypeTag>,
        args: Vec<Value>,
    ) -> VMResult<()> {
        let mut type_params = vec![];
        for ty in &ty_args {
            type_params.push(self.loader.load_type(ty, context)?);
        }
        let func = self.loader.load_function(function_name, module, context)?;

        self.loader
            .verify_ty_args(func.type_parameters(), &type_params)?;
        // REVIEW: argument verification should happen in the interpreter
        //verify_args(func.parameters(), &args)?;

        Interpreter::entrypoint(
            context,
            &self.loader,
            txn_data,
            gas_schedule,
            func,
            type_params,
            args,
        )
    }

    pub fn cache_module(
        &self,
        module: VerifiedModule,
        context: &mut dyn InterpreterContext,
    ) -> VMResult<()> {
        self.loader.cache_module(module, context)
    }
}

/// Verify if the transaction arguments match the type signature of the main function.
fn verify_args(signature: &Signature, args: &[Value]) -> VMResult<()> {
    if signature.len() != args.len() {
        return Err(
            VMStatus::new(StatusCode::TYPE_MISMATCH).with_message(format!(
                "argument length mismatch: expected {} got {}",
                signature.len(),
                args.len()
            )),
        );
    }
    for (tok, val) in signature.0.iter().zip(args) {
        if !val.is_valid_script_arg(tok) {
            return Err(VMStatus::new(StatusCode::TYPE_MISMATCH)
                .with_message("argument type mismatch".to_string()));
        }
    }
    Ok(())
}
