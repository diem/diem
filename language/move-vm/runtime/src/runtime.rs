// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{interpreter::Interpreter, loader::Loader};
use libra_logger::prelude::*;
use libra_types::vm_error::{StatusCode, VMStatus};
use move_core_types::{
    account_address::AccountAddress,
    identifier::IdentStr,
    language_storage::{ModuleId, TypeTag},
};
use move_vm_types::{data_store::DataStore, gas_schedule::CostStrategy, values::Value};
use vm::{
    access::ModuleAccess,
    errors::{verification_error, vm_error, Location, VMResult},
    file_format::SignatureToken,
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
        sender: AccountAddress,
        data_store: &mut dyn DataStore,
        _cost_strategy: &mut CostStrategy,
    ) -> VMResult<()> {
        // deserialize the module. Perform bounds check. After this indexes can be
        // used with the `[]` operator
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
        if compiled_module.address() != &sender {
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

        // perform bytecode and loading verification
        self.loader.verify_module(&compiled_module)?;

        data_store.publish_module(module_id, module)
    }

    pub(crate) fn execute_script(
        &self,
        script: Vec<u8>,
        ty_args: Vec<TypeTag>,
        mut args: Vec<Value>,
        sender: AccountAddress,
        data_store: &mut dyn DataStore,
        cost_strategy: &mut CostStrategy,
    ) -> VMResult<()> {
        // signer helper closure
        fn is_signer_reference(s: &SignatureToken) -> bool {
            use SignatureToken as S;
            match s {
                S::Reference(inner) => matches!(&**inner, S::Signer),
                _ => false,
            }
        }

        // load the script, perform verification
        let (main, type_params) = self.loader.load_script(&script, &ty_args, data_store)?;

        // build the arguments list for the main and check the arguments are of restricted types
        let first_param_opt = main.parameters().0.get(0);
        if first_param_opt.map_or(false, |sig| is_signer_reference(sig)) {
            args.insert(0, Value::transaction_argument_signer_reference(sender))
        }
        check_args(&args)?;

        // run the script
        Interpreter::entrypoint(
            main,
            type_params,
            args,
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
        data_store: &mut dyn DataStore,
        cost_strategy: &mut CostStrategy,
    ) -> VMResult<()> {
        // load the function in the given module, perform verification of the module and
        // its dependencies if the module was not loaded
        let (func, type_params) =
            self.loader
                .load_function(function_name, module, &ty_args, data_store)?;

        // check the arguments provided are of restricted types
        check_args(&args)?;

        // run the function
        Interpreter::entrypoint(
            func,
            type_params,
            args,
            data_store,
            cost_strategy,
            &self.loader,
        )
    }
}

/// Check that the transaction arguments are acceptable by the VM.
/// Constants are the only arguments allowed.
fn check_args(args: &[Value]) -> VMResult<()> {
    for val in args {
        if !val.is_constant_or_signer_ref() {
            return Err(VMStatus::new(StatusCode::TYPE_MISMATCH)
                .with_message("VM argument types are restricted".to_string()));
        }
    }
    Ok(())
}
