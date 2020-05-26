// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{interpreter::Interpreter, loader::Loader};
use bytecode_verifier::VerifiedModule;
use libra_logger::prelude::*;
use libra_types::vm_error::{StatusCode, VMStatus};
use move_core_types::{
    account_address::AccountAddress,
    gas_schedule::{AbstractMemorySize, GasCarrier},
    identifier::IdentStr,
    language_storage::{ModuleId, TypeTag},
};
use move_vm_types::{data_store::DataStore, gas_schedule::CostStrategy, values::Value};
use vm::{
    access::ModuleAccess,
    errors::{verification_error, vm_error, Location, VMResult},
    file_format::{Signature, SignatureToken},
    CompiledModule, IndexKind,
};

/// An instantiation of the MoveVM.
pub(crate) struct VMRuntime {
    loader: Loader,
}

impl VMRuntime {
    pub(crate) fn new() -> Self {
        VMRuntime {
            loader: Loader::new(),
        }
    }

    pub(crate) fn publish_module(
        &self,
        module: Vec<u8>,
        sender: &AccountAddress,
        data_store: &mut dyn DataStore,
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
        if compiled_module.address() != sender {
            return Err(verification_error(
                IndexKind::AddressIdentifier,
                compiled_module.self_handle_idx().0 as usize,
                StatusCode::MODULE_ADDRESS_DOES_NOT_MATCH_SENDER,
            ));
        }

        // Make sure that there is not already a module with this name published
        // under the transaction sender's account.
        let module_id = compiled_module.self_id();
        if data_store.exists_module(&module_id) {
            return Err(vm_error(
                Location::default(),
                StatusCode::DUPLICATE_MODULE_NAME,
            ));
        };

        let verified_module = VerifiedModule::new(compiled_module).map_err(|(_, e)| e)?;
        Loader::check_natives(&verified_module)?;
        data_store.publish_module(module_id, module)
    }

    pub(crate) fn execute_script(
        &self,
        script: Vec<u8>,
        ty_args: Vec<TypeTag>,
        mut args: Vec<Value>,
        sender: AccountAddress,
        txn_size: AbstractMemorySize<GasCarrier>,
        data_store: &mut dyn DataStore,
        cost_strategy: &mut CostStrategy,
    ) -> VMResult<()> {
        fn is_signer_reference(s: &SignatureToken) -> bool {
            use SignatureToken as S;
            match s {
                S::Reference(inner) => matches!(&**inner, S::Signer),
                _ => false,
            }
        }

        let mut type_params = vec![];
        for ty in &ty_args {
            type_params.push(self.loader.load_type(ty, data_store)?);
        }
        let main = self.loader.load_script(&script, data_store)?;

        self.loader
            .verify_ty_args(main.type_parameters(), &type_params)?;
        let first_param_opt = main.parameters().0.get(0);
        if first_param_opt.map_or(false, |sig| is_signer_reference(sig)) {
            args.insert(0, Value::transaction_argument_signer_reference(sender))
        }
        verify_args(main.parameters(), &args)?;

        Interpreter::entrypoint(
            main,
            type_params,
            args,
            sender,
            txn_size,
            data_store,
            cost_strategy,
            &self.loader,
        )
    }

    pub(crate) fn execute_function(
        &self,
        module: &ModuleId,
        function_name: &IdentStr,
        ty_args: Vec<TypeTag>,
        args: Vec<Value>,
        sender: AccountAddress,
        txn_size: AbstractMemorySize<GasCarrier>,
        data_store: &mut dyn DataStore,
        cost_strategy: &mut CostStrategy,
    ) -> VMResult<()> {
        let mut type_params = vec![];
        for ty in &ty_args {
            type_params.push(self.loader.load_type(ty, data_store)?);
        }
        let func = self
            .loader
            .load_function(function_name, module, data_store)?;

        self.loader
            .verify_ty_args(func.type_parameters(), &type_params)?;
        // REVIEW: argument verification should happen in the interpreter
        //verify_args(func.parameters(), &args)?;

        Interpreter::entrypoint(
            func,
            type_params,
            args,
            sender,
            txn_size,
            data_store,
            cost_strategy,
            &self.loader,
        )
    }

    pub(crate) fn cache_module(
        &self,
        module: VerifiedModule,
        data_store: &mut dyn DataStore,
    ) -> VMResult<()> {
        self.loader.cache_module(module, data_store)
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
