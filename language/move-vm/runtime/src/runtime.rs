// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    data_cache::{RemoteCache, TransactionDataCache},
    interpreter::Interpreter,
    loader::Loader,
    session::Session,
};
use move_core_types::{
    account_address::AccountAddress,
    identifier::IdentStr,
    language_storage::{ModuleId, TypeTag},
    vm_status::StatusCode,
};
use move_vm_types::{
    data_store::DataStore, gas_schedule::CostStrategy, logger::Logger, values::Value,
};
use vm::{
    access::ModuleAccess,
    errors::{verification_error, Location, PartialVMError, PartialVMResult, VMResult},
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

    pub fn new_session<'r, R: RemoteCache>(&self, remote: &'r R) -> Session<'r, '_, R> {
        Session {
            runtime: self,
            data_cache: TransactionDataCache::new(remote, &self.loader),
        }
    }

    pub(crate) fn publish_module(
        &self,
        module: Vec<u8>,
        sender: AccountAddress,
        data_store: &mut impl DataStore,
        _cost_strategy: &mut CostStrategy,
        logger: &impl Logger,
    ) -> VMResult<()> {
        // deserialize the module. Perform bounds check. After this indexes can be
        // used with the `[]` operator
        let compiled_module = match CompiledModule::deserialize(&module) {
            Ok(module) => module,
            Err(err) => {
                logger.warn(format!("[VM] module deserialization failed {:?}", err).as_str());
                return Err(err.finish(Location::Undefined));
            }
        };

        // Make sure the module's self address matches the transaction sender. The self address is
        // where the module will actually be published. If we did not check this, the sender could
        // publish a module under anyone's account.
        if compiled_module.address() != &sender {
            return Err(verification_error(
                StatusCode::MODULE_ADDRESS_DOES_NOT_MATCH_SENDER,
                IndexKind::AddressIdentifier,
                compiled_module.self_handle_idx().0,
            )
            .finish(Location::Undefined));
        }

        // Make sure that there is not already a module with this name published
        // under the transaction sender's account.
        let module_id = compiled_module.self_id();
        if data_store.exists_module(&module_id)? {
            return Err(
                PartialVMError::new(StatusCode::DUPLICATE_MODULE_NAME).finish(Location::Undefined)
            );
        };

        // perform bytecode and loading verification
        self.loader.verify_module_verify_no_missing_dependencies(
            &compiled_module,
            data_store,
            logger,
        )?;

        data_store.publish_module(&module_id, module)
    }

    pub(crate) fn execute_script(
        &self,
        script: Vec<u8>,
        ty_args: Vec<TypeTag>,
        mut args: Vec<Value>,
        senders: Vec<AccountAddress>,
        data_store: &mut impl DataStore,
        cost_strategy: &mut CostStrategy,
        logger: &impl Logger,
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
        let (main, type_params) = self
            .loader
            .load_script(&script, &ty_args, data_store, logger)?;

        // Build the arguments list for the main and check the arguments are of restricted types.
        // Signers are built up from left-to-right. Either all signer arguments are used, or no
        // signer arguments can be be used by a script.
        let parameters = &main.parameters().0;
        let has_signer_parameters = parameters.get(0).map_or(false, is_signer_reference);
        let mut signers_and_args = if has_signer_parameters {
            if parameters.len() != args.len() + senders.len() {
                return Err(PartialVMError::new(StatusCode::TYPE_MISMATCH)
                    .with_message("Scripts must use all or no signers".to_string())
                    .finish(Location::Script));
            }
            // add signers to args
            senders
                .into_iter()
                .map(Value::transaction_argument_signer_reference)
                .collect()
        } else {
            // no signer parameters, don't add to args
            vec![]
        };
        signers_and_args.append(&mut args);
        check_args(&signers_and_args).map_err(|e| e.finish(Location::Script))?;

        // run the script
        Interpreter::entrypoint(
            main,
            type_params,
            signers_and_args,
            data_store,
            cost_strategy,
            &self.loader,
            logger,
        )
    }

    pub(crate) fn execute_function(
        &self,
        module: &ModuleId,
        function_name: &IdentStr,
        ty_args: Vec<TypeTag>,
        args: Vec<Value>,
        data_store: &mut impl DataStore,
        cost_strategy: &mut CostStrategy,
        logger: &impl Logger,
    ) -> VMResult<()> {
        // load the function in the given module, perform verification of the module and
        // its dependencies if the module was not loaded
        let (func, type_params) =
            self.loader
                .load_function(function_name, module, &ty_args, data_store, logger)?;

        // check the arguments provided are of restricted types
        check_args(&args).map_err(|e| e.finish(Location::Module(module.clone())))?;

        // run the function
        Interpreter::entrypoint(
            func,
            type_params,
            args,
            data_store,
            cost_strategy,
            &self.loader,
            logger,
        )
    }
}

// Check that the transaction arguments are acceptable by the VM.
// Constants and a reference to a `Signer` are the only arguments allowed.
// This check is more of a rough filter to remove obvious bad arguments.
fn check_args(args: &[Value]) -> PartialVMResult<()> {
    for val in args {
        if !val.is_constant_or_signer_ref() {
            return Err(PartialVMError::new(StatusCode::TYPE_MISMATCH)
                .with_message("VM argument types are restricted".to_string()));
        }
    }
    Ok(())
}
