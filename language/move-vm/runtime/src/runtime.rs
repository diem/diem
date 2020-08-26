// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    data_cache::{RemoteCache, TransactionDataCache},
    interpreter::Interpreter,
    loader::Loader,
    session::Session,
};
use libra_logger::prelude::*;
use move_core_types::{
    account_address::AccountAddress,
    identifier::IdentStr,
    language_storage::{ModuleId, TypeTag},
    vm_status::StatusCode,
};
use move_vm_types::{
    data_store::DataStore, gas_schedule::CostStrategy, loaded_data::runtime_types::Type,
    values::Value,
};
use vm::{
    access::ModuleAccess,
    errors::{verification_error, Location, PartialVMError, VMResult},
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

    pub fn deserialize_args(
        &self,
        loc: Location,
        tys: &[Type],
        args: Vec<Vec<u8>>,
    ) -> VMResult<Vec<Value>> {
        // Check if the number of arguments matches the number of arguments.
        // REVIEW: the error code `TYPE_MISMATCH` seems a little bit weird here.
        if tys.len() != args.len() {
            return Err(PartialVMError::new(StatusCode::TYPE_MISMATCH)
                .with_message(format!(
                    "argument length mismatch: expected {} got {}",
                    tys.len(),
                    args.len()
                ))
                .finish(loc));
        }

        // Get the layouts and kind information if the parameter types.
        // This could fail under two circumstances:
        //   1) The loader is in an invalid state.
        //   2) The outer environment is trying call a Move function with an unsupported parameter type
        //      (e.g. a reference).
        // Both are considered invariant violations currently.
        let mut layouts = vec![];
        let mut kinfos = vec![];
        for ty in tys {
            match self.loader.type_to_type_layout(ty) {
                Ok(layout) => layouts.push(layout),
                Err(err) => {
                    error!("[VM] failed to get layout from type");
                    return Err(err.finish(Location::Undefined));
                }
            }

            match self.loader.type_to_kind_info(ty) {
                Ok(kinfo) => kinfos.push(kinfo),
                Err(err) => {
                    error!("[VM] failed to get kind info from type");
                    return Err(err.finish(Location::Undefined));
                }
            }
        }

        // Try to deserialize the arguments according to the layouts.
        let mut vals = vec![];
        for ((layout, kinfo), arg) in layouts.iter().zip(kinfos.iter()).zip(args.iter()) {
            match Value::simple_deserialize(arg, kinfo, layout) {
                Some(val) => vals.push(val),
                None => {
                    warn!("[VM] argument deserialization failed");
                    return Err(
                        PartialVMError::new(StatusCode::FAILED_TO_DESERIALIZE_RESOURCE)
                            .finish(Location::Script),
                    );
                }
            }
        }

        Ok(vals)
    }

    pub(crate) fn publish_module(
        &self,
        module: Vec<u8>,
        sender: AccountAddress,
        data_store: &mut impl DataStore,
        _cost_strategy: &mut CostStrategy,
    ) -> VMResult<()> {
        // deserialize the module. Perform bounds check. After this indexes can be
        // used with the `[]` operator
        let compiled_module = match CompiledModule::deserialize(&module) {
            Ok(module) => module,
            Err(err) => {
                warn!("[VM] module deserialization failed {:?}", err);
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
        self.loader
            .verify_module_verify_no_missing_dependencies(&compiled_module, data_store)?;

        data_store.publish_module(&module_id, module)
    }

    pub(crate) fn execute_script(
        &self,
        script: Vec<u8>,
        ty_args: Vec<TypeTag>,
        args: Vec<Vec<u8>>,
        senders: Vec<AccountAddress>,
        data_store: &mut impl DataStore,
        cost_strategy: &mut CostStrategy,
    ) -> VMResult<()> {
        // load the script, perform verification
        let (main, type_params) = self.loader.load_script(&script, &ty_args, data_store)?;

        // Build the arguments list for the main and check the arguments are of restricted types.
        // Signers are built up from left-to-right. Either all signer arguments are used, or no
        // signer arguments can be be used by a script.
        fn is_signer_reference(ty: &Type) -> bool {
            match ty {
                Type::Reference(ty) | Type::MutableReference(ty) => match &**ty {
                    Type::Signer => true,
                    _ => false,
                },
                _ => false,
            }
        }
        let first_param_is_signer_ref = type_params.get(0).map_or(false, is_signer_reference);
        let args = if first_param_is_signer_ref {
            let n_senders = senders.len();
            if type_params.len() != n_senders + args.len() {
                return Err(PartialVMError::new(StatusCode::TYPE_MISMATCH)
                    .with_message("Scripts must use all or no signers".to_string())
                    .finish(Location::Script));
            }
            if type_params[1..n_senders]
                .iter()
                .any(|ty| !is_signer_reference(ty))
            {
                return Err(PartialVMError::new(StatusCode::TYPE_MISMATCH)
                    .with_message("Some of the first arguments are not signers".to_string())
                    .finish(Location::Script));
            }
            let mut vals: Vec<Value> = senders
                .into_iter()
                .map(|addr| Value::transaction_argument_signer_reference(addr))
                .collect();
            vals.extend(self.deserialize_args(
                Location::Script,
                &type_params[n_senders..],
                args,
            )?);
            vals
        } else {
            self.deserialize_args(Location::Script, &type_params, args)?
        };

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
        args: Vec<Vec<u8>>,
        data_store: &mut impl DataStore,
        cost_strategy: &mut CostStrategy,
    ) -> VMResult<()> {
        // load the function in the given module, perform verification of the module and
        // its dependencies if the module was not loaded
        let (func, type_params) =
            self.loader
                .load_function(function_name, module, &ty_args, data_store)?;

        // check the arguments provided are of restricted types
        let args = self.deserialize_args(Location::Module(module.clone()), &type_params, args)?;

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
