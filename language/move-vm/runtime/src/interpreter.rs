// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    data_operations::{borrow_global, move_resource_from, move_resource_to, resource_exists},
    loader::{Function, Loader, Resolver},
    native_functions::FunctionContext,
    trace,
};
use libra_logger::prelude::*;
use libra_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    vm_error::{StatusCode, StatusType, VMStatus},
};
use move_core_types::gas_schedule::{AbstractMemorySize, GasAlgebra, GasCarrier};
use move_vm_types::{
    data_store::DataStore,
    gas_schedule::CostStrategy,
    loaded_data::{runtime_types::Type, types::FatStructType},
    values::{self, IntegerValue, Locals, Reference, Struct, StructRef, VMValueCast, Value},
};
use std::{cmp::min, collections::VecDeque, fmt::Write, sync::Arc};
use vm::{
    errors::*,
    file_format::{
        Bytecode, FunctionHandleIndex, FunctionInstantiationIndex, StructDefInstantiationIndex,
        StructDefinitionIndex,
    },
    file_format_common::Opcodes,
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
pub(crate) struct Interpreter {
    /// Operand stack, where Move `Value`s are stored for stack operations.
    operand_stack: Stack,
    /// The stack of active functions.
    call_stack: CallStack,
}

impl Interpreter {
    /// Entrypoint into the interpreter. All external calls need to be routed through this
    /// function.
    pub(crate) fn entrypoint(
        function: Arc<Function>,
        ty_args: Vec<Type>,
        args: Vec<Value>,
        data_store: &mut dyn DataStore,
        cost_strategy: &mut CostStrategy,
        loader: &Loader,
    ) -> VMResult<()> {
        // We count the intrinsic cost of the transaction here, since that needs to also cover the
        // setup of the function.
        let mut interp = Self::new();
        interp.execute(loader, data_store, cost_strategy, function, ty_args, args)
    }

    /// Create a new instance of an `Interpreter` in the context of a transaction with a
    /// given module cache and gas schedule.
    fn new() -> Self {
        Interpreter {
            operand_stack: Stack::new(),
            call_stack: CallStack::new(),
        }
    }

    /// Internal execution entry point.
    fn execute(
        &mut self,
        loader: &Loader,
        data_store: &mut dyn DataStore,
        cost_strategy: &mut CostStrategy,
        function: Arc<Function>,
        ty_args: Vec<Type>,
        args: Vec<Value>,
    ) -> VMResult<()> {
        // No unwinding of the call stack and value stack need to be done here -- the context will
        // take care of that.
        self.execute_main(loader, data_store, cost_strategy, function, ty_args, args)
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
        loader: &Loader,
        data_store: &mut dyn DataStore,
        cost_strategy: &mut CostStrategy,
        function: Arc<Function>,
        ty_args: Vec<Type>,
        args: Vec<Value>,
    ) -> VMResult<()> {
        let mut locals = Locals::new(function.local_count());
        // TODO: assert consistency of args and function formals
        for (i, value) in args.into_iter().enumerate() {
            locals.store_loc(i, value)?;
        }
        let mut current_frame = Frame::new(function, ty_args, locals);
        loop {
            let resolver = current_frame.resolver(loader);
            let exit_code =
                current_frame //self
                    .execute_code(&resolver, self, data_store, cost_strategy)
                    .or_else(|err| Err(self.maybe_core_dump(err, &current_frame)))?;
            match exit_code {
                ExitCode::Return => {
                    current_frame.locals.check_resources_for_return()?;
                    if let Some(frame) = self.call_stack.pop() {
                        current_frame = frame;
                    } else {
                        return Ok(());
                    }
                }
                ExitCode::Call(fh_idx) => {
                    cost_strategy.charge_instr_with_size(
                        Opcodes::CALL,
                        AbstractMemorySize::new(1 as GasCarrier),
                    )?;
                    let func = resolver.function_at(fh_idx);
                    if func.is_native() {
                        self.call_native(&resolver, data_store, cost_strategy, func, vec![])?;
                        continue;
                    }
                    // TODO: when a native function is executed, the current frame has not yet
                    // been pushed onto the call stack. Fix it.
                    let frame = self
                        .make_call_frame(func, vec![])
                        .or_else(|err| Err(self.maybe_core_dump(err, &current_frame)))?;
                    self.call_stack.push(current_frame).or_else(|frame| {
                        let err = VMStatus::new(StatusCode::CALL_STACK_OVERFLOW);
                        Err(self.maybe_core_dump(err, &frame))
                    })?;
                    current_frame = frame;
                }
                ExitCode::CallGeneric(idx) => {
                    let func_inst = resolver.function_instantiation_at(idx);
                    cost_strategy.charge_instr_with_size(
                        Opcodes::CALL_GENERIC,
                        AbstractMemorySize::new((func_inst.instantiation_size() + 1) as GasCarrier),
                    )?;
                    let func = loader.function_at(func_inst.handle());
                    let ty_args = func_inst.materialize(current_frame.ty_args())?;
                    if func.is_native() {
                        self.call_native(&resolver, data_store, cost_strategy, func, ty_args)?;
                        continue;
                    }
                    // TODO: when a native function is executed, the current frame has not yet
                    // been pushed onto the call stack. Fix it.
                    let frame = self
                        .make_call_frame(func, ty_args)
                        .or_else(|err| Err(self.maybe_core_dump(err, &current_frame)))?;
                    self.call_stack.push(current_frame).or_else(|frame| {
                        let err = VMStatus::new(StatusCode::CALL_STACK_OVERFLOW);
                        Err(self.maybe_core_dump(err, &frame))
                    })?;
                    current_frame = frame;
                }
            }
        }
    }

    /// Returns a `Frame` if the call is to a Move function. Calls to native functions are
    /// "inlined" and this returns `None`.
    ///
    /// Native functions do not push a frame at the moment and as such errors from a native
    /// function are incorrectly attributed to the caller.
    fn make_call_frame(&mut self, func: Arc<Function>, ty_args: Vec<Type>) -> VMResult<Frame> {
        let mut locals = Locals::new(func.local_count());
        let arg_count = func.arg_count();
        for i in 0..arg_count {
            locals.store_loc(arg_count - i - 1, self.operand_stack.pop()?)?;
        }
        Ok(Frame::new(func, ty_args, locals))
    }

    /// Call a native functions.
    fn call_native(
        &mut self,
        resolver: &Resolver,
        data_store: &mut dyn DataStore,
        cost_strategy: &mut CostStrategy,
        function: Arc<Function>,
        ty_args: Vec<Type>,
    ) -> VMResult<()> {
        let mut arguments = VecDeque::new();
        let expected_args = function.arg_count();
        for _ in 0..expected_args {
            arguments.push_front(self.operand_stack.pop()?);
        }
        let mut native_context = FunctionContext::new(self, data_store, cost_strategy, resolver);
        let native_function = function.get_native()?;
        let result = native_function.dispatch(&mut native_context, ty_args, arguments)?;
        cost_strategy.deduct_gas(result.cost)?;
        result.result.and_then(|values| {
            for value in values {
                self.operand_stack.push(value)?;
            }
            Ok(())
        })
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
        resolver: &Resolver,
        data_store: &mut dyn DataStore,
        address: AccountAddress,
        idx: StructDefinitionIndex,
        op: F,
    ) -> VMResult<AbstractMemorySize<GasCarrier>>
    where
        F: FnOnce(
            &mut Self,
            &mut dyn DataStore,
            AccessPath,
            &FatStructType,
        ) -> VMResult<AbstractMemorySize<GasCarrier>>,
    {
        let struct_type = resolver.struct_at(idx);
        let libra_type = resolver.get_libra_type_info(
            &struct_type.module,
            struct_type.name.as_ident_str(),
            &[],
            data_store,
        )?;
        let ap = AccessPath::new(address, libra_type.resource_key().to_vec());
        op(self, data_store, ap, libra_type.fat_type())
    }

    fn global_data_op_generic<F>(
        &mut self,
        resolver: &Resolver,
        data_store: &mut dyn DataStore,
        address: AccountAddress,
        idx: StructDefInstantiationIndex,
        frame: &Frame,
        op: F,
    ) -> VMResult<AbstractMemorySize<GasCarrier>>
    where
        F: FnOnce(
            &mut Self,
            &mut dyn DataStore,
            AccessPath,
            &FatStructType,
        ) -> VMResult<AbstractMemorySize<GasCarrier>>,
    {
        let struct_inst = resolver.struct_instantiation_at(idx);
        let mut instantiation = vec![];
        for inst in struct_inst.get_instantiation() {
            instantiation.push(inst.subst(frame.ty_args())?);
        }
        let struct_type = resolver.struct_type_at(struct_inst.get_def_idx());
        let libra_type = resolver.get_libra_type_info(
            &struct_type.module,
            struct_type.name.as_ident_str(),
            &instantiation,
            data_store,
        )?;
        let ap = AccessPath::new(address, libra_type.resource_key().to_vec());
        op(self, data_store, ap, libra_type.fat_type())
    }

    /// BorrowGlobal (mutable and not) opcode.
    fn borrow_global(
        &mut self,
        data_store: &mut dyn DataStore,
        ap: AccessPath,
        struct_ty: &FatStructType,
    ) -> VMResult<AbstractMemorySize<GasCarrier>> {
        let g = borrow_global(data_store, &ap, struct_ty)?;
        let size = g.size();
        self.operand_stack.push(g.borrow_global()?)?;
        Ok(size)
    }

    /// Exists opcode.
    fn exists(
        &mut self,
        data_store: &mut dyn DataStore,
        ap: AccessPath,
        struct_ty: &FatStructType,
    ) -> VMResult<AbstractMemorySize<GasCarrier>> {
        let (exists, mem_size) = resource_exists(data_store, &ap, struct_ty)?;
        self.operand_stack.push(Value::bool(exists))?;
        Ok(mem_size)
    }

    /// MoveFrom opcode.
    fn move_from(
        &mut self,
        data_store: &mut dyn DataStore,
        ap: AccessPath,
        struct_ty: &FatStructType,
    ) -> VMResult<AbstractMemorySize<GasCarrier>> {
        let resource = move_resource_from(data_store, &ap, struct_ty)?;
        let size = resource.size();
        self.operand_stack.push(resource)?;
        Ok(size)
    }

    /// MoveTo opcode.
    fn move_to(
        resource: Struct,
    ) -> impl FnOnce(
        &mut Interpreter,
        &mut dyn DataStore,
        AccessPath,
        &FatStructType,
    ) -> VMResult<AbstractMemorySize<GasCarrier>> {
        |_interpreter, data_store, ap, struct_ty| {
            let size = resource.size();
            move_resource_to(data_store, &ap, struct_ty, resource)?;
            Ok(size)
        }
    }

    //
    // Debugging and logging helpers.
    //

    /// Given an `VMStatus` generate a core dump if the error is an `InvariantViolation`.
    fn maybe_core_dump(&self, mut err: VMStatus, current_frame: &Frame) -> VMStatus {
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
        resolver: &Resolver,
        idx: usize,
        frame: &Frame,
    ) -> VMResult<()> {
        // Print out the function name with type arguments.
        let func = &frame.function;

        debug_write!(buf, "    [{}] ", idx)?;
        if let Some(module) = func.module_id() {
            debug_write!(buf, "{}::{}::", module.address(), module.name(),)?;
        }
        debug_write!(buf, "{}", func.name())?;
        let ty_args = frame.ty_args();
        let mut fat_ty_args = vec![];
        for ty in ty_args {
            fat_ty_args.push(resolver.type_to_fat_type(ty)?);
        }
        if !fat_ty_args.is_empty() {
            debug_write!(buf, "<")?;
            let mut it = fat_ty_args.iter();
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
        let code = func.code();
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
        if func.local_count() > 0 {
            let mut tys = vec![];
            for local in &func.locals().0 {
                tys.push(resolver.make_fat_type(local, ty_args)?);
            }
            values::debug::print_locals(buf, &tys, &frame.locals)?;
            debug_writeln!(buf)?;
        } else {
            debug_writeln!(buf, "            (none)")?;
        }

        debug_writeln!(buf)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub(crate) fn debug_print_stack_trace<B: Write>(
        &self,
        buf: &mut B,
        resolver: &Resolver,
    ) -> VMResult<()> {
        debug_writeln!(buf, "Call Stack:")?;
        for (i, frame) in self.call_stack.0.iter().enumerate() {
            self.debug_print_frame(buf, resolver, i, frame)?;
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
    fn get_internal_state(&self, current_frame: &Frame) -> String {
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
        let code = current_frame.function.code();
        let pc = current_frame.pc as usize;
        if pc < code.len() {
            let mut i = 0;
            for bytecode in &code[..pc] {
                internal_state.push_str(format!("{}> {:?}\n", i, bytecode).as_str());
                i += 1;
            }
            internal_state.push_str(format!("{}* {:?}\n", i, code[pc]).as_str());
        }
        internal_state.push_str(format!("Locals:\n{}\n", current_frame.locals).as_str());
        internal_state.push_str("Operand Stack:\n");
        for value in &self.operand_stack.0 {
            internal_state.push_str(format!("{}\n", value).as_str());
        }
        internal_state
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
struct CallStack(Vec<Frame>);

impl CallStack {
    /// Create a new empty call stack.
    fn new() -> Self {
        CallStack(vec![])
    }

    /// Push a `Frame` on the call stack.
    fn push(&mut self, frame: Frame) -> ::std::result::Result<(), Frame> {
        if self.0.len() < CALL_STACK_SIZE_LIMIT {
            self.0.push(frame);
            Ok(())
        } else {
            Err(frame)
        }
    }

    /// Pop a `Frame` off the call stack.
    fn pop(&mut self) -> Option<Frame> {
        self.0.pop()
    }
}

/// A `Frame` is the execution context for a function. It holds the locals of the function and
/// the function itself.
#[derive(Debug)]
struct Frame {
    pc: u16,
    locals: Locals,
    function: Arc<Function>,
    ty_args: Vec<Type>,
}

/// An `ExitCode` from `execute_code_unit`.
#[derive(Debug)]
enum ExitCode {
    Return,
    Call(FunctionHandleIndex),
    CallGeneric(FunctionInstantiationIndex),
}

impl Frame {
    /// Create a new `Frame` given a `Function` and the function `Locals`.
    ///
    /// The locals must be loaded before calling this.
    fn new(function: Arc<Function>, ty_args: Vec<Type>, locals: Locals) -> Self {
        Frame {
            pc: 0,
            locals,
            function,
            ty_args,
        }
    }

    /// Execute a Move function until a return or a call opcode is found.
    fn execute_code(
        &mut self,
        resolver: &Resolver,
        interpreter: &mut Interpreter,
        data_store: &mut dyn DataStore,
        cost_strategy: &mut CostStrategy,
    ) -> VMResult<ExitCode> {
        let code = self.function.code();
        loop {
            for instruction in &code[self.pc as usize..] {
                trace!(self.function.pretty_string(), self.pc, instruction);
                self.pc += 1;

                match instruction {
                    Bytecode::Pop => {
                        cost_strategy.charge_instr(Opcodes::POP)?;
                        interpreter.operand_stack.pop()?;
                    }
                    Bytecode::Ret => {
                        cost_strategy.charge_instr(Opcodes::RET)?;
                        return Ok(ExitCode::Return);
                    }
                    Bytecode::BrTrue(offset) => {
                        cost_strategy.charge_instr(Opcodes::BR_TRUE)?;
                        if interpreter.operand_stack.pop_as::<bool>()? {
                            self.pc = *offset;
                            break;
                        }
                    }
                    Bytecode::BrFalse(offset) => {
                        cost_strategy.charge_instr(Opcodes::BR_FALSE)?;
                        if !interpreter.operand_stack.pop_as::<bool>()? {
                            self.pc = *offset;
                            break;
                        }
                    }
                    Bytecode::Branch(offset) => {
                        cost_strategy.charge_instr(Opcodes::BRANCH)?;
                        self.pc = *offset;
                        break;
                    }
                    Bytecode::LdU8(int_const) => {
                        cost_strategy.charge_instr(Opcodes::LD_U8)?;
                        interpreter.operand_stack.push(Value::u8(*int_const))?;
                    }
                    Bytecode::LdU64(int_const) => {
                        cost_strategy.charge_instr(Opcodes::LD_U64)?;
                        interpreter.operand_stack.push(Value::u64(*int_const))?;
                    }
                    Bytecode::LdU128(int_const) => {
                        cost_strategy.charge_instr(Opcodes::LD_U128)?;
                        interpreter.operand_stack.push(Value::u128(*int_const))?;
                    }
                    Bytecode::LdConst(idx) => {
                        let constant = resolver.constant_at(*idx);
                        cost_strategy.charge_instr_with_size(
                            Opcodes::LD_CONST,
                            AbstractMemorySize::new(constant.data.len() as GasCarrier),
                        )?;
                        interpreter.operand_stack.push(
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
                        cost_strategy.charge_instr(Opcodes::LD_TRUE)?;
                        interpreter.operand_stack.push(Value::bool(true))?;
                    }
                    Bytecode::LdFalse => {
                        cost_strategy.charge_instr(Opcodes::LD_FALSE)?;
                        interpreter.operand_stack.push(Value::bool(false))?;
                    }
                    Bytecode::CopyLoc(idx) => {
                        let local = self.locals.copy_loc(*idx as usize)?;
                        cost_strategy.charge_instr_with_size(Opcodes::COPY_LOC, local.size())?;
                        interpreter.operand_stack.push(local)?;
                    }
                    Bytecode::MoveLoc(idx) => {
                        let local = self.locals.move_loc(*idx as usize)?;
                        cost_strategy.charge_instr_with_size(Opcodes::MOVE_LOC, local.size())?;

                        interpreter.operand_stack.push(local)?;
                    }
                    Bytecode::StLoc(idx) => {
                        let value_to_store = interpreter.operand_stack.pop()?;
                        cost_strategy
                            .charge_instr_with_size(Opcodes::ST_LOC, value_to_store.size())?;
                        self.locals.store_loc(*idx as usize, value_to_store)?;
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
                        cost_strategy.charge_instr(opcode)?;
                        interpreter
                            .operand_stack
                            .push(self.locals.borrow_loc(*idx as usize)?)?;
                    }
                    Bytecode::ImmBorrowField(fh_idx) | Bytecode::MutBorrowField(fh_idx) => {
                        let opcode = match instruction {
                            Bytecode::MutBorrowField(_) => Opcodes::MUT_BORROW_FIELD,
                            _ => Opcodes::IMM_BORROW_FIELD,
                        };
                        cost_strategy.charge_instr(opcode)?;

                        let reference = interpreter.operand_stack.pop_as::<StructRef>()?;
                        let offset = resolver.field_offset(*fh_idx);
                        let field_ref = reference.borrow_field(offset)?;
                        interpreter.operand_stack.push(field_ref)?;
                    }
                    Bytecode::ImmBorrowFieldGeneric(fi_idx)
                    | Bytecode::MutBorrowFieldGeneric(fi_idx) => {
                        let opcode = match instruction {
                            Bytecode::MutBorrowField(_) => Opcodes::MUT_BORROW_FIELD_GENERIC,
                            _ => Opcodes::IMM_BORROW_FIELD_GENERIC,
                        };
                        cost_strategy.charge_instr(opcode)?;

                        let reference = interpreter.operand_stack.pop_as::<StructRef>()?;
                        let offset = resolver.field_instantiation_offset(*fi_idx);
                        let field_ref = reference.borrow_field(offset)?;
                        interpreter.operand_stack.push(field_ref)?;
                    }
                    Bytecode::Pack(sd_idx) => {
                        let field_count = resolver.field_count(*sd_idx);
                        let args = interpreter.operand_stack.popn(field_count)?;
                        let size = args.iter().fold(
                            AbstractMemorySize::new(GasCarrier::from(field_count)),
                            |acc, v| acc.add(v.size()),
                        );
                        cost_strategy.charge_instr_with_size(Opcodes::PACK, size)?;
                        let is_resource = resolver.struct_at(*sd_idx).is_resource;
                        interpreter
                            .operand_stack
                            .push(Value::struct_(Struct::pack(args, is_resource)))?;
                    }
                    Bytecode::PackGeneric(si_idx) => {
                        let field_count = resolver.field_instantiation_count(*si_idx);
                        let args = interpreter.operand_stack.popn(field_count)?;
                        let size = args.iter().fold(
                            AbstractMemorySize::new(GasCarrier::from(field_count)),
                            |acc, v| acc.add(v.size()),
                        );
                        cost_strategy.charge_instr_with_size(Opcodes::PACK_GENERIC, size)?;
                        let struct_def = resolver.struct_instantiation_at(*si_idx);
                        let struct_ty = resolver.struct_type_at(struct_def.get_def_idx());
                        let is_nominal_resource = struct_ty.is_resource;

                        let is_resource = if is_nominal_resource {
                            true
                        } else {
                            let mut is_resource = false;
                            for ty in struct_def.get_instantiation() {
                                if resolver.is_resource(&ty.subst(self.ty_args())?)? {
                                    is_resource = true;
                                }
                            }
                            is_resource
                        };

                        interpreter
                            .operand_stack
                            .push(Value::struct_(Struct::pack(args, is_resource)))?;
                    }
                    Bytecode::Unpack(sd_idx) => {
                        let field_count = resolver.field_count(*sd_idx);
                        let struct_ = interpreter.operand_stack.pop_as::<Struct>()?;
                        cost_strategy.charge_instr_with_size(
                            Opcodes::UNPACK,
                            AbstractMemorySize::new(GasCarrier::from(field_count)),
                        )?;
                        // TODO: Whether or not we want this gas metering in the loop is
                        // questionable.  However, if we don't have it in the loop we could wind up
                        // doing a fair bit of work before charging for it.
                        for value in struct_.unpack()? {
                            cost_strategy.charge_instr_with_size(Opcodes::UNPACK, value.size())?;
                            interpreter.operand_stack.push(value)?;
                        }
                    }
                    Bytecode::UnpackGeneric(si_idx) => {
                        let field_count = resolver.field_instantiation_count(*si_idx);
                        let struct_ = interpreter.operand_stack.pop_as::<Struct>()?;
                        cost_strategy.charge_instr_with_size(
                            Opcodes::UNPACK_GENERIC,
                            AbstractMemorySize::new(GasCarrier::from(field_count)),
                        )?;
                        // TODO: Whether or not we want this gas metering in the loop is
                        // questionable.  However, if we don't have it in the loop we could wind up
                        // doing a fair bit of work before charging for it.
                        for value in struct_.unpack()? {
                            cost_strategy
                                .charge_instr_with_size(Opcodes::UNPACK_GENERIC, value.size())?;
                            interpreter.operand_stack.push(value)?;
                        }
                    }
                    Bytecode::ReadRef => {
                        let reference = interpreter.operand_stack.pop_as::<Reference>()?;
                        let value = reference.read_ref()?;
                        cost_strategy.charge_instr_with_size(Opcodes::READ_REF, value.size())?;
                        interpreter.operand_stack.push(value)?;
                    }
                    Bytecode::WriteRef => {
                        let reference = interpreter.operand_stack.pop_as::<Reference>()?;
                        let value = interpreter.operand_stack.pop()?;
                        cost_strategy.charge_instr_with_size(Opcodes::WRITE_REF, value.size())?;
                        reference.write_ref(value)?;
                    }
                    Bytecode::CastU8 => {
                        cost_strategy.charge_instr(Opcodes::CAST_U8)?;
                        let integer_value = interpreter.operand_stack.pop_as::<IntegerValue>()?;
                        interpreter
                            .operand_stack
                            .push(Value::u8(integer_value.cast_u8()?))?;
                    }
                    Bytecode::CastU64 => {
                        cost_strategy.charge_instr(Opcodes::CAST_U64)?;
                        let integer_value = interpreter.operand_stack.pop_as::<IntegerValue>()?;
                        interpreter
                            .operand_stack
                            .push(Value::u64(integer_value.cast_u64()?))?;
                    }
                    Bytecode::CastU128 => {
                        cost_strategy.charge_instr(Opcodes::CAST_U128)?;
                        let integer_value = interpreter.operand_stack.pop_as::<IntegerValue>()?;
                        interpreter
                            .operand_stack
                            .push(Value::u128(integer_value.cast_u128()?))?;
                    }
                    // Arithmetic Operations
                    Bytecode::Add => {
                        cost_strategy.charge_instr(Opcodes::ADD)?;
                        interpreter.binop_int(IntegerValue::add_checked)?
                    }
                    Bytecode::Sub => {
                        cost_strategy.charge_instr(Opcodes::SUB)?;
                        interpreter.binop_int(IntegerValue::sub_checked)?
                    }
                    Bytecode::Mul => {
                        cost_strategy.charge_instr(Opcodes::MUL)?;
                        interpreter.binop_int(IntegerValue::mul_checked)?
                    }
                    Bytecode::Mod => {
                        cost_strategy.charge_instr(Opcodes::MOD)?;
                        interpreter.binop_int(IntegerValue::rem_checked)?
                    }
                    Bytecode::Div => {
                        cost_strategy.charge_instr(Opcodes::DIV)?;
                        interpreter.binop_int(IntegerValue::div_checked)?
                    }
                    Bytecode::BitOr => {
                        cost_strategy.charge_instr(Opcodes::BIT_OR)?;
                        interpreter.binop_int(IntegerValue::bit_or)?
                    }
                    Bytecode::BitAnd => {
                        cost_strategy.charge_instr(Opcodes::BIT_AND)?;
                        interpreter.binop_int(IntegerValue::bit_and)?
                    }
                    Bytecode::Xor => {
                        cost_strategy.charge_instr(Opcodes::XOR)?;
                        interpreter.binop_int(IntegerValue::bit_xor)?
                    }
                    Bytecode::Shl => {
                        cost_strategy.charge_instr(Opcodes::SHL)?;
                        let rhs = interpreter.operand_stack.pop_as::<u8>()?;
                        let lhs = interpreter.operand_stack.pop_as::<IntegerValue>()?;
                        interpreter
                            .operand_stack
                            .push(lhs.shl_checked(rhs)?.into_value())?;
                    }
                    Bytecode::Shr => {
                        cost_strategy.charge_instr(Opcodes::SHR)?;
                        let rhs = interpreter.operand_stack.pop_as::<u8>()?;
                        let lhs = interpreter.operand_stack.pop_as::<IntegerValue>()?;
                        interpreter
                            .operand_stack
                            .push(lhs.shr_checked(rhs)?.into_value())?;
                    }
                    Bytecode::Or => {
                        cost_strategy.charge_instr(Opcodes::OR)?;
                        interpreter.binop_bool(|l, r| Ok(l || r))?
                    }
                    Bytecode::And => {
                        cost_strategy.charge_instr(Opcodes::AND)?;
                        interpreter.binop_bool(|l, r| Ok(l && r))?
                    }
                    Bytecode::Lt => {
                        cost_strategy.charge_instr(Opcodes::LT)?;
                        interpreter.binop_bool(IntegerValue::lt)?
                    }
                    Bytecode::Gt => {
                        cost_strategy.charge_instr(Opcodes::GT)?;
                        interpreter.binop_bool(IntegerValue::gt)?
                    }
                    Bytecode::Le => {
                        cost_strategy.charge_instr(Opcodes::LE)?;
                        interpreter.binop_bool(IntegerValue::le)?
                    }
                    Bytecode::Ge => {
                        cost_strategy.charge_instr(Opcodes::GE)?;
                        interpreter.binop_bool(IntegerValue::ge)?
                    }
                    Bytecode::Abort => {
                        cost_strategy.charge_instr(Opcodes::ABORT)?;
                        let error_code = interpreter.operand_stack.pop_as::<u64>()?;
                        return Err(VMStatus::new(StatusCode::ABORTED)
                            .with_sub_status(error_code)
                            .with_message(format!(
                                "{} at offset {}",
                                self.function.pretty_string(),
                                self.pc,
                            )));
                    }
                    Bytecode::Eq => {
                        let lhs = interpreter.operand_stack.pop()?;
                        let rhs = interpreter.operand_stack.pop()?;
                        cost_strategy
                            .charge_instr_with_size(Opcodes::EQ, lhs.size().add(rhs.size()))?;
                        interpreter
                            .operand_stack
                            .push(Value::bool(lhs.equals(&rhs)?))?;
                    }
                    Bytecode::Neq => {
                        let lhs = interpreter.operand_stack.pop()?;
                        let rhs = interpreter.operand_stack.pop()?;
                        cost_strategy
                            .charge_instr_with_size(Opcodes::NEQ, lhs.size().add(rhs.size()))?;
                        interpreter
                            .operand_stack
                            .push(Value::bool(!lhs.equals(&rhs)?))?;
                    }
                    Bytecode::GetTxnSenderAddress => {
                        return Err(VMStatus::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                            .with_message(
                                "GetTxnSenderAddress is deprecated and will be removed soon"
                                    .to_string(),
                            ));
                    }
                    Bytecode::MutBorrowGlobal(sd_idx) | Bytecode::ImmBorrowGlobal(sd_idx) => {
                        let addr = interpreter.operand_stack.pop_as::<AccountAddress>()?;
                        let size = interpreter.global_data_op(
                            resolver,
                            data_store,
                            addr,
                            *sd_idx,
                            Interpreter::borrow_global,
                        )?;
                        cost_strategy.charge_instr_with_size(Opcodes::MUT_BORROW_GLOBAL, size)?;
                    }
                    Bytecode::MutBorrowGlobalGeneric(si_idx)
                    | Bytecode::ImmBorrowGlobalGeneric(si_idx) => {
                        let addr = interpreter.operand_stack.pop_as::<AccountAddress>()?;
                        let size = interpreter.global_data_op_generic(
                            resolver,
                            data_store,
                            addr,
                            *si_idx,
                            self,
                            Interpreter::borrow_global,
                        )?;
                        cost_strategy
                            .charge_instr_with_size(Opcodes::MUT_BORROW_GLOBAL_GENERIC, size)?;
                    }
                    Bytecode::Exists(sd_idx) => {
                        let addr = interpreter.operand_stack.pop_as::<AccountAddress>()?;
                        let size = interpreter.global_data_op(
                            resolver,
                            data_store,
                            addr,
                            *sd_idx,
                            Interpreter::exists,
                        )?;
                        cost_strategy.charge_instr_with_size(Opcodes::EXISTS, size)?;
                    }
                    Bytecode::ExistsGeneric(si_idx) => {
                        let addr = interpreter.operand_stack.pop_as::<AccountAddress>()?;
                        let size = interpreter.global_data_op_generic(
                            resolver,
                            data_store,
                            addr,
                            *si_idx,
                            self,
                            Interpreter::exists,
                        )?;
                        cost_strategy.charge_instr_with_size(Opcodes::EXISTS_GENERIC, size)?;
                    }
                    Bytecode::MoveFrom(sd_idx) => {
                        let addr = interpreter.operand_stack.pop_as::<AccountAddress>()?;
                        let size = interpreter.global_data_op(
                            resolver,
                            data_store,
                            addr,
                            *sd_idx,
                            Interpreter::move_from,
                        )?;
                        // TODO: Have this calculate before pulling in the data based upon
                        // the size of the data that we are about to read in.
                        cost_strategy.charge_instr_with_size(Opcodes::MOVE_FROM, size)?;
                    }
                    Bytecode::MoveFromGeneric(si_idx) => {
                        let addr = interpreter.operand_stack.pop_as::<AccountAddress>()?;
                        let size = interpreter.global_data_op_generic(
                            resolver,
                            data_store,
                            addr,
                            *si_idx,
                            self,
                            Interpreter::move_from,
                        )?;
                        // TODO: Have this calculate before pulling in the data based upon
                        // the size of the data that we are about to read in.
                        cost_strategy.charge_instr_with_size(Opcodes::MOVE_FROM_GENERIC, size)?;
                    }
                    Bytecode::MoveToSender(_) => {
                        return Err(VMStatus::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                            .with_message(
                                "MoveToSender is deprecated and will be removed soon".to_string(),
                            ));
                    }
                    Bytecode::MoveToSenderGeneric(_) => {
                        return Err(VMStatus::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                            .with_message(
                                "MoveToSender is deprecated and will be removed soon".to_string(),
                            ));
                    }
                    Bytecode::MoveTo(sd_idx) => {
                        let resource = interpreter.operand_stack.pop_as::<Struct>()?;
                        let signer_reference = interpreter.operand_stack.pop_as::<StructRef>()?;
                        let addr = signer_reference
                            .borrow_field(0)?
                            .value_as::<Reference>()?
                            .read_ref()?
                            .value_as::<AccountAddress>()?;
                        let size = interpreter.global_data_op(
                            resolver,
                            data_store,
                            addr,
                            *sd_idx,
                            Interpreter::move_to(resource),
                        )?;
                        cost_strategy.charge_instr_with_size(Opcodes::MOVE_TO, size)?;
                    }
                    Bytecode::MoveToGeneric(si_idx) => {
                        let resource = interpreter.operand_stack.pop_as::<Struct>()?;
                        let signer_reference = interpreter.operand_stack.pop_as::<StructRef>()?;
                        let addr = signer_reference
                            .borrow_field(0)?
                            .value_as::<Reference>()?
                            .read_ref()?
                            .value_as::<AccountAddress>()?;
                        let size = interpreter.global_data_op_generic(
                            resolver,
                            data_store,
                            addr,
                            *si_idx,
                            self,
                            Interpreter::move_to(resource),
                        )?;
                        cost_strategy.charge_instr_with_size(Opcodes::MOVE_TO_GENERIC, size)?;
                    }
                    Bytecode::FreezeRef => {
                        // FreezeRef should just be a null op as we don't distinguish between mut
                        // and immut ref at runtime.
                    }
                    Bytecode::Not => {
                        cost_strategy.charge_instr(Opcodes::NOT)?;
                        let value = !interpreter.operand_stack.pop_as::<bool>()?;
                        interpreter.operand_stack.push(Value::bool(value))?;
                    }
                    Bytecode::Nop => {
                        cost_strategy.charge_instr(Opcodes::NOP)?;
                    }
                }
            }
            // ok we are out, it's a branch, check the pc for good luck
            // TODO: re-work the logic here. Tests should have a more
            // natural way to plug in
            if self.pc as usize >= code.len() {
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

    fn ty_args(&self) -> &[Type] {
        &self.ty_args
    }

    fn resolver<'a>(&self, loader: &'a Loader) -> Resolver<'a> {
        self.function.get_resolver(loader)
    }
}
