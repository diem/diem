// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[cfg(any(test, feature = "instruction_synthesis"))]
use crate::chain_state::TransactionExecutionContext;
#[cfg(any(test, feature = "instruction_synthesis"))]
use crate::data_cache::RemoteCache;
use crate::{
    code_cache::module_cache::ModuleCache,
    counters::*,
    execution_context::InterpreterContext,
    gas,
    identifier::{create_access_path, resource_storage_key},
    loaded_data::{
        function::{FunctionRef, FunctionReference},
        loaded_module::LoadedModule,
    },
    system_module_names::{
        ACCOUNT_MODULE, ACCOUNT_STRUCT_NAME, CREATE_ACCOUNT_NAME, EMIT_EVENT_NAME,
        SAVE_ACCOUNT_NAME,
    },
};
use libra_logger::prelude::*;
use libra_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    byte_array::ByteArray,
    contract_event::ContractEvent,
    event::EventKey,
    identifier::IdentStr,
    language_storage::{ModuleId, StructTag, TypeTag},
    transaction::MAX_TRANSACTION_SIZE_IN_BYTES,
    vm_error::{StatusCode, StatusType, VMStatus},
};
#[cfg(any(test, feature = "instruction_synthesis"))]
use std::collections::HashMap;
use std::{collections::VecDeque, convert::TryFrom, marker::PhantomData};
#[cfg(any(test, feature = "instruction_synthesis"))]
use vm::gas_schedule::MAXIMUM_NUMBER_OF_GAS_UNITS;
use vm::{
    access::ModuleAccess,
    errors::*,
    file_format::{
        Bytecode, FunctionHandleIndex, LocalIndex, LocalsSignatureIndex, SignatureToken,
        StructDefinitionIndex,
    },
    gas_schedule::{
        calculate_intrinsic_gas, AbstractMemorySize, CostTable, GasAlgebra, GasCarrier,
        NativeCostIndex, Opcodes,
    },
    transaction_metadata::TransactionMetadata,
};
use vm_runtime_types::{
    loaded_data::struct_def::StructDef,
    native_functions::dispatch::resolve_native_function,
    value::{IntegerValue, Locals, ReferenceValue, Struct, Value},
};

fn derive_type_tag(
    module: &impl ModuleAccess,
    type_actual_tags: &[TypeTag],
    ty: &SignatureToken,
) -> VMResult<TypeTag> {
    use SignatureToken::*;

    match ty {
        Bool => Ok(TypeTag::Bool),
        Address => Ok(TypeTag::Address),
        U8 => Ok(TypeTag::U8),
        U64 => Ok(TypeTag::U64),
        U128 => Ok(TypeTag::U128),
        ByteArray => Ok(TypeTag::ByteArray),
        TypeParameter(idx) => type_actual_tags
            .get(*idx as usize)
            .ok_or_else(|| {
                VMStatus::new(StatusCode::VERIFIER_INVARIANT_VIOLATION).with_message(
                    "Cannot derive type tag: type parameter index out of bounds.".to_string(),
                )
            })
            .map(|inner| inner.clone()),
        Reference(_) | MutableReference(_) => {
            Err(VMStatus::new(StatusCode::VERIFIER_INVARIANT_VIOLATION)
                .with_message("Cannot derive type tag for references.".to_string()))
        }
        Struct(idx, struct_type_actuals) => {
            let struct_type_actuals_tags = struct_type_actuals
                .iter()
                .map(|ty| derive_type_tag(module, type_actual_tags, ty))
                .collect::<VMResult<Vec<_>>>()?;
            let struct_handle = module.struct_handle_at(*idx);
            let struct_name = module.identifier_at(struct_handle.name);
            let module_handle = module.module_handle_at(struct_handle.module);
            let module_address = module.address_at(module_handle.address);
            let module_name = module.identifier_at(module_handle.name);
            Ok(TypeTag::Struct(StructTag {
                address: *module_address,
                module: module_name.into(),
                name: struct_name.into(),
                type_params: struct_type_actuals_tags,
            }))
        }
    }
}

/// `Interpreter` instances can execute Move functions.
///
/// An `Interpreter` instance is a stand alone execution context for a function.
/// It mimics execution on a single thread, with an call stack and an operand stack.
/// The `Interpreter` receives a reference to a data store used by certain opcodes
/// to do operations on data on chain and a `TransactionMetadata` which is also used to resolve
/// specific opcodes.
/// A `ModuleCache` is also provided to resolve external references to code.
// REVIEW: abstract the data store better (maybe a single Trait for both data and event?)
// The ModuleCache should be a Loader with a proper API.
// Resolve where GasMeter should live.
pub struct Interpreter<'alloc, 'txn, P>
where
    'alloc: 'txn,
    P: ModuleCache<'alloc>,
{
    /// Operand stack, where Move `Value`s are stored for stack operations.
    operand_stack: Stack,
    /// The stack of active functions.
    call_stack: CallStack<'txn>,
    /// Transaction data to resolve special bytecodes (e.g. GetTxnSequenceNumber, GetTxnPublicKey,
    /// GetTxnSenderAddress, ...)
    txn_data: &'txn TransactionMetadata,
    /// Code cache, this is effectively the loader.
    module_cache: &'txn P,
    gas_schedule: &'txn CostTable,
    phantom: PhantomData<&'alloc ()>,
}

impl<'alloc, 'txn, P> Interpreter<'alloc, 'txn, P>
where
    'alloc: 'txn,
    P: ModuleCache<'alloc>,
{
    /// Execute a function.
    /// `module` is an identifier for the name the module is stored in. `function_name` is the name
    /// of the function. If such function is found, the VM will execute this function with arguments
    /// `args`. The return value will be placed on the top of the value stack and abort if an error
    /// occurs.
    // REVIEW: this should probably disappear or at the very least only one between
    // `execute_function` and `entrypoint` should exist. It's a bit messy at
    // the moment given tooling and testing. Once we remove Program transactions and we
    // clean up the loader we will have a better time cleaning this up.
    pub fn execute_function(
        context: &mut dyn InterpreterContext,
        module_cache: &'txn P,
        txn_data: &'txn TransactionMetadata,
        gas_schedule: &'txn CostTable,
        module: &ModuleId,
        function_name: &IdentStr,
        args: Vec<Value>,
    ) -> VMResult<()> {
        let mut interp = Self::new(module_cache, txn_data, gas_schedule);
        let loaded_module = interp.module_cache.get_loaded_module(module)?;
        let func_idx = loaded_module
            .function_defs_table
            .get(function_name)
            .ok_or_else(|| VMStatus::new(StatusCode::LINKER_ERROR))?;
        let func = FunctionRef::new(loaded_module, *func_idx);

        interp.execute(context, func, args)
    }

    /// Entrypoint into the interpreter. All external calls need to be routed through this
    /// function.
    pub(crate) fn entrypoint(
        context: &mut dyn InterpreterContext,
        module_cache: &'txn P,
        txn_data: &'txn TransactionMetadata,
        gas_schedule: &'txn CostTable,
        func: FunctionRef<'txn>,
        args: Vec<Value>,
    ) -> VMResult<()> {
        // We charge an intrinsic amount of gas based upon the size of the transaction submitted
        // (in raw bytes).
        let txn_size = txn_data.transaction_size();
        // The callers of this function verify the transaction before executing it. Transaction
        // verification ensures the following condition.
        assume!(txn_size.get() <= (MAX_TRANSACTION_SIZE_IN_BYTES as u64));
        // We count the intrinsic cost of the transaction here, since that needs to also cover the
        // setup of the function.
        let mut interp = Self::new(module_cache, txn_data, gas_schedule);
        let starting_gas = context.remaining_gas();
        gas!(consume: context, calculate_intrinsic_gas(txn_size))?;
        let ret = interp.execute(context, func, args);
        record_stats!(
            observe | TXN_EXECUTION_GAS_USAGE | starting_gas.sub(context.remaining_gas()).get()
        );
        ret
    }

    /// Create an account on the blockchain by calling into `CREATE_ACCOUNT_NAME` function stored
    /// in the `ACCOUNT_MODULE` on chain.
    // REVIEW: this should not live here
    pub(crate) fn create_account_entry(
        context: &mut dyn InterpreterContext,
        module_cache: &'txn P,
        txn_data: &'txn TransactionMetadata,
        gas_schedule: &'txn CostTable,
        addr: AccountAddress,
    ) -> VMResult<()> {
        let account_module = module_cache.get_loaded_module(&ACCOUNT_MODULE)?;
        let mut interp = Self::new(module_cache, txn_data, gas_schedule);
        interp.execute_function_call(
            context,
            &ACCOUNT_MODULE,
            &CREATE_ACCOUNT_NAME,
            vec![Value::address(addr)],
        )?;

        let account_resource = interp.operand_stack.pop_as::<Struct>()?;
        interp.save_account(context, account_module, addr, account_resource)
    }

    /// Create a new instance of an `Interpreter` in the context of a transaction with a
    /// given module cache and gas schedule.
    fn new(
        module_cache: &'txn P,
        txn_data: &'txn TransactionMetadata,
        gas_schedule: &'txn CostTable,
    ) -> Self {
        Interpreter {
            operand_stack: Stack::new(),
            call_stack: CallStack::new(),
            gas_schedule,
            txn_data,
            module_cache,
            phantom: PhantomData,
        }
    }

    /// Execute a function.
    /// `module` is an identifier for the name the module is stored in. `function_name` is the name
    /// of the function. If such function is found, the VM will execute this function with arguments
    /// `args`. The return value will be placed on the top of the value stack and abort if an error
    /// occurs.
    // REVIEW: this should probably disappear or at the very least only one between
    // `execute_function` and `entrypoint` should exist. It's a bit messy at
    // the moment given tooling and testing. Once we remove Program transactions and we
    // clean up the loader we will have a better time cleaning this up.
    fn execute_function_call(
        &mut self,
        context: &mut dyn InterpreterContext,
        module: &ModuleId,
        function_name: &IdentStr,
        args: Vec<Value>,
    ) -> VMResult<()> {
        let loaded_module = self.module_cache.get_loaded_module(module)?;
        let func_idx = loaded_module
            .function_defs_table
            .get(function_name)
            .ok_or_else(|| VMStatus::new(StatusCode::LINKER_ERROR))?;
        let func = FunctionRef::new(loaded_module, *func_idx);

        self.execute(context, func, args)
    }

    /// Internal execution entry point.
    fn execute(
        &mut self,
        context: &mut dyn InterpreterContext,
        function: FunctionRef<'txn>,
        args: Vec<Value>,
    ) -> VMResult<()> {
        // No unwinding of the call stack and value stack need to be done here -- the context will
        // take care of that.
        self.execute_main(context, function, args, 0)
    }

    /// Main loop for the execution of a function.
    ///
    /// This function sets up a `Frame` and calls `execute_code_unit` to execute code of the
    /// function represented by the frame. Control comes back to this function on return or
    /// on call. When that happens the frame is changes to a new one (call) or to the one
    /// at the top of the stack (return). If the call stack is empty execution is completed.
    // REVIEW: create account will be removed in favor of a native function (no opcode) and
    // we can simplify this code quite a bit.
    fn execute_main(
        &mut self,
        context: &mut dyn InterpreterContext,
        function: FunctionRef<'txn>,
        args: Vec<Value>,
        create_account_marker: usize,
    ) -> VMResult<()> {
        let mut locals = Locals::new(function.local_count());
        // TODO: assert consistency of args and function formals
        for (i, value) in args.into_iter().enumerate() {
            locals.store_loc(i, value)?;
        }
        let mut current_frame = Frame::new(function, vec![], locals);
        loop {
            let code = current_frame.code_definition();
            let exit_code = self
                .execute_code_unit(context, &mut current_frame, code)
                .or_else(|err| Err(self.maybe_core_dump(err, &current_frame)))?;
            match exit_code {
                ExitCode::Return => {
                    // TODO: assert consistency of current frame: stack height correct
                    if create_account_marker == self.call_stack.0.len() {
                        return Ok(());
                    }
                    if let Some(frame) = self.call_stack.pop() {
                        current_frame = frame;
                    } else {
                        return Err(self.unreachable("call stack cannot be empty", &current_frame));
                    }
                }
                ExitCode::Call(idx, type_actuals_idx) => {
                    let type_actuals = &current_frame
                        .module()
                        .locals_signature_at(type_actuals_idx)
                        .0;
                    gas!(
                        instr: context,
                        self,
                        Opcodes::CALL,
                        AbstractMemorySize::new((type_actuals.len() + 1) as GasCarrier)
                    )?;
                    let type_actual_tags = type_actuals
                        .iter()
                        .map(|ty| {
                            derive_type_tag(
                                current_frame.module(),
                                current_frame.type_actual_tags(),
                                ty,
                            )
                        })
                        .collect::<VMResult<Vec<_>>>()?;

                    let opt_frame = self
                        .make_call_frame(context, current_frame.module(), idx, type_actual_tags)
                        .or_else(|err| Err(self.maybe_core_dump(err, &current_frame)))?;
                    if let Some(frame) = opt_frame {
                        self.call_stack.push(current_frame).or_else(|frame| {
                            let err = VMStatus::new(StatusCode::CALL_STACK_OVERFLOW);
                            Err(self.maybe_core_dump(err, &frame))
                        })?;
                        current_frame = frame;
                    }
                }
            }
        }
    }

    /// Execute a Move function until a return or a call opcode is found.
    fn execute_code_unit(
        &mut self,
        context: &mut dyn InterpreterContext,
        frame: &mut Frame<'txn, FunctionRef<'txn>>,
        code: &[Bytecode],
    ) -> VMResult<ExitCode> {
        // TODO: re-enbale this once gas metering is sorted out
        //let code = frame.code_definition();
        loop {
            for instruction in &code[frame.pc as usize..] {
                frame.pc += 1;

                match instruction {
                    Bytecode::Pop => {
                        gas!(const_instr: context, self, Opcodes::POP)?;
                        self.operand_stack.pop()?;
                    }
                    Bytecode::Ret => {
                        gas!(const_instr: context, self, Opcodes::RET)?;
                        return Ok(ExitCode::Return);
                    }
                    Bytecode::BrTrue(offset) => {
                        gas!(const_instr: context, self, Opcodes::BR_TRUE)?;
                        if self.operand_stack.pop_as::<bool>()? {
                            frame.pc = *offset;
                            break;
                        }
                    }
                    Bytecode::BrFalse(offset) => {
                        gas!(const_instr: context, self, Opcodes::BR_FALSE)?;
                        if !self.operand_stack.pop_as::<bool>()? {
                            frame.pc = *offset;
                            break;
                        }
                    }
                    Bytecode::Branch(offset) => {
                        gas!(const_instr: context, self, Opcodes::BRANCH)?;
                        frame.pc = *offset;
                        break;
                    }
                    Bytecode::LdU8(int_const) => {
                        gas!(const_instr: context, self, Opcodes::LD_U8)?;
                        self.operand_stack.push(Value::u8(*int_const))?;
                    }
                    Bytecode::LdU64(int_const) => {
                        gas!(const_instr: context, self, Opcodes::LD_U64)?;
                        self.operand_stack.push(Value::u64(*int_const))?;
                    }
                    Bytecode::LdU128(int_const) => {
                        gas!(const_instr: context, self, Opcodes::LD_U128)?;
                        self.operand_stack.push(Value::u128(*int_const))?;
                    }
                    Bytecode::LdAddr(idx) => {
                        gas!(const_instr: context, self, Opcodes::LD_ADDR)?;
                        self.operand_stack
                            .push(Value::address(*frame.module().address_at(*idx)))?;
                    }
                    Bytecode::LdByteArray(idx) => {
                        let byte_array = frame.module().byte_array_at(*idx);
                        gas!(
                            instr: context,
                            self,
                            Opcodes::LD_BYTEARRAY,
                            AbstractMemorySize::new(byte_array.len() as GasCarrier)
                        )?;
                        self.operand_stack
                            .push(Value::byte_array(byte_array.clone()))?;
                    }
                    Bytecode::LdTrue => {
                        gas!(const_instr: context, self, Opcodes::LD_TRUE)?;
                        self.operand_stack.push(Value::bool(true))?;
                    }
                    Bytecode::LdFalse => {
                        gas!(const_instr: context, self, Opcodes::LD_TRUE)?;
                        self.operand_stack.push(Value::bool(false))?;
                    }
                    Bytecode::CopyLoc(idx) => {
                        let local = frame.copy_loc(*idx)?;
                        gas!(instr: context, self, Opcodes::COPY_LOC, local.size())?;
                        self.operand_stack.push(local)?;
                    }
                    Bytecode::MoveLoc(idx) => {
                        let local = frame.move_loc(*idx)?;
                        gas!(instr: context, self, Opcodes::MOVE_LOC, local.size())?;
                        self.operand_stack.push(local)?;
                    }
                    Bytecode::StLoc(idx) => {
                        let value_to_store = self.operand_stack.pop()?;
                        gas!(instr: context, self, Opcodes::ST_LOC, value_to_store.size())?;
                        frame.store_loc(*idx, value_to_store)?;
                    }
                    Bytecode::Call(idx, type_actuals_idx) => {
                        return Ok(ExitCode::Call(*idx, *type_actuals_idx));
                    }
                    Bytecode::MutBorrowLoc(idx) | Bytecode::ImmBorrowLoc(idx) => {
                        let opcode = match instruction {
                            Bytecode::MutBorrowLoc(_) => Opcodes::MUT_BORROW_LOC,
                            _ => Opcodes::IMM_BORROW_LOC,
                        };
                        gas!(const_instr: context, self, opcode)?;
                        self.operand_stack.push(frame.borrow_loc(*idx)?)?;
                    }
                    Bytecode::ImmBorrowField(fd_idx) | Bytecode::MutBorrowField(fd_idx) => {
                        let opcode = match instruction {
                            Bytecode::MutBorrowField(_) => Opcodes::MUT_BORROW_FIELD,
                            _ => Opcodes::IMM_BORROW_FIELD,
                        };
                        gas!(const_instr: context, self, opcode)?;
                        let field_offset = frame.module().get_field_offset(*fd_idx)?;
                        let reference = self.operand_stack.pop_as::<ReferenceValue>()?;
                        let field_ref = reference.borrow_field(field_offset as usize)?;
                        self.operand_stack.push(field_ref)?;
                    }
                    Bytecode::Pack(sd_idx, _) => {
                        let struct_def = frame.module().struct_def_at(*sd_idx);
                        let field_count = struct_def.declared_field_count()?;
                        let args = self.operand_stack.popn(field_count)?;
                        let size = args.iter().fold(
                            AbstractMemorySize::new(GasCarrier::from(field_count)),
                            |acc, arg| acc.add(arg.size()),
                        );
                        gas!(instr: context, self, Opcodes::PACK, size)?;
                        self.operand_stack.push(Value::struct_(Struct::new(args)))?;
                    }
                    Bytecode::Unpack(sd_idx, _) => {
                        let struct_def = frame.module().struct_def_at(*sd_idx);
                        let field_count = struct_def.declared_field_count()?;
                        let struct_ = self.operand_stack.pop_as::<Struct>()?;
                        gas!(
                            instr: context,
                            self,
                            Opcodes::UNPACK,
                            AbstractMemorySize::new(GasCarrier::from(field_count))
                        )?;
                        // TODO: Whether or not we want this gas metering in the loop is
                        // questionable.  However, if we don't have it in the loop we could wind up
                        // doing a fair bit of work before charging for it.
                        for idx in 0..field_count {
                            let value = struct_.get_field_value(idx as usize)?;
                            gas!(instr: context, self, Opcodes::UNPACK, value.size())?;
                            self.operand_stack.push(value)?;
                        }
                    }
                    Bytecode::ReadRef => {
                        let reference = self.operand_stack.pop_as::<ReferenceValue>()?;
                        let value = reference.read_ref()?;
                        gas!(instr: context, self, Opcodes::READ_REF, value.size())?;
                        self.operand_stack.push(value)?;
                    }
                    Bytecode::WriteRef => {
                        let reference = self.operand_stack.pop_as::<ReferenceValue>()?;
                        let value = self.operand_stack.pop()?;
                        gas!(instr: context, self, Opcodes::WRITE_REF, value.size())?;
                        reference.write_ref(value);
                    }
                    Bytecode::CastU8 => {
                        gas!(const_instr: context, self, Opcodes::CAST_U8)?;
                        let integer_value = self.operand_stack.pop_as::<IntegerValue>()?;
                        self.operand_stack.push(Value::u8(integer_value.into()))?;
                    }
                    Bytecode::CastU64 => {
                        gas!(const_instr: context, self, Opcodes::CAST_U64)?;
                        let integer_value = self.operand_stack.pop_as::<IntegerValue>()?;
                        self.operand_stack.push(Value::u64(integer_value.into()))?;
                    }
                    Bytecode::CastU128 => {
                        gas!(const_instr: context, self, Opcodes::CAST_U128)?;
                        let integer_value = self.operand_stack.pop_as::<IntegerValue>()?;
                        self.operand_stack.push(Value::u128(integer_value.into()))?;
                    }
                    // Arithmetic Operations
                    Bytecode::Add => {
                        gas!(const_instr: context, self, Opcodes::ADD)?;
                        self.binop_int(IntegerValue::add_checked)?
                    }
                    Bytecode::Sub => {
                        gas!(const_instr: context, self, Opcodes::SUB)?;
                        self.binop_int(IntegerValue::sub_checked)?
                    }
                    Bytecode::Mul => {
                        gas!(const_instr: context, self, Opcodes::MUL)?;
                        self.binop_int(IntegerValue::mul_checked)?
                    }
                    Bytecode::Mod => {
                        gas!(const_instr: context, self, Opcodes::MOD)?;
                        self.binop_int(IntegerValue::rem_checked)?
                    }
                    Bytecode::Div => {
                        gas!(const_instr: context, self, Opcodes::DIV)?;
                        self.binop_int(IntegerValue::div_checked)?
                    }
                    Bytecode::BitOr => {
                        gas!(const_instr: context, self, Opcodes::BIT_OR)?;
                        self.binop_int(IntegerValue::bit_or)?
                    }
                    Bytecode::BitAnd => {
                        gas!(const_instr: context, self, Opcodes::BIT_AND)?;
                        self.binop_int(IntegerValue::bit_and)?
                    }
                    Bytecode::Xor => {
                        gas!(const_instr: context, self, Opcodes::XOR)?;
                        self.binop_int(IntegerValue::bit_xor)?
                    }
                    Bytecode::Shl => {
                        gas!(const_instr: context, self, Opcodes::SHL)?;
                        self.binop_int(IntegerValue::shl_checked)?
                    }
                    Bytecode::Shr => {
                        gas!(const_instr: context, self, Opcodes::SHR)?;
                        self.binop_int(IntegerValue::shr_checked)?
                    }
                    Bytecode::Or => {
                        gas!(const_instr: context, self, Opcodes::OR)?;
                        self.binop_bool(|l, r| l || r)?
                    }
                    Bytecode::And => {
                        gas!(const_instr: context, self, Opcodes::AND)?;
                        self.binop_bool(|l, r| l && r)?
                    }
                    Bytecode::Lt => {
                        gas!(const_instr: context, self, Opcodes::LT)?;
                        self.binop_bool(|l: u64, r| l < r)?
                    }
                    Bytecode::Gt => {
                        gas!(const_instr: context, self, Opcodes::GT)?;
                        self.binop_bool(|l: u64, r| l > r)?
                    }
                    Bytecode::Le => {
                        gas!(const_instr: context, self, Opcodes::LE)?;
                        self.binop_bool(|l: u64, r| l <= r)?
                    }
                    Bytecode::Ge => {
                        gas!(const_instr: context, self, Opcodes::GE)?;
                        self.binop_bool(|l: u64, r| l >= r)?
                    }
                    Bytecode::Abort => {
                        gas!(const_instr: context, self, Opcodes::ABORT)?;
                        let error_code = self.operand_stack.pop_as::<u64>()?;
                        return Err(VMStatus::new(StatusCode::ABORTED).with_sub_status(error_code));
                    }
                    Bytecode::Eq => {
                        let lhs = self.operand_stack.pop()?;
                        let rhs = self.operand_stack.pop()?;
                        gas!(
                            instr: context,
                            self,
                            Opcodes::EQ,
                            lhs.size().add(rhs.size())
                        )?;
                        self.operand_stack.push(Value::bool(lhs.equals(&rhs)?))?;
                    }
                    Bytecode::Neq => {
                        let lhs = self.operand_stack.pop()?;
                        let rhs = self.operand_stack.pop()?;
                        gas!(
                            instr: context,
                            self,
                            Opcodes::NEQ,
                            lhs.size().add(rhs.size())
                        )?;
                        self.operand_stack
                            .push(Value::bool(lhs.not_equals(&rhs)?))?;
                    }
                    Bytecode::GetTxnSenderAddress => {
                        gas!(const_instr: context, self, Opcodes::GET_TXN_SENDER)?;
                        self.operand_stack
                            .push(Value::address(self.txn_data.sender()))?;
                    }
                    Bytecode::MutBorrowGlobal(idx, _) | Bytecode::ImmBorrowGlobal(idx, _) => {
                        let addr = self.operand_stack.pop_as::<AccountAddress>()?;
                        let size = self.global_data_op(
                            context,
                            addr,
                            *idx,
                            frame.module(),
                            Self::borrow_global,
                        )?;
                        gas!(instr: context, self, Opcodes::MUT_BORROW_GLOBAL, size)?;
                    }
                    Bytecode::Exists(idx, _) => {
                        let addr = self.operand_stack.pop_as::<AccountAddress>()?;
                        let size =
                            self.global_data_op(context, addr, *idx, frame.module(), Self::exists)?;
                        gas!(instr: context, self, Opcodes::EXISTS, size)?;
                    }
                    Bytecode::MoveFrom(idx, _) => {
                        let addr = self.operand_stack.pop_as::<AccountAddress>()?;
                        let size = self.global_data_op(
                            context,
                            addr,
                            *idx,
                            frame.module(),
                            Self::move_from,
                        )?;
                        // TODO: Have this calculate before pulling in the data based upon
                        // the size of the data that we are about to read in.
                        gas!(instr: context, self, Opcodes::MOVE_FROM, size)?;
                    }
                    Bytecode::MoveToSender(idx, _) => {
                        let addr = self.txn_data.sender();
                        let size = self.global_data_op(
                            context,
                            addr,
                            *idx,
                            frame.module(),
                            Self::move_to_sender,
                        )?;
                        gas!(instr: context, self, Opcodes::MOVE_TO, size)?;
                    }
                    Bytecode::FreezeRef => {
                        // FreezeRef should just be a null op as we don't distinguish between mut
                        // and immut ref at runtime.
                    }
                    Bytecode::Not => {
                        gas!(const_instr: context, self, Opcodes::NOT)?;
                        let value = !self.operand_stack.pop_as::<bool>()?;
                        self.operand_stack.push(Value::bool(value))?;
                    }
                    Bytecode::GetGasRemaining
                    | Bytecode::GetTxnPublicKey
                    | Bytecode::GetTxnSequenceNumber
                    | Bytecode::GetTxnMaxGasUnits
                    | Bytecode::GetTxnGasUnitPrice => {
                        return Err(VMStatus::new(StatusCode::VERIFIER_INVARIANT_VIOLATION)
                            .with_message(
                                "This opcode is deprecated and will be removed soon".to_string(),
                            ));
                    }
                }
            }
            // ok we are out, it's a branch, check the pc for good luck
            // TODO: re-work the logic here. Cost synthesis and tests should have a more
            // natural way to plug in
            if frame.pc as usize >= code.len() {
                if cfg!(test) || cfg!(feature = "instruction_synthesis") {
                    // In order to test the behavior of an instruction stream, hitting end of the
                    // code should report no error so that we can check the
                    // locals.
                    return Ok(ExitCode::Return);
                } else {
                    return Err(VMStatus::new(StatusCode::PC_OVERFLOW));
                }
            }
        }
    }

    /// Returns a `Frame` if the call is to a Move function. Calls to native functions are
    /// "inlined" and this returns `None`.
    ///
    /// Native functions do not push a frame at the moment and as such errors from a native
    /// function are incorrectly attributed to the caller.
    fn make_call_frame(
        &mut self,
        context: &mut dyn InterpreterContext,
        module: &LoadedModule,
        idx: FunctionHandleIndex,
        type_actual_tags: Vec<TypeTag>,
    ) -> VMResult<Option<Frame<'txn, FunctionRef<'txn>>>> {
        let func = self.module_cache.resolve_function_ref(module, idx)?;
        if func.is_native() {
            self.call_native(context, func, type_actual_tags)?;
            Ok(None)
        } else {
            let mut locals = Locals::new(func.local_count());
            let arg_count = func.arg_count();
            for i in 0..arg_count {
                locals.store_loc(arg_count - i - 1, self.operand_stack.pop()?)?;
            }
            Ok(Some(Frame::new(func, type_actual_tags, locals)))
        }
    }

    /// Call a native functions.
    fn call_native(
        &mut self,
        context: &mut dyn InterpreterContext,
        function: FunctionRef<'txn>,
        type_actual_tags: Vec<TypeTag>,
    ) -> VMResult<()> {
        let module = function.module();
        let module_id = module.self_id();
        let function_name = function.name();
        let native_function = resolve_native_function(&module_id, function_name)
            .ok_or_else(|| VMStatus::new(StatusCode::LINKER_ERROR))?;
        if module_id == *ACCOUNT_MODULE && function_name == EMIT_EVENT_NAME.as_ident_str() {
            self.call_emit_event(context, type_actual_tags)
        } else if module_id == *ACCOUNT_MODULE && function_name == SAVE_ACCOUNT_NAME.as_ident_str()
        {
            self.call_save_account(context)
        } else {
            let mut arguments = VecDeque::new();
            let expected_args = native_function.num_args();
            // REVIEW: this is checked again in every functions, rationalize it!
            if function.arg_count() != expected_args {
                // Should not be possible due to bytecode verifier but this
                // assertion is here to make sure
                // the view the type checker had lines up with the
                // execution of the native function
                return Err(VMStatus::new(StatusCode::LINKER_ERROR));
            }
            for _ in 0..expected_args {
                arguments.push_front(self.operand_stack.pop()?);
            }
            let result = (native_function.dispatch)(arguments, self.gas_schedule)?;
            gas!(consume: context, result.cost)?;
            result.result.and_then(|values| {
                for value in values {
                    self.operand_stack.push(value)?;
                }
                Ok(())
            })
        }
    }

    /// Emit an event if the native function was `write_to_event_store`.
    fn call_emit_event(
        &mut self,
        context: &mut dyn InterpreterContext,
        mut type_actual_tags: Vec<TypeTag>,
    ) -> VMResult<()> {
        if type_actual_tags.len() != 1 {
            return Err(
                VMStatus::new(StatusCode::VERIFIER_INVARIANT_VIOLATION).with_message(format!(
                    "write_to_event_storage expects 1 argument got {}.",
                    type_actual_tags.len()
                )),
            );
        }
        let type_tag = type_actual_tags.pop().unwrap();

        let msg = self
            .operand_stack
            .pop()?
            .simple_serialize()
            .ok_or_else(|| VMStatus::new(StatusCode::DATA_FORMAT_ERROR))?;
        let count = self.operand_stack.pop_as::<u64>()?;
        let key = self.operand_stack.pop_as::<ByteArray>()?;
        let guid = EventKey::try_from(key.as_bytes())
            .map_err(|_| VMStatus::new(StatusCode::EVENT_KEY_MISMATCH))?;
        context.push_event(ContractEvent::new(guid, count, type_tag, msg));
        Ok(())
    }

    /// Save an account into the data store.
    fn call_save_account(&mut self, context: &mut dyn InterpreterContext) -> VMResult<()> {
        let account_module = self.module_cache.get_loaded_module(&ACCOUNT_MODULE)?;
        let account_resource = self.operand_stack.pop_as::<Struct>()?;
        let address = self.operand_stack.pop_as::<AccountAddress>()?;
        self.save_account(context, account_module, address, account_resource)
    }

    /// Perform a binary operation to two values at the top of the stack.
    fn binop<F, T>(&mut self, f: F) -> VMResult<()>
    where
        VMResult<T>: From<Value>,
        F: FnOnce(T, T) -> VMResult<Value>,
    {
        let rhs = self.operand_stack.pop_as::<T>()?;
        let lhs = self.operand_stack.pop_as::<T>()?;
        let result = f(lhs, rhs)?;
        self.operand_stack.push(result)
    }

    /// Perform a binary operation for integer values.
    fn binop_int<F>(&mut self, f: F) -> VMResult<()>
    where
        F: FnOnce(IntegerValue, IntegerValue) -> VMResult<IntegerValue>,
    {
        self.binop(|lhs, rhs| {
            Ok(match f(lhs, rhs)? {
                IntegerValue::U8(x) => Value::u8(x),
                IntegerValue::U64(x) => Value::u64(x),
                IntegerValue::U128(x) => Value::u128(x),
            })
        })
    }

    /// Perform a binary operation for boolean values.
    fn binop_bool<F, T>(&mut self, f: F) -> VMResult<()>
    where
        VMResult<T>: From<Value>,
        F: FnOnce(T, T) -> bool,
    {
        self.binop(|lhs, rhs| Ok(Value::bool(f(lhs, rhs))))
    }

    /// Entry point for all global store operations (effectively opcodes).
    ///
    /// This performs common operation on the data store and then executes the specific
    /// opcode.
    fn global_data_op<F>(
        &mut self,
        context: &mut dyn InterpreterContext,
        address: AccountAddress,
        idx: StructDefinitionIndex,
        module: &LoadedModule,
        op: F,
    ) -> VMResult<AbstractMemorySize<GasCarrier>>
    where
        F: FnOnce(
            &mut Self,
            &mut dyn InterpreterContext,
            AccessPath,
            StructDef,
        ) -> VMResult<AbstractMemorySize<GasCarrier>>,
    {
        let ap = Self::make_access_path(module, idx, address);
        let struct_def = self.module_cache.resolve_struct_def(module, idx, context)?;
        op(self, context, ap, struct_def)
    }

    /// BorrowGlobal (mutable and not) opcode.
    fn borrow_global(
        &mut self,
        context: &mut dyn InterpreterContext,
        ap: AccessPath,
        struct_def: StructDef,
    ) -> VMResult<AbstractMemorySize<GasCarrier>> {
        let global_ref = context.borrow_global(&ap, struct_def)?;
        let size = global_ref.size();
        self.operand_stack.push(Value::global_ref(global_ref))?;
        Ok(size)
    }

    /// Exists opcode.
    fn exists(
        &mut self,
        context: &mut dyn InterpreterContext,
        ap: AccessPath,
        struct_def: StructDef,
    ) -> VMResult<AbstractMemorySize<GasCarrier>> {
        let (exists, mem_size) = context.resource_exists(&ap, struct_def)?;
        self.operand_stack.push(Value::bool(exists))?;
        Ok(mem_size)
    }

    /// MoveFrom opcode.
    fn move_from(
        &mut self,
        context: &mut dyn InterpreterContext,
        ap: AccessPath,
        struct_def: StructDef,
    ) -> VMResult<AbstractMemorySize<GasCarrier>> {
        let resource = context.move_resource_from(&ap, struct_def)?;
        let size = resource.size();
        self.operand_stack.push(resource)?;
        Ok(size)
    }

    /// MoveToSender opcode.
    fn move_to_sender(
        &mut self,
        context: &mut dyn InterpreterContext,
        ap: AccessPath,
        struct_def: StructDef,
    ) -> VMResult<AbstractMemorySize<GasCarrier>> {
        let resource = self.operand_stack.pop_as::<Struct>()?;
        let size = resource.size();
        context.move_resource_to(&ap, struct_def, resource)?;
        Ok(size)
    }

    /// Helper to create a resource storage key (`AccessPath`) for global storage operations.
    fn make_access_path(
        module: &impl ModuleAccess,
        idx: StructDefinitionIndex,
        address: AccountAddress,
    ) -> AccessPath {
        let struct_tag = resource_storage_key(module, idx);
        create_access_path(&address, struct_tag)
    }

    fn save_account(
        &mut self,
        context: &mut dyn InterpreterContext,
        account_module: &LoadedModule,
        addr: AccountAddress,
        account_resource: Struct,
    ) -> VMResult<()> {
        gas!(
            consume: context,
            self.gas_schedule
                .native_cost(NativeCostIndex::SAVE_ACCOUNT)
                .total()
        )?;
        let account_struct_id = account_module
            .struct_defs_table
            .get(&*ACCOUNT_STRUCT_NAME)
            .ok_or_else(|| VMStatus::new(StatusCode::LINKER_ERROR))?;
        let account_struct_def =
            self.module_cache
                .resolve_struct_def(account_module, *account_struct_id, context)?;

        // TODO: Adding the freshly created account's expiration date to the TransactionOutput here.
        let account_path = Self::make_access_path(account_module, *account_struct_id, addr);
        context.move_resource_to(&account_path, account_struct_def, account_resource)
    }

    //
    // Debugging and logging helpers.
    //

    /// Given an `VMStatus` generate a core dump if the error is an `InvariantViolation`.
    fn maybe_core_dump(
        &self,
        err: VMStatus,
        current_frame: &Frame<'txn, FunctionRef<'txn>>,
    ) -> VMStatus {
        if err.is(StatusType::InvariantViolation) {
            crit!(
                "Error: {:?}\nCORE DUMP: >>>>>>>>>>>>\n{}\n<<<<<<<<<<<<\n",
                err,
                self.get_internal_state(current_frame),
            );
        }
        err
    }

    /// Generate a string which is the status of the interpreter: call stack, current bytecode
    /// stream, locals and operand stack.
    ///
    /// It is used when generating a core dump but can be used for debugging of the interpreter.
    /// It will be exposed via a debug module to give developers a way to print the internals
    /// of an execution.
    fn get_internal_state(&self, current_frame: &Frame<'txn, FunctionRef<'txn>>) -> String {
        let mut internal_state = "Call stack:\n".to_string();
        for (i, frame) in self.call_stack.0.iter().enumerate() {
            internal_state.push_str(
                format!(
                    " frame #{}: {} [pc = {}]\n",
                    i,
                    frame.function.pretty_string(),
                    frame.pc,
                )
                .as_str(),
            );
        }
        internal_state.push_str(
            format!(
                "*frame #{}: {} [pc = {}]:\n",
                self.call_stack.0.len(),
                current_frame.function.pretty_string(),
                current_frame.pc,
            )
            .as_str(),
        );
        let code = current_frame.code_definition();
        let pc = current_frame.pc as usize;
        if pc < code.len() {
            let mut i = 0;
            for bytecode in &code[0..pc] {
                internal_state.push_str(format!("{}> {:?}\n", i, bytecode).as_str());
                i += 1;
            }
            internal_state.push_str(format!("{}* {:?}\n", i, code[pc]).as_str());
        }
        internal_state
            .push_str(format!("Locals:\n{}", current_frame.locals.pretty_string()).as_str());
        internal_state.push_str("Operand Stack:\n");
        for value in &self.operand_stack.0 {
            internal_state.push_str(format!("{}\n", value.pretty_string()).as_str());
        }
        internal_state
    }

    /// Generate a core dump and an `UNREACHABLE` invariant violation.
    fn unreachable(&self, msg: &str, current_frame: &Frame<'txn, FunctionRef<'txn>>) -> VMStatus {
        let err = VMStatus::new(StatusCode::UNREACHABLE).with_message(msg.to_string());
        self.maybe_core_dump(err, current_frame)
    }
}

// TODO Determine stack size limits based on gas limit
const OPERAND_STACK_SIZE_LIMIT: usize = 1024;
const CALL_STACK_SIZE_LIMIT: usize = 1024;

/// The operand stack.
struct Stack(Vec<Value>);

impl Stack {
    /// Create a new empty operand stack.
    fn new() -> Self {
        Stack(vec![])
    }

    /// Push a `Value` on the stack if the max stack size has not been reached. Abort execution
    /// otherwise.
    fn push(&mut self, value: Value) -> VMResult<()> {
        if self.0.len() < OPERAND_STACK_SIZE_LIMIT {
            self.0.push(value);
            Ok(())
        } else {
            Err(VMStatus::new(StatusCode::EXECUTION_STACK_OVERFLOW))
        }
    }

    /// Pop a `Value` off the stack or abort execution if the stack is empty.
    fn pop(&mut self) -> VMResult<Value> {
        self.0
            .pop()
            .ok_or_else(|| VMStatus::new(StatusCode::EMPTY_VALUE_STACK))
    }

    /// Pop a `Value` of a given type off the stack. Abort if the value is not of the given
    /// type or if the stack is empty.
    fn pop_as<T>(&mut self) -> VMResult<T>
    where
        VMResult<T>: From<Value>,
    {
        self.pop()?.value_as()
    }

    /// Pop n values off the stack.
    fn popn(&mut self, n: u16) -> VMResult<Vec<Value>> {
        let remaining_stack_size = self
            .0
            .len()
            .checked_sub(n as usize)
            .ok_or_else(|| VMStatus::new(StatusCode::EMPTY_VALUE_STACK))?;
        let args = self.0.split_off(remaining_stack_size);
        Ok(args)
    }
}

/// A call stack.
#[derive(Debug)]
struct CallStack<'txn>(Vec<Frame<'txn, FunctionRef<'txn>>>);

impl<'txn> CallStack<'txn> {
    /// Create a new empty call stack.
    fn new() -> Self {
        CallStack(vec![])
    }

    /// Push a `Frame` on the call stack.
    fn push(
        &mut self,
        frame: Frame<'txn, FunctionRef<'txn>>,
    ) -> ::std::result::Result<(), Frame<'txn, FunctionRef<'txn>>> {
        if self.0.len() < CALL_STACK_SIZE_LIMIT {
            self.0.push(frame);
            Ok(())
        } else {
            Err(frame)
        }
    }

    /// Pop a `Frame` off the call stack.
    fn pop(&mut self) -> Option<Frame<'txn, FunctionRef<'txn>>> {
        self.0.pop()
    }
}

/// A `Frame` is the execution context for a function. It holds the locals of the function and
/// the function itself.
#[derive(Debug)]
struct Frame<'txn, F: 'txn> {
    pc: u16,
    locals: Locals,
    function: F,
    type_actual_tags: Vec<TypeTag>,
    phantom: PhantomData<&'txn F>,
}

/// An `ExitCode` from `execute_code_unit`.
#[derive(Debug)]
enum ExitCode {
    /// A `Return` opcode was found.
    Return,
    /// A `Call` opcode was found.
    Call(FunctionHandleIndex, LocalsSignatureIndex),
}

impl<'txn, F> Frame<'txn, F>
where
    F: FunctionReference<'txn>,
{
    /// Create a new `Frame` given a `FunctionReference` and the function `Locals`.
    ///
    /// The locals must be loaded before calling this.
    fn new(function: F, type_actual_tags: Vec<TypeTag>, locals: Locals) -> Self {
        Frame {
            pc: 0,
            locals,
            function,
            type_actual_tags,
            phantom: PhantomData,
        }
    }

    /// Return the code stream of this function.
    fn code_definition(&self) -> &'txn [Bytecode] {
        self.function.code_definition()
    }

    /// Return the `LoadedModule` this function lives in.
    fn module(&self) -> &'txn LoadedModule {
        self.function.module()
    }

    /// Copy a local from this frame at the given index. Return an error if the index is
    /// out of bounds or the local is `Invalid`.
    fn copy_loc(&self, idx: LocalIndex) -> VMResult<Value> {
        self.locals.copy_loc(idx as usize)
    }

    /// Move a local from this frame at the given index. Return an error if the index is
    /// out of bounds or the local is `Invalid`.
    fn move_loc(&mut self, idx: LocalIndex) -> VMResult<Value> {
        self.locals.move_loc(idx as usize)
    }

    /// Store a `Value` into a local at the given index. Return an error if the index is
    /// out of bounds.
    fn store_loc(&mut self, idx: LocalIndex, value: Value) -> VMResult<()> {
        self.locals.store_loc(idx as usize, value)
    }

    /// Borrow a local from this frame at the given index. Return an error if the index is
    /// out of bounds or the local is `Invalid`.
    fn borrow_loc(&mut self, idx: LocalIndex) -> VMResult<Value> {
        self.locals.borrow_loc(idx as usize)
    }

    fn type_actual_tags(&self) -> &[TypeTag] {
        &self.type_actual_tags
    }
}

//
// Below are all the functions needed for gas synthesis and gas cost.
// The story is going to change given those functions expose internals of the Interpreter that
// should never leak out.
// For now they are grouped in a couple of temporary struct and impl that can be used
// to determine what the needs of gas logic has to be.
//
#[cfg(any(test, feature = "instruction_synthesis"))]
pub struct InterpreterForCostSynthesis<'alloc, 'txn, P>
where
    'alloc: 'txn,
    P: ModuleCache<'alloc>,
{
    interpreter: Interpreter<'alloc, 'txn, P>,
    context: TransactionExecutionContext<'txn>,
}

#[cfg(any(test, feature = "instruction_synthesis"))]
impl<'alloc, 'txn, P> InterpreterForCostSynthesis<'alloc, 'txn, P>
where
    'alloc: 'txn,
    P: ModuleCache<'alloc>,
{
    pub fn new(
        module_cache: &'txn P,
        txn_data: &'txn TransactionMetadata,
        data_cache: &'txn dyn RemoteCache,
        gas_schedule: &'txn CostTable,
    ) -> Self {
        let interpreter = Interpreter::new(module_cache, txn_data, gas_schedule);
        let context = TransactionExecutionContext::new(*MAXIMUM_NUMBER_OF_GAS_UNITS, data_cache);
        InterpreterForCostSynthesis {
            interpreter,
            context,
        }
    }

    pub fn clear_writes(&mut self) {
        self.context.clear();
    }

    pub fn set_stack(&mut self, stack: Vec<Value>) {
        self.interpreter.operand_stack.0 = stack;
    }

    pub fn call_stack_height(&self) -> usize {
        self.interpreter.call_stack.0.len()
    }

    pub fn pop_call(&mut self) {
        self.interpreter
            .call_stack
            .pop()
            .expect("call stack must not be empty");
    }

    pub fn push_frame(&mut self, func: FunctionRef<'txn>, type_actual_tags: Vec<TypeTag>) {
        let count = func.local_count();
        self.interpreter
            .call_stack
            .push(Frame::new(func, type_actual_tags, Locals::new(count)))
            .expect("Call stack limit reached");
    }

    pub fn load_call(&mut self, args: HashMap<LocalIndex, Value>) {
        let mut current_frame = self.interpreter.call_stack.pop().expect("frame must exist");
        for (local_index, local) in args.into_iter() {
            current_frame
                .store_loc(local_index, local)
                .expect("local must exist");
        }
        self.interpreter
            .call_stack
            .push(current_frame)
            .expect("Call stack limit reached");
    }

    pub fn execute_code_snippet(&mut self, code: &[Bytecode]) -> VMResult<()> {
        let mut current_frame = self.interpreter.call_stack.pop().expect("frame must exist");
        self.interpreter
            .execute_code_unit(&mut self.context, &mut current_frame, code)?;
        self.interpreter
            .call_stack
            .push(current_frame)
            .expect("Call stack limit reached");
        Ok(())
    }
}
