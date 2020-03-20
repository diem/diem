// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    code_cache::{module_cache::VMModuleCache, script_cache::ScriptCache},
    interpreter::Interpreter,
    interpreter_context::InterpreterContext,
    loaded_data::{
        function::{FunctionRef, FunctionReference},
        loaded_module::LoadedModule,
    },
};
use bytecode_verifier::VerifiedModule;
use libra_logger::prelude::*;
use libra_types::{
    language_storage::ModuleId,
    vm_error::{StatusCode, VMStatus},
};
use move_core_types::identifier::{IdentStr, Identifier};
use move_vm_cache::Arena;
use move_vm_types::{
    loaded_data::types::{StructType, Type},
    values::Value,
};
use vm::{
    access::ModuleAccess,
    errors::{verification_error, vm_error, Location, VMResult},
    file_format::{FunctionHandleIndex, FunctionSignature, SignatureToken, StructDefinitionIndex},
    gas_schedule::CostTable,
    transaction_metadata::TransactionMetadata,
    CompiledModule, IndexKind,
};

/// An instantiation of the MoveVM.
/// `code_cache` is the top level module cache that holds loaded published modules.
/// `script_cache` is the cache that stores all the scripts that have previously been invoked.
/// `publishing_option` is the publishing option that is set. This can be one of either:
/// * Locked, with a whitelist of scripts that the VM is allowed to execute. For scripts that aren't
///   in the whitelist, the VM will just reject it in `verify_transaction`.
/// * Custom scripts, which will allow arbitrary valid scripts, but no module publishing
/// * Open script and module publishing
pub(crate) struct VMRuntime<'alloc> {
    code_cache: VMModuleCache<'alloc>,
    script_cache: ScriptCache<'alloc>,
}

impl<'alloc> VMRuntime<'alloc> {
    /// Create a new VM instance with an Arena allocator to store the modules and a `config` that
    /// contains the whitelist that this VM is allowed to execute.
    pub fn new(allocator: &'alloc Arena<LoadedModule>) -> Self {
        VMRuntime {
            code_cache: VMModuleCache::new(allocator),
            script_cache: ScriptCache::new(allocator),
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
                IndexKind::AddressPool,
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
        args: Vec<Value>,
    ) -> VMResult<()> {
        let main = self.script_cache.cache_script(&script, context)?;

        if !verify_actuals(main.signature(), &args) {
            return Err(VMStatus::new(StatusCode::TYPE_MISMATCH)
                .with_message("Actual Type Mismatch".to_string()));
        }

        Interpreter::entrypoint(context, self, txn_data, gas_schedule, main, args)
    }

    pub fn execute_function(
        &self,
        context: &mut dyn InterpreterContext,
        txn_data: &TransactionMetadata,
        gas_schedule: &CostTable,
        module: &ModuleId,
        function_name: &IdentStr,
        args: Vec<Value>,
    ) -> VMResult<()> {
        Interpreter::execute_function(
            context,
            self,
            txn_data,
            gas_schedule,
            module,
            function_name,
            args,
        )
    }

    pub fn cache_module(&self, module: VerifiedModule) {
        self.code_cache.cache_module(module);
    }

    pub fn resolve_struct_def_by_name(
        &self,
        module_id: &ModuleId,
        name: &Identifier,
        ty_args: &[Type],
        context: &mut dyn InterpreterContext,
    ) -> VMResult<StructType> {
        let module = self.code_cache.get_loaded_module(module_id, context)?;
        let struct_idx = module.get_struct_def_index(name)?;
        self.code_cache
            .resolve_struct_def(module, *struct_idx, ty_args, context)
    }

    pub fn resolve_struct_def(
        &self,
        module: &LoadedModule,
        idx: StructDefinitionIndex,
        ty_args: &[Type],
        data_view: &dyn InterpreterContext,
    ) -> VMResult<StructType> {
        self.code_cache
            .resolve_struct_def(module, idx, ty_args, data_view)
    }

    pub fn resolve_function_ref(
        &self,
        caller_module: &LoadedModule,
        idx: FunctionHandleIndex,
        data_view: &dyn InterpreterContext,
    ) -> VMResult<FunctionRef<'alloc>> {
        self.code_cache
            .resolve_function_ref(caller_module, idx, data_view)
    }

    pub fn resolve_signature_token(
        &self,
        module: &LoadedModule,
        tok: &SignatureToken,
        ty_args: &[Type],
        data_view: &dyn InterpreterContext,
    ) -> VMResult<Type> {
        self.code_cache
            .resolve_signature_token(module, tok, ty_args, data_view)
    }

    pub fn get_loaded_module(
        &self,
        id: &ModuleId,
        data_view: &dyn InterpreterContext,
    ) -> VMResult<&'alloc LoadedModule> {
        self.code_cache.get_loaded_module(id, data_view)
    }
}

/// Verify if the transaction arguments match the type signature of the main function.
fn verify_actuals(signature: &FunctionSignature, args: &[Value]) -> bool {
    if signature.arg_types.len() != args.len() {
        warn!(
            "[VM] different argument length: actuals {}, formals {}",
            args.len(),
            signature.arg_types.len()
        );
        return false;
    }
    for (ty, arg) in signature.arg_types.iter().zip(args.iter()) {
        if !arg.is_valid_script_arg(ty) {
            warn!(
                "[VM] different argument type: formal {:?}, actual {:?}",
                ty, arg
            );
            return false;
        }
    }
    true
}
