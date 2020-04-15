// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    gas,
    interpreter_context::InterpreterContext,
    loaded_data::{
        function::{FunctionRef, FunctionReference},
        loaded_module::LoadedModule,
    },
    runtime::VMRuntime,
    special_names::{EMIT_EVENT_NAME, PRINT_STACK_TRACE_NAME, SAVE_ACCOUNT_NAME},
};
use libra_logger::prelude::*;
use libra_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_config::{self, AccountResource, BalanceResource},
    contract_event::ContractEvent,
    event::EventKey,
    move_resource::MoveResource,
    transaction::MAX_TRANSACTION_SIZE_IN_BYTES,
    vm_error::{StatusCode, StatusType, VMStatus},
};
use move_core_types::{
    gas_schedule::{AbstractMemorySize, CostTable, GasAlgebra, GasCarrier, NativeCostIndex},
    identifier::IdentStr,
};
use move_vm_types::{
    identifier::create_access_path,
    loaded_data::types::{StructType, Type},
    native_functions::dispatch::NativeFunction,
    values::{self, IntegerValue, Locals, Reference, Struct, StructRef, VMValueCast, Value},
};
use std::{cmp::min, collections::VecDeque, convert::TryFrom, fmt::Write, marker::PhantomData};
use vm::{
    access::ModuleAccess,
    errors::*,
    file_format::{
        Bytecode, FunctionHandleIndex, FunctionInstantiationIndex, LocalIndex, Signature,
        StructDefinitionIndex,
    },
    gas_schedule::{calculate_intrinsic_gas, Opcodes},
    transaction_metadata::TransactionMetadata,
};

macro_rules! debug_write {
    ($($toks: tt)*) => {
        write!($($toks)*).map_err(|_|
            VMStatus::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                .with_message("failed to write to buffer".to_string())
        )
    };
}

macro_rules! debug_writeln {
    ($($toks: tt)*) => {
        writeln!($($toks)*).map_err(|_|
            VMStatus::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                .with_message("failed to write to buffer".to_string())
        )
    };
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
pub(crate) struct Interpreter<'txn> {
    /// Operand stack, where Move `Value`s are stored for stack operations.
    operand_stack: Stack,
    /// The stack of active functions.
    call_stack: CallStack<'txn>,
    /// Transaction data to resolve special bytecodes (e.g. GetTxnSequenceNumber, GetTxnPublicKey,
    /// GetTxnSenderAddress, ...)
    txn_data: &'txn TransactionMetadata,
    gas_schedule: &'txn CostTable,
}

impl<'txn> Interpreter<'txn> {
    /// Entrypoint into the interpreter. All external calls need to be routed through this
    /// function.
    pub(crate) fn entrypoint(
        context: &mut dyn InterpreterContext,
        runtime: &'txn VMRuntime<'_>,
        txn_data: &'txn TransactionMetadata,
        gas_schedule: &'txn CostTable,
        func: FunctionRef<'txn>,
        ty_args: Vec<Type>,
        args: Vec<Value>,
    ) -> VMResult<()> {
        // We charge an intrinsic amount of gas based upon the size of the transaction submitted
        // (in raw bytes).
        let txn_size = txn_data.transaction_size();
        // The callers of this function verify the transaction before executing it. Transaction
        // verification ensures the following condition.
        // TODO: This is enforced by Libra but needs to be enforced by other callers of the Move VM
        // as well.
        assume!(txn_size.get() <= (MAX_TRANSACTION_SIZE_IN_BYTES as u64));
        // We count the intrinsic cost of the transaction here, since that needs to also cover the
        // setup of the function.
        let mut interp = Self::new(txn_data, gas_schedule);
        gas!(consume: context, calculate_intrinsic_gas(txn_size))?;
        interp.execute(runtime, context, func, ty_args, args)
    }

    /// Create a new instance of an `Interpreter` in the context of a transaction with a
    /// given module cache and gas schedule.
    fn new(txn_data: &'txn TransactionMetadata, gas_schedule: &'txn CostTable) -> Self {
        Interpreter {
            operand_stack: Stack::new(),
            call_stack: CallStack::new(),
            gas_schedule,
            txn_data,
        }
    }

    /// Internal execution entry point.
    fn execute(
        &mut self,
        runtime: &'txn VMRuntime<'_>,
        context: &mut dyn InterpreterContext,
        function: FunctionRef<'txn>,
        ty_args: Vec<Type>,
        args: Vec<Value>,
    ) -> VMResult<()> {
        // No unwinding of the call stack and value stack need to be done here -- the context will
        // take care of that.
        self.execute_main(runtime, context, function, ty_args, args, 0)
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
        runtime: &'txn VMRuntime<'_>,
        context: &mut dyn InterpreterContext,
        function: FunctionRef<'txn>,
        ty_args: Vec<Type>,
        args: Vec<Value>,
        create_account_marker: usize,
    ) -> VMResult<()> {
        let mut locals = Locals::new(function.local_count());
        // TODO: assert consistency of args and function formals
        for (i, value) in args.into_iter().enumerate() {
            locals.store_loc(i, value)?;
        }
        let mut current_frame = Frame::new(function, ty_args, locals);
        loop {
            let code = current_frame.code_definition();
            let exit_code = self
                .execute_code_unit(runtime, context, &mut current_frame, code)
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
                ExitCode::Call(fh_idx) => {
                    gas!(
                        instr: context,
                        self,
                        Opcodes::CALL_GENERIC,
                        AbstractMemorySize::new(1 as GasCarrier)
                    )?;
                    // TODO: when a native function is executed, the current frame has not yet
                    // been pushed onto the call stack. Fix it.
                    let opt_frame = self
                        .make_call_frame(runtime, context, current_frame.module(), fh_idx, vec![])
                        .or_else(|err| Err(self.maybe_core_dump(err, &current_frame)))?;
                    if let Some(frame) = opt_frame {
                        self.call_stack.push(current_frame).or_else(|frame| {
                            let err = VMStatus::new(StatusCode::CALL_STACK_OVERFLOW);
                            Err(self.maybe_core_dump(err, &frame))
                        })?;
                        current_frame = frame;
                    }
                }
                ExitCode::CallGeneric(idx) => {
                    let func_inst = &current_frame.module().function_instantiation_at(idx);
                    let ty_arg_sigs = &current_frame
                        .module()
                        .signature_at(func_inst.type_parameters)
                        .0;
                    gas!(
                        instr: context,
                        self,
                        Opcodes::CALL_GENERIC,
                        AbstractMemorySize::new((ty_arg_sigs.len() + 1) as GasCarrier)
                    )?;
                    let ty_args = ty_arg_sigs
                        .iter()
                        .map(|tok| {
                            runtime.resolve_signature_token(
                                current_frame.module(),
                                tok,
                                current_frame.ty_args(),
                                context,
                            )
                        })
                        .collect::<VMResult<Vec<_>>>()?;

                    // TODO: when a native function is executed, the current frame has not yet
                    // been pushed onto the call stack. Fix it.
                    let opt_frame = self
                        .make_call_frame(
                            runtime,
                            context,
                            current_frame.module(),
                            func_inst.handle,
                            ty_args,
                        )
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
        runtime: &VMRuntime<'_>,
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
                    Bytecode::LdConst(idx) => {
                        let constant = frame.module().constant_at(*idx);
                        gas!(
                            instr: context,
                            self,
                            Opcodes::LD_CONST,
                            AbstractMemorySize::new(constant.data.len() as GasCarrier)
                        )?;
                        self.operand_stack.push(
                            Value::deserialize_constant(constant).ok_or_else(|| {
                                VMStatus::new(StatusCode::VERIFIER_INVARIANT_VIOLATION)
                                    .with_message(
                                    "Verifier failed to verify the deserialization of constants"
                                        .to_owned(),
                                )
                            })?,
                        )?
                    }
                    Bytecode::LdTrue => {
                        gas!(const_instr: context, self, Opcodes::LD_TRUE)?;
                        self.operand_stack.push(Value::bool(true))?;
                    }
                    Bytecode::LdFalse => {
                        gas!(const_instr: context, self, Opcodes::LD_FALSE)?;
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
                    Bytecode::Call(idx) => {
                        return Ok(ExitCode::Call(*idx));
                    }
                    Bytecode::CallGeneric(idx) => {
                        return Ok(ExitCode::CallGeneric(*idx));
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
                        let field_handle = frame.module().field_handle_at(*fd_idx);
                        let field_offset = field_handle.field;
                        let reference = self.operand_stack.pop_as::<StructRef>()?;
                        let field_ref = reference.borrow_field(field_offset as usize)?;
                        self.operand_stack.push(field_ref)?;
                    }
                    Bytecode::ImmBorrowFieldGeneric(fi_idx)
                    | Bytecode::MutBorrowFieldGeneric(fi_idx) => {
                        let field_inst = frame.module().field_instantiation_at(*fi_idx);
                        let field_handle = frame.module().field_handle_at(field_inst.handle);
                        let opcode = match instruction {
                            Bytecode::MutBorrowField(_) => Opcodes::MUT_BORROW_FIELD_GENERIC,
                            _ => Opcodes::IMM_BORROW_FIELD_GENERIC,
                        };
                        gas!(const_instr: context, self, opcode)?;
                        let field_offset = field_handle.field;
                        let reference = self.operand_stack.pop_as::<StructRef>()?;
                        let field_ref = reference.borrow_field(field_offset as usize)?;
                        self.operand_stack.push(field_ref)?;
                    }
                    Bytecode::Pack(sd_idx) => {
                        let struct_def = frame.module().struct_def_at(*sd_idx);
                        let field_count = struct_def.declared_field_count()?;
                        let args = self.operand_stack.popn(field_count)?;
                        let size = args.iter().fold(
                            AbstractMemorySize::new(GasCarrier::from(field_count)),
                            |acc, v| acc.add(v.size()),
                        );
                        gas!(instr: context, self, Opcodes::PACK, size)?;
                        self.operand_stack
                            .push(Value::struct_(Struct::pack(args)))?;
                    }
                    Bytecode::PackGeneric(si_idx) => {
                        let struct_inst = frame.module().struct_instantiation_at(*si_idx);
                        let struct_def = frame.module().struct_def_at(struct_inst.def);
                        let field_count = struct_def.declared_field_count()?;
                        let args = self.operand_stack.popn(field_count)?;
                        let size = args.iter().fold(
                            AbstractMemorySize::new(GasCarrier::from(field_count)),
                            |acc, v| acc.add(v.size()),
                        );
                        gas!(instr: context, self, Opcodes::PACK_GENERIC, size)?;
                        self.operand_stack
                            .push(Value::struct_(Struct::pack(args)))?;
                    }
                    Bytecode::Unpack(sd_idx) => {
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
                        for value in struct_.unpack()? {
                            gas!(instr: context, self, Opcodes::UNPACK, value.size())?;
                            self.operand_stack.push(value)?;
                        }
                    }
                    Bytecode::UnpackGeneric(si_idx) => {
                        let struct_inst = frame.module().struct_instantiation_at(*si_idx);
                        let struct_def = frame.module().struct_def_at(struct_inst.def);
                        let field_count = struct_def.declared_field_count()?;
                        let struct_ = self.operand_stack.pop_as::<Struct>()?;
                        gas!(
                            instr: context,
                            self,
                            Opcodes::UNPACK_GENERIC,
                            AbstractMemorySize::new(GasCarrier::from(field_count))
                        )?;
                        // TODO: Whether or not we want this gas metering in the loop is
                        // questionable.  However, if we don't have it in the loop we could wind up
                        // doing a fair bit of work before charging for it.
                        for value in struct_.unpack()? {
                            gas!(instr: context, self, Opcodes::UNPACK_GENERIC, value.size())?;
                            self.operand_stack.push(value)?;
                        }
                    }
                    Bytecode::ReadRef => {
                        let reference = self.operand_stack.pop_as::<Reference>()?;
                        let value = reference.read_ref()?;
                        gas!(instr: context, self, Opcodes::READ_REF, value.size())?;
                        self.operand_stack.push(value)?;
                    }
                    Bytecode::WriteRef => {
                        let reference = self.operand_stack.pop_as::<Reference>()?;
                        let value = self.operand_stack.pop()?;
                        gas!(instr: context, self, Opcodes::WRITE_REF, value.size())?;
                        reference.write_ref(value)?;
                    }
                    Bytecode::CastU8 => {
                        gas!(const_instr: context, self, Opcodes::CAST_U8)?;
                        let integer_value = self.operand_stack.pop_as::<IntegerValue>()?;
                        self.operand_stack
                            .push(Value::u8(integer_value.cast_u8()?))?;
                    }
                    Bytecode::CastU64 => {
                        gas!(const_instr: context, self, Opcodes::CAST_U64)?;
                        let integer_value = self.operand_stack.pop_as::<IntegerValue>()?;
                        self.operand_stack
                            .push(Value::u64(integer_value.cast_u64()?))?;
                    }
                    Bytecode::CastU128 => {
                        gas!(const_instr: context, self, Opcodes::CAST_U128)?;
                        let integer_value = self.operand_stack.pop_as::<IntegerValue>()?;
                        self.operand_stack
                            .push(Value::u128(integer_value.cast_u128()?))?;
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
                        let rhs = self.operand_stack.pop_as::<u8>()?;
                        let lhs = self.operand_stack.pop_as::<IntegerValue>()?;
                        self.operand_stack
                            .push(lhs.shl_checked(rhs)?.into_value())?;
                    }
                    Bytecode::Shr => {
                        gas!(const_instr: context, self, Opcodes::SHR)?;
                        let rhs = self.operand_stack.pop_as::<u8>()?;
                        let lhs = self.operand_stack.pop_as::<IntegerValue>()?;
                        self.operand_stack
                            .push(lhs.shr_checked(rhs)?.into_value())?;
                    }
                    Bytecode::Or => {
                        gas!(const_instr: context, self, Opcodes::OR)?;
                        self.binop_bool(|l, r| Ok(l || r))?
                    }
                    Bytecode::And => {
                        gas!(const_instr: context, self, Opcodes::AND)?;
                        self.binop_bool(|l, r| Ok(l && r))?
                    }
                    Bytecode::Lt => {
                        gas!(const_instr: context, self, Opcodes::LT)?;
                        self.binop_bool(IntegerValue::lt)?
                    }
                    Bytecode::Gt => {
                        gas!(const_instr: context, self, Opcodes::GT)?;
                        self.binop_bool(IntegerValue::gt)?
                    }
                    Bytecode::Le => {
                        gas!(const_instr: context, self, Opcodes::LE)?;
                        self.binop_bool(IntegerValue::le)?
                    }
                    Bytecode::Ge => {
                        gas!(const_instr: context, self, Opcodes::GE)?;
                        self.binop_bool(IntegerValue::ge)?
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
                        self.operand_stack.push(Value::bool(!lhs.equals(&rhs)?))?;
                    }
                    Bytecode::GetTxnSenderAddress => {
                        gas!(const_instr: context, self, Opcodes::GET_TXN_SENDER)?;
                        self.operand_stack
                            .push(Value::address(self.txn_data.sender()))?;
                    }
                    Bytecode::MutBorrowGlobal(sd_idx) | Bytecode::ImmBorrowGlobal(sd_idx) => {
                        let addr = self.operand_stack.pop_as::<AccountAddress>()?;
                        let size = self.global_data_op(
                            runtime,
                            context,
                            addr,
                            *sd_idx,
                            &Signature(vec![]),
                            frame,
                            Self::borrow_global,
                        )?;
                        gas!(instr: context, self, Opcodes::MUT_BORROW_GLOBAL, size)?;
                    }
                    Bytecode::MutBorrowGlobalGeneric(si_idx)
                    | Bytecode::ImmBorrowGlobalGeneric(si_idx) => {
                        let struct_inst = frame.module().struct_instantiation_at(*si_idx);
                        let instantiation =
                            frame.module().signature_at(struct_inst.type_parameters);
                        let addr = self.operand_stack.pop_as::<AccountAddress>()?;
                        let size = self.global_data_op(
                            runtime,
                            context,
                            addr,
                            struct_inst.def,
                            instantiation,
                            frame,
                            Self::borrow_global,
                        )?;
                        gas!(
                            instr: context,
                            self,
                            Opcodes::MUT_BORROW_GLOBAL_GENERIC,
                            size
                        )?;
                    }
                    Bytecode::Exists(sd_idx) => {
                        let addr = self.operand_stack.pop_as::<AccountAddress>()?;
                        let size = self.global_data_op(
                            runtime,
                            context,
                            addr,
                            *sd_idx,
                            &Signature(vec![]),
                            frame,
                            Self::exists,
                        )?;
                        gas!(instr: context, self, Opcodes::EXISTS, size)?;
                    }
                    Bytecode::ExistsGeneric(si_idx) => {
                        let struct_inst = frame.module().struct_instantiation_at(*si_idx);
                        let instantiation =
                            frame.module().signature_at(struct_inst.type_parameters);
                        let addr = self.operand_stack.pop_as::<AccountAddress>()?;
                        let size = self.global_data_op(
                            runtime,
                            context,
                            addr,
                            struct_inst.def,
                            instantiation,
                            frame,
                            Self::exists,
                        )?;
                        gas!(instr: context, self, Opcodes::EXISTS_GENERIC, size)?;
                    }
                    Bytecode::MoveFrom(sd_idx) => {
                        let addr = self.operand_stack.pop_as::<AccountAddress>()?;
                        let size = self.global_data_op(
                            runtime,
                            context,
                            addr,
                            *sd_idx,
                            &Signature(vec![]),
                            frame,
                            Self::move_from,
                        )?;
                        // TODO: Have this calculate before pulling in the data based upon
                        // the size of the data that we are about to read in.
                        gas!(instr: context, self, Opcodes::MOVE_FROM, size)?;
                    }
                    Bytecode::MoveFromGeneric(si_idx) => {
                        let struct_inst = frame.module().struct_instantiation_at(*si_idx);
                        let instantiation =
                            frame.module().signature_at(struct_inst.type_parameters);
                        let addr = self.operand_stack.pop_as::<AccountAddress>()?;
                        let size = self.global_data_op(
                            runtime,
                            context,
                            addr,
                            struct_inst.def,
                            instantiation,
                            frame,
                            Self::move_from,
                        )?;
                        // TODO: Have this calculate before pulling in the data based upon
                        // the size of the data that we are about to read in.
                        gas!(instr: context, self, Opcodes::MOVE_FROM_GENERIC, size)?;
                    }
                    Bytecode::MoveToSender(sd_idx) => {
                        let addr = self.txn_data.sender();
                        let size = self.global_data_op(
                            runtime,
                            context,
                            addr,
                            *sd_idx,
                            &Signature(vec![]),
                            frame,
                            Self::move_to_sender,
                        )?;
                        gas!(instr: context, self, Opcodes::MOVE_TO, size)?;
                    }
                    Bytecode::MoveToSenderGeneric(si_idx) => {
                        let struct_inst = frame.module().struct_instantiation_at(*si_idx);
                        let instantiation =
                            frame.module().signature_at(struct_inst.type_parameters);
                        let addr = self.txn_data.sender();
                        let size = self.global_data_op(
                            runtime,
                            context,
                            addr,
                            struct_inst.def,
                            instantiation,
                            frame,
                            Self::move_to_sender,
                        )?;
                        gas!(instr: context, self, Opcodes::MOVE_TO_GENERIC, size)?;
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
                    Bytecode::Nop => {
                        gas!(const_instr: context, self, Opcodes::NOP)?;
                    }
                }
            }
            // ok we are out, it's a branch, check the pc for good luck
            // TODO: re-work the logic here. Tests should have a more
            // natural way to plug in
            if frame.pc as usize >= code.len() {
                if cfg!(test) {
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
        runtime: &'txn VMRuntime<'_>,
        context: &mut dyn InterpreterContext,
        module: &LoadedModule,
        idx: FunctionHandleIndex,
        ty_args: Vec<Type>,
    ) -> VMResult<Option<Frame<'txn, FunctionRef<'txn>>>> {
        let func = runtime.resolve_function_ref(module, idx, context)?;
        if func.is_native() {
            self.call_native(runtime, context, func, ty_args)?;
            Ok(None)
        } else {
            let mut locals = Locals::new(func.local_count());
            let arg_count = func.arg_count();
            for i in 0..arg_count {
                locals.store_loc(arg_count - i - 1, self.operand_stack.pop()?)?;
            }
            Ok(Some(Frame::new(func, ty_args, locals)))
        }
    }

    /// Call a native functions.
    fn call_native(
        &mut self,
        runtime: &'txn VMRuntime<'_>,
        context: &mut dyn InterpreterContext,
        function: FunctionRef<'txn>,
        ty_args: Vec<Type>,
    ) -> VMResult<()> {
        let module = function.module();
        let module_id = module.self_id();
        let function_name = function.name();
        let native_function = {
            NativeFunction::resolve(&module_id, function_name)
                .ok_or_else(|| VMStatus::new(StatusCode::LINKER_ERROR))?
        };
        if module_id == *account_config::EVENT_MODULE
            && function_name == EMIT_EVENT_NAME.as_ident_str()
        {
            self.call_emit_event(context, ty_args)
        } else if module_id == *account_config::ACCOUNT_MODULE
            && function_name == SAVE_ACCOUNT_NAME.as_ident_str()
        {
            self.call_save_account(runtime, context, ty_args)
        } else if module_id == *account_config::DEBUG_MODULE
            && function_name == PRINT_STACK_TRACE_NAME.as_ident_str()
        {
            self.call_print_stack_trace(runtime, context, ty_args)
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
            let result = native_function.dispatch(ty_args, arguments, self.gas_schedule)?;
            gas!(consume: context, result.cost)?;
            result.result.and_then(|values| {
                for value in values {
                    self.operand_stack.push(value)?;
                }
                Ok(())
            })
        }
    }

    #[allow(unused_variables)]
    fn call_print_stack_trace(
        &mut self,
        runtime: &'txn VMRuntime<'_>,
        context: &mut dyn InterpreterContext,
        ty_args: Vec<Type>,
    ) -> VMResult<()> {
        if !ty_args.is_empty() {
            return Err(
                VMStatus::new(StatusCode::VERIFIER_INVARIANT_VIOLATION).with_message(format!(
                    "print_stack_trace expects no type arguments got {}.",
                    ty_args.len()
                )),
            );
        }

        #[cfg(feature = "debug_module")]
        {
            let mut s = String::new();
            self.debug_print_stack_trace(&mut s, runtime, context)?;
            println!("{}", s);
        }

        Ok(())
    }

    /// Emit an event if the native function was `write_to_event_store`.
    fn call_emit_event(
        &mut self,
        context: &mut dyn InterpreterContext,
        mut ty_args: Vec<Type>,
    ) -> VMResult<()> {
        if ty_args.len() != 1 {
            return Err(
                VMStatus::new(StatusCode::VERIFIER_INVARIANT_VIOLATION).with_message(format!(
                    "write_to_event_storage expects 1 type argument got {}.",
                    ty_args.len()
                )),
            );
        }

        let ty = ty_args.pop().unwrap();

        let msg = self
            .operand_stack
            .pop()?
            .simple_serialize(&ty)
            .ok_or_else(|| VMStatus::new(StatusCode::DATA_FORMAT_ERROR))?;
        let count = self.operand_stack.pop_as::<u64>()?;
        let key = self.operand_stack.pop_as::<Vec<u8>>()?;
        let guid = EventKey::try_from(key.as_slice())
            .map_err(|_| VMStatus::new(StatusCode::EVENT_KEY_MISMATCH))?;
        context.push_event(ContractEvent::new(guid, count, ty.into_type_tag()?, msg));
        Ok(())
    }

    /// Save an account into the data store.
    fn call_save_account(
        &mut self,
        runtime: &'txn VMRuntime<'_>,
        context: &mut dyn InterpreterContext,
        ty_args: Vec<Type>,
    ) -> VMResult<()> {
        gas!(
            consume: context,
            self.gas_schedule
                .native_cost(NativeCostIndex::SAVE_ACCOUNT)
                .total()
        )?;
        let account_module = runtime.get_loaded_module(&account_config::ACCOUNT_MODULE, context)?;
        let event_module = runtime.get_loaded_module(&account_config::EVENT_MODULE, context)?;
        let account_type_module =
            runtime.get_loaded_module(&account_config::ACCOUNT_TYPE_MODULE, context)?;
        let address = self.operand_stack.pop_as::<AccountAddress>()?;
        if address == account_config::CORE_CODE_ADDRESS {
            return Err(VMStatus::new(StatusCode::CREATE_NULL_ACCOUNT));
        }
        Self::save_under_address(
            runtime,
            context,
            &[],
            event_module,
            account_config::event_handle_generator_struct_name(),
            self.operand_stack.pop_as::<Struct>()?,
            address,
        )?;
        Self::save_under_address(
            runtime,
            context,
            &[],
            account_module,
            &AccountResource::struct_identifier(),
            self.operand_stack.pop_as::<Struct>()?,
            address,
        )?;
        Self::save_under_address(
            runtime,
            context,
            &[ty_args[0].clone()],
            account_module,
            &BalanceResource::struct_identifier(),
            self.operand_stack.pop_as::<Struct>()?,
            address,
        )?;
        Self::save_under_address(
            runtime,
            context,
            &[ty_args[1].clone()],
            account_type_module,
            account_config::account_type_struct_name(),
            self.operand_stack.pop_as::<Struct>()?,
            address,
        )
    }

    /// Perform a binary operation to two values at the top of the stack.
    fn binop<F, T>(&mut self, f: F) -> VMResult<()>
    where
        Value: VMValueCast<T>,
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
        Value: VMValueCast<T>,
        F: FnOnce(T, T) -> VMResult<bool>,
    {
        self.binop(|lhs, rhs| Ok(Value::bool(f(lhs, rhs)?)))
    }

    /// Entry point for all global store operations (effectively opcodes).
    ///
    /// This performs common operation on the data store and then executes the specific
    /// opcode.
    fn global_data_op<F>(
        &mut self,
        runtime: &VMRuntime<'_>,
        context: &mut dyn InterpreterContext,
        address: AccountAddress,
        idx: StructDefinitionIndex,
        ty_arg_sigs: &Signature,
        frame: &Frame<'txn, FunctionRef<'txn>>,
        op: F,
    ) -> VMResult<AbstractMemorySize<GasCarrier>>
    where
        F: FnOnce(
            &mut Self,
            &mut dyn InterpreterContext,
            AccessPath,
            StructType,
        ) -> VMResult<AbstractMemorySize<GasCarrier>>,
    {
        let module = frame.module();
        let ty_args = ty_arg_sigs
            .0
            .iter()
            .map(|tok| {
                runtime.resolve_signature_token(frame.module(), tok, frame.ty_args(), context)
            })
            .collect::<VMResult<Vec<_>>>()?;

        let struct_ty = runtime.resolve_struct_def(module, idx, &ty_args, context)?;
        let ap = create_access_path(address, struct_ty.clone().into_struct_tag()?);
        op(self, context, ap, struct_ty)
    }

    /// BorrowGlobal (mutable and not) opcode.
    fn borrow_global(
        &mut self,
        context: &mut dyn InterpreterContext,
        ap: AccessPath,
        struct_ty: StructType,
    ) -> VMResult<AbstractMemorySize<GasCarrier>> {
        let g = context.borrow_global(&ap, struct_ty)?;
        let size = g.size();
        self.operand_stack.push(g.borrow_global()?)?;
        Ok(size)
    }

    /// Exists opcode.
    fn exists(
        &mut self,
        context: &mut dyn InterpreterContext,
        ap: AccessPath,
        struct_ty: StructType,
    ) -> VMResult<AbstractMemorySize<GasCarrier>> {
        let (exists, mem_size) = context.resource_exists(&ap, struct_ty)?;
        self.operand_stack.push(Value::bool(exists))?;
        Ok(mem_size)
    }

    /// MoveFrom opcode.
    fn move_from(
        &mut self,
        context: &mut dyn InterpreterContext,
        ap: AccessPath,
        struct_ty: StructType,
    ) -> VMResult<AbstractMemorySize<GasCarrier>> {
        let resource = context.move_resource_from(&ap, struct_ty)?;
        let size = resource.size();
        self.operand_stack.push(resource)?;
        Ok(size)
    }

    /// MoveToSender opcode.
    fn move_to_sender(
        &mut self,
        context: &mut dyn InterpreterContext,
        ap: AccessPath,
        struct_ty: StructType,
    ) -> VMResult<AbstractMemorySize<GasCarrier>> {
        let resource = self.operand_stack.pop_as::<Struct>()?;
        let size = resource.size();
        context.move_resource_to(&ap, struct_ty, resource)?;
        Ok(size)
    }

    // Save a resource under the address specified by `account_address`
    fn save_under_address(
        runtime: &'txn VMRuntime<'_>,
        context: &mut dyn InterpreterContext,
        ty_args: &[Type],
        module: &LoadedModule,
        struct_name: &IdentStr,
        resource_to_save: Struct,
        account_address: AccountAddress,
    ) -> VMResult<()> {
        let struct_id = module
            .struct_defs_table
            .get(struct_name)
            .ok_or_else(|| VMStatus::new(StatusCode::LINKER_ERROR))?;
        let struct_ty = runtime.resolve_struct_def(module, *struct_id, ty_args, context)?;
        let path = create_access_path(account_address, struct_ty.clone().into_struct_tag()?);
        context.move_resource_to(&path, struct_ty, resource_to_save)
    }

    //
    // Debugging and logging helpers.
    //

    /// Given an `VMStatus` generate a core dump if the error is an `InvariantViolation`.
    fn maybe_core_dump(
        &self,
        mut err: VMStatus,
        current_frame: &Frame<'txn, FunctionRef<'txn>>,
    ) -> VMStatus {
        // a verification error cannot happen at runtime so change it into an invariant violation.
        if err.status_type() == StatusType::Verification {
            crit!("Verification error during runtime: {:?}", err);
            let mut new_err = VMStatus::new(StatusCode::VERIFICATION_ERROR);
            new_err.message = err.message;
            err = new_err;
        }
        if err.is(StatusType::InvariantViolation) {
            let state = self.get_internal_state(current_frame);
            crit!(
                "Error: {:?}\nCORE DUMP: >>>>>>>>>>>>\n{}\n<<<<<<<<<<<<\n",
                err,
                state,
            );
        }
        err
    }

    #[allow(dead_code)]
    fn debug_print_frame<B: Write>(
        &self,
        buf: &mut B,
        runtime: &'txn VMRuntime<'_>,
        context: &mut dyn InterpreterContext,
        idx: usize,
        frame: &Frame<'txn, FunctionRef<'txn>>,
    ) -> VMResult<()> {
        // Print out the function name with type arguments.
        let func = &frame.function;

        debug_write!(
            buf,
            "    [{}] {}::{}::{}",
            idx,
            func.module().address(),
            func.module().name(),
            func.name()
        )?;
        let ty_args = frame.ty_args();
        if !ty_args.is_empty() {
            debug_write!(buf, "<")?;
            let mut it = ty_args.iter();
            if let Some(ty) = it.next() {
                ty.debug_print(buf)?;
                for ty in it {
                    debug_write!(buf, ", ")?;
                    ty.debug_print(buf)?;
                }
            }
            debug_write!(buf, ">")?;
        }
        debug_writeln!(buf)?;

        // Print out the current instruction.
        debug_writeln!(buf)?;
        debug_writeln!(buf, "        Code:")?;
        let pc = frame.pc as usize;
        let code = frame.code_definition();
        let before = if pc > 3 { pc - 3 } else { 0 };
        let after = min(code.len(), pc + 4);
        for (idx, instr) in code.iter().enumerate().take(pc).skip(before) {
            debug_writeln!(buf, "            [{}] {:?}", idx, instr)?;
        }
        debug_writeln!(buf, "          > [{}] {:?}", pc, &code[pc])?;
        for (idx, instr) in code.iter().enumerate().take(after).skip(pc + 1) {
            debug_writeln!(buf, "            [{}] {:?}", idx, instr)?;
        }

        // Print out the locals.
        debug_writeln!(buf)?;
        debug_writeln!(buf, "        Locals:")?;
        match func.locals() {
            Some(locals) => {
                let tys = locals
                    .0
                    .iter()
                    .map(|tok| {
                        runtime.resolve_signature_token(func.module(), tok, ty_args, context)
                    })
                    .collect::<VMResult<Vec<_>>>()?;
                values::debug::print_locals(buf, &tys, &frame.locals)?;
                debug_writeln!(buf)?;
            }
            None => {
                debug_writeln!(buf, "            (none)")?;
            }
        }

        debug_writeln!(buf)?;
        Ok(())
    }

    #[allow(dead_code)]
    fn debug_print_stack_trace<B: Write>(
        &self,
        buf: &mut B,
        runtime: &'txn VMRuntime<'_>,
        context: &mut dyn InterpreterContext,
    ) -> VMResult<()> {
        debug_writeln!(buf, "Call Stack:")?;
        for (i, frame) in self.call_stack.0.iter().enumerate() {
            self.debug_print_frame(buf, runtime, context, i, frame)?;
        }
        debug_writeln!(buf, "Operand Stack:")?;
        for (idx, val) in self.operand_stack.0.iter().enumerate() {
            // TODO: Currently we do not know the types of the values on the operand stack.
            // Revisit.
            debug_writeln!(buf, "    [{}] {}", idx, val)?;
        }
        Ok(())
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
            for bytecode in &code[..pc] {
                internal_state.push_str(format!("{}> {:?}\n", i, bytecode).as_str());
                i += 1;
            }
            internal_state.push_str(format!("{}* {:?}\n", i, code[pc]).as_str());
        }
        internal_state.push_str(format!("Locals:\n{}", current_frame.locals).as_str());
        internal_state.push_str("Operand Stack:\n");
        for value in &self.operand_stack.0 {
            internal_state.push_str(format!("{}\n", value).as_str());
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
        Value: VMValueCast<T>,
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
    ty_args: Vec<Type>,
    phantom: PhantomData<&'txn F>,
}

/// An `ExitCode` from `execute_code_unit`.
#[derive(Debug)]
enum ExitCode {
    Return,
    Call(FunctionHandleIndex),
    CallGeneric(FunctionInstantiationIndex),
}

impl<'txn, F> Frame<'txn, F>
where
    F: FunctionReference<'txn>,
{
    /// Create a new `Frame` given a `FunctionReference` and the function `Locals`.
    ///
    /// The locals must be loaded before calling this.
    fn new(function: F, ty_args: Vec<Type>, locals: Locals) -> Self {
        Frame {
            pc: 0,
            locals,
            function,
            ty_args,
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

    fn ty_args(&self) -> &[Type] {
        &self.ty_args
    }
}
