// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    loader::{Function, Loader, Resolver},
    native_functions::NativeContext,
    trace,
};
use fail::fail_point;
use move_binary_format::{
    errors::*,
    file_format::{Bytecode, FunctionHandleIndex, FunctionInstantiationIndex},
    file_format_common::Opcodes,
};
use move_core_types::{
    account_address::AccountAddress,
    gas_schedule::{AbstractMemorySize, GasAlgebra, GasCarrier},
    vm_status::{StatusCode, StatusType},
};
use move_vm_types::{
    data_store::DataStore,
    gas_schedule::GasStatus,
    loaded_data::runtime_types::Type,
    values::{
        self, GlobalValue, IntegerValue, Locals, Reference, Struct, StructRef, VMValueCast, Value,
    },
};
use std::{cmp::min, collections::VecDeque, fmt::Write, mem, sync::Arc};
use tracing::error;

macro_rules! debug_write {
    ($($toks: tt)*) => {
        write!($($toks)*).map_err(|_|
            PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                .with_message("failed to write to buffer".to_string())
        )
    };
}

macro_rules! debug_writeln {
    ($($toks: tt)*) => {
        writeln!($($toks)*).map_err(|_|
            PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                .with_message("failed to write to buffer".to_string())
        )
    };
}

macro_rules! set_err_info {
    ($frame:ident, $e:expr) => {{
        $e.at_code_offset($frame.function.index(), $frame.pc)
            .finish($frame.location())
    }};
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
        data_store: &mut impl DataStore,
        gas_status: &mut GasStatus,
        loader: &Loader,
    ) -> VMResult<Vec<Value>> {
        // We count the intrinsic cost of the transaction here, since that needs to also cover the
        // setup of the function.
        let mut interp = Self::new();
        interp.execute(loader, data_store, gas_status, function, ty_args, args)
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
        data_store: &mut impl DataStore,
        gas_status: &mut GasStatus,
        function: Arc<Function>,
        ty_args: Vec<Type>,
        args: Vec<Value>,
    ) -> VMResult<Vec<Value>> {
        // No unwinding of the call stack and value stack need to be done here -- the context will
        // take care of that.
        self.execute_main(loader, data_store, gas_status, function, ty_args, args)
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
        data_store: &mut impl DataStore,
        gas_status: &mut GasStatus,
        function: Arc<Function>,
        ty_args: Vec<Type>,
        args: Vec<Value>,
    ) -> VMResult<Vec<Value>> {
        let mut locals = Locals::new(function.local_count());
        for (i, value) in args.into_iter().enumerate() {
            locals
                .store_loc(i, value)
                .map_err(|e| self.set_location(e))?;
        }

        let mut current_frame = Frame::new(function, ty_args, locals);
        loop {
            let resolver = current_frame.resolver(loader);
            let exit_code = current_frame //self
                .execute_code(&resolver, self, data_store, gas_status)
                .map_err(|err| self.maybe_core_dump(err, &current_frame))?;
            match exit_code {
                ExitCode::Return => {
                    if let Some(frame) = self.call_stack.pop() {
                        current_frame = frame;
                        current_frame.pc += 1; // advance past the Call instruction in the caller
                    } else {
                        return Ok(mem::take(&mut self.operand_stack.0));
                    }
                }
                ExitCode::Call(fh_idx) => {
                    gas_status
                        .charge_instr_with_size(Opcodes::CALL, AbstractMemorySize::new(1))
                        .map_err(|e| set_err_info!(current_frame, e))?;
                    let func = resolver.function_from_handle(fh_idx);
                    gas_status
                        .charge_instr_with_size(
                            Opcodes::CALL,
                            AbstractMemorySize::new(func.arg_count() as GasCarrier),
                        )
                        .map_err(|e| set_err_info!(current_frame, e))?;
                    if func.is_native() {
                        self.call_native(&resolver, data_store, gas_status, func, vec![])?;
                        current_frame.pc += 1; // advance past the Call instruction in the caller
                        continue;
                    }
                    let frame = self
                        .make_call_frame(func, vec![])
                        .map_err(|err| self.maybe_core_dump(err, &current_frame))?;
                    self.call_stack.push(current_frame).map_err(|frame| {
                        let err = PartialVMError::new(StatusCode::CALL_STACK_OVERFLOW);
                        let err = set_err_info!(frame, err);
                        self.maybe_core_dump(err, &frame)
                    })?;
                    current_frame = frame;
                }
                ExitCode::CallGeneric(idx) => {
                    let arity = resolver.type_params_count(idx);
                    gas_status
                        .charge_instr_with_size(
                            Opcodes::CALL_GENERIC,
                            AbstractMemorySize::new((arity + 1) as GasCarrier),
                        )
                        .map_err(|e| set_err_info!(current_frame, e))?;
                    let ty_args = resolver
                        .instantiate_generic_function(idx, current_frame.ty_args())
                        .map_err(|e| set_err_info!(current_frame, e))?;
                    let func = resolver.function_from_instantiation(idx);
                    gas_status
                        .charge_instr_with_size(
                            Opcodes::CALL_GENERIC,
                            AbstractMemorySize::new(func.arg_count() as GasCarrier),
                        )
                        .map_err(|e| set_err_info!(current_frame, e))?;
                    if func.is_native() {
                        self.call_native(&resolver, data_store, gas_status, func, ty_args)?;
                        current_frame.pc += 1; // advance past the Call instruction in the caller
                        continue;
                    }
                    let frame = self
                        .make_call_frame(func, ty_args)
                        .map_err(|err| self.maybe_core_dump(err, &current_frame))?;
                    self.call_stack.push(current_frame).map_err(|frame| {
                        let err = PartialVMError::new(StatusCode::CALL_STACK_OVERFLOW);
                        let err = set_err_info!(frame, err);
                        self.maybe_core_dump(err, &frame)
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
            locals
                .store_loc(
                    arg_count - i - 1,
                    self.operand_stack.pop().map_err(|e| self.set_location(e))?,
                )
                .map_err(|e| self.set_location(e))?;
        }
        Ok(Frame::new(func, ty_args, locals))
    }

    /// Call a native functions.
    fn call_native(
        &mut self,
        resolver: &Resolver,
        data_store: &mut dyn DataStore,
        gas_status: &mut GasStatus,
        function: Arc<Function>,
        ty_args: Vec<Type>,
    ) -> VMResult<()> {
        // Note: refactor if native functions push a frame on the stack
        self.call_native_impl(resolver, data_store, gas_status, function.clone(), ty_args)
            .map_err(|e| match function.module_id() {
                Some(id) => e
                    .at_code_offset(function.index(), 0)
                    .finish(Location::Module(id.clone())),
                None => {
                    let err = PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                        .with_message(
                            "Unexpected native function not located in a module".to_owned(),
                        );
                    self.set_location(err)
                }
            })
    }

    fn call_native_impl(
        &mut self,
        resolver: &Resolver,
        data_store: &mut dyn DataStore,
        gas_status: &mut GasStatus,
        function: Arc<Function>,
        ty_args: Vec<Type>,
    ) -> PartialVMResult<()> {
        let mut arguments = VecDeque::new();
        let expected_args = function.arg_count();
        for _ in 0..expected_args {
            arguments.push_front(self.operand_stack.pop()?);
        }
        let mut native_context = NativeContext::new(self, data_store, gas_status, resolver);
        let native_function = function.get_native()?;
        let result = native_function(&mut native_context, ty_args, arguments)?;
        gas_status.deduct_gas(result.cost)?;
        let return_values = result
            .result
            .map_err(|code| PartialVMError::new(StatusCode::ABORTED).with_sub_status(code))?;
        for value in return_values {
            self.operand_stack.push(value)?;
        }
        Ok(())
    }

    /// Perform a binary operation to two values at the top of the stack.
    fn binop<F, T>(&mut self, f: F) -> PartialVMResult<()>
    where
        Value: VMValueCast<T>,
        F: FnOnce(T, T) -> PartialVMResult<Value>,
    {
        let rhs = self.operand_stack.pop_as::<T>()?;
        let lhs = self.operand_stack.pop_as::<T>()?;
        let result = f(lhs, rhs)?;
        self.operand_stack.push(result)
    }

    /// Perform a binary operation for integer values.
    fn binop_int<F>(&mut self, f: F) -> PartialVMResult<()>
    where
        F: FnOnce(IntegerValue, IntegerValue) -> PartialVMResult<IntegerValue>,
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
    fn binop_bool<F, T>(&mut self, f: F) -> PartialVMResult<()>
    where
        Value: VMValueCast<T>,
        F: FnOnce(T, T) -> PartialVMResult<bool>,
    {
        self.binop(|lhs, rhs| Ok(Value::bool(f(lhs, rhs)?)))
    }

    /// Load a resource from the data store.
    fn load_resource<'b>(
        data_store: &'b mut impl DataStore,
        addr: AccountAddress,
        ty: &Type,
    ) -> PartialVMResult<&'b mut GlobalValue> {
        match data_store.load_resource(addr, ty) {
            Ok(gv) => Ok(gv),
            Err(e) => {
                error!(
                    "[VM] error loading resource at ({}, {:?}): {:?} from data store",
                    addr, ty, e
                );
                Err(e)
            }
        }
    }

    /// BorrowGlobal (mutable and not) opcode.
    fn borrow_global(
        &mut self,
        data_store: &mut impl DataStore,
        addr: AccountAddress,
        ty: &Type,
    ) -> PartialVMResult<AbstractMemorySize<GasCarrier>> {
        let g = Self::load_resource(data_store, addr, ty)?.borrow_global()?;
        let size = g.size();
        self.operand_stack.push(g)?;
        Ok(size)
    }

    /// Exists opcode.
    fn exists(
        &mut self,
        data_store: &mut impl DataStore,
        addr: AccountAddress,
        ty: &Type,
    ) -> PartialVMResult<AbstractMemorySize<GasCarrier>> {
        let gv = Self::load_resource(data_store, addr, ty)?;
        let mem_size = gv.size();
        let exists = gv.exists()?;
        self.operand_stack.push(Value::bool(exists))?;
        Ok(mem_size)
    }

    /// MoveFrom opcode.
    fn move_from(
        &mut self,
        data_store: &mut impl DataStore,
        addr: AccountAddress,
        ty: &Type,
    ) -> PartialVMResult<AbstractMemorySize<GasCarrier>> {
        let resource = Self::load_resource(data_store, addr, ty)?.move_from()?;
        let size = resource.size();
        self.operand_stack.push(resource)?;
        Ok(size)
    }

    /// MoveTo opcode.
    fn move_to(
        &mut self,
        data_store: &mut impl DataStore,
        addr: AccountAddress,
        ty: &Type,
        resource: Value,
    ) -> PartialVMResult<AbstractMemorySize<GasCarrier>> {
        let size = resource.size();
        Self::load_resource(data_store, addr, ty)?.move_to(resource)?;
        Ok(size)
    }

    //
    // Debugging and logging helpers.
    //

    /// Given an `VMStatus` generate a core dump if the error is an `InvariantViolation`.
    fn maybe_core_dump(&self, mut err: VMError, current_frame: &Frame) -> VMError {
        // a verification error cannot happen at runtime so change it into an invariant violation.
        if err.status_type() == StatusType::Verification {
            error!("Verification error during runtime: {:?}", err);
            let new_err = PartialVMError::new(StatusCode::VERIFICATION_ERROR);
            let new_err = match err.message() {
                None => new_err,
                Some(msg) => new_err.with_message(msg.to_owned()),
            };
            err = new_err.finish(err.location().clone())
        }
        if err.status_type() == StatusType::InvariantViolation {
            let state = self.get_internal_state(current_frame);

            error!(
                "Error: {:?}\nCORE DUMP: >>>>>>>>>>>>\n{}\n<<<<<<<<<<<<\n",
                err, state,
            );
        }
        err
    }

    #[allow(dead_code)]
    fn debug_print_frame<B: Write>(
        &self,
        buf: &mut B,
        loader: &Loader,
        idx: usize,
        frame: &Frame,
    ) -> PartialVMResult<()> {
        // Print out the function name with type arguments.
        let func = &frame.function;

        debug_write!(buf, "    [{}] ", idx)?;
        if let Some(module) = func.module_id() {
            debug_write!(buf, "{}::{}::", module.address(), module.name(),)?;
        }
        debug_write!(buf, "{}", func.name())?;
        let ty_args = frame.ty_args();
        let mut ty_tags = vec![];
        for ty in ty_args {
            ty_tags.push(loader.type_to_type_tag(ty)?);
        }
        if !ty_tags.is_empty() {
            debug_write!(buf, "<")?;
            let mut it = ty_tags.iter();
            if let Some(tag) = it.next() {
                debug_write!(buf, "{}", tag)?;
                for tag in it {
                    debug_write!(buf, ", ")?;
                    debug_write!(buf, "{}", tag)?;
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
            values::debug::print_locals(buf, &frame.locals)?;
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
        loader: &Loader,
    ) -> PartialVMResult<()> {
        debug_writeln!(buf, "Call Stack:")?;
        for (i, frame) in self.call_stack.0.iter().enumerate() {
            self.debug_print_frame(buf, loader, i, frame)?;
        }
        debug_writeln!(buf, "Operand Stack:")?;
        for (idx, val) in self.operand_stack.0.iter().enumerate() {
            // TODO: Currently we do not know the types of the values on the operand stack.
            // Revisit.
            debug_write!(buf, "    [{}] ", idx)?;
            values::debug::print_value(buf, val)?;
            debug_writeln!(buf)?;
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

    fn set_location(&self, err: PartialVMError) -> VMError {
        err.finish(self.call_stack.current_location())
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
    fn push(&mut self, value: Value) -> PartialVMResult<()> {
        if self.0.len() < OPERAND_STACK_SIZE_LIMIT {
            self.0.push(value);
            Ok(())
        } else {
            Err(PartialVMError::new(StatusCode::EXECUTION_STACK_OVERFLOW))
        }
    }

    /// Pop a `Value` off the stack or abort execution if the stack is empty.
    fn pop(&mut self) -> PartialVMResult<Value> {
        self.0
            .pop()
            .ok_or_else(|| PartialVMError::new(StatusCode::EMPTY_VALUE_STACK))
    }

    /// Pop a `Value` of a given type off the stack. Abort if the value is not of the given
    /// type or if the stack is empty.
    fn pop_as<T>(&mut self) -> PartialVMResult<T>
    where
        Value: VMValueCast<T>,
    {
        self.pop()?.value_as()
    }

    /// Pop n values off the stack.
    fn popn(&mut self, n: u16) -> PartialVMResult<Vec<Value>> {
        let remaining_stack_size = self
            .0
            .len()
            .checked_sub(n as usize)
            .ok_or_else(|| PartialVMError::new(StatusCode::EMPTY_VALUE_STACK))?;
        let args = self.0.split_off(remaining_stack_size);
        Ok(args)
    }
}

/// A call stack.
// #[derive(Debug)]
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

    fn current_location(&self) -> Location {
        let location_opt = self.0.last().map(|frame| frame.location());
        location_opt.unwrap_or(Location::Undefined)
    }
}

/// A `Frame` is the execution context for a function. It holds the locals of the function and
/// the function itself.
// #[derive(Debug)]
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
        data_store: &mut impl DataStore,
        gas_status: &mut GasStatus,
    ) -> VMResult<ExitCode> {
        self.execute_code_impl(resolver, interpreter, data_store, gas_status)
            .map_err(|e| {
                e.at_code_offset(self.function.index(), self.pc)
                    .finish(self.location())
            })
    }

    fn execute_code_impl(
        &mut self,
        resolver: &Resolver,
        interpreter: &mut Interpreter,
        data_store: &mut impl DataStore,
        gas_status: &mut GasStatus,
    ) -> PartialVMResult<ExitCode> {
        let code = self.function.code();
        loop {
            for instruction in &code[self.pc as usize..] {
                trace!(
                    &self.function,
                    &self.locals,
                    self.pc,
                    instruction,
                    resolver,
                    interpreter
                );

                fail_point!("move_vm::interpreter_loop", |_| {
                    Err(
                        PartialVMError::new(StatusCode::VERIFIER_INVARIANT_VIOLATION).with_message(
                            "Injected move_vm::interpreter verifier failure".to_owned(),
                        ),
                    )
                });

                match instruction {
                    Bytecode::Pop => {
                        gas_status.charge_instr(Opcodes::POP)?;
                        interpreter.operand_stack.pop()?;
                    }
                    Bytecode::Ret => {
                        gas_status.charge_instr(Opcodes::RET)?;
                        return Ok(ExitCode::Return);
                    }
                    Bytecode::BrTrue(offset) => {
                        gas_status.charge_instr(Opcodes::BR_TRUE)?;
                        if interpreter.operand_stack.pop_as::<bool>()? {
                            self.pc = *offset;
                            break;
                        }
                    }
                    Bytecode::BrFalse(offset) => {
                        gas_status.charge_instr(Opcodes::BR_FALSE)?;
                        if !interpreter.operand_stack.pop_as::<bool>()? {
                            self.pc = *offset;
                            break;
                        }
                    }
                    Bytecode::Branch(offset) => {
                        gas_status.charge_instr(Opcodes::BRANCH)?;
                        self.pc = *offset;
                        break;
                    }
                    Bytecode::LdU8(int_const) => {
                        gas_status.charge_instr(Opcodes::LD_U8)?;
                        interpreter.operand_stack.push(Value::u8(*int_const))?;
                    }
                    Bytecode::LdU64(int_const) => {
                        gas_status.charge_instr(Opcodes::LD_U64)?;
                        interpreter.operand_stack.push(Value::u64(*int_const))?;
                    }
                    Bytecode::LdU128(int_const) => {
                        gas_status.charge_instr(Opcodes::LD_U128)?;
                        interpreter.operand_stack.push(Value::u128(*int_const))?;
                    }
                    Bytecode::LdConst(idx) => {
                        let constant = resolver.constant_at(*idx);
                        gas_status.charge_instr_with_size(
                            Opcodes::LD_CONST,
                            AbstractMemorySize::new(constant.data.len() as GasCarrier),
                        )?;
                        interpreter.operand_stack.push(
                            Value::deserialize_constant(constant).ok_or_else(|| {
                                PartialVMError::new(StatusCode::VERIFIER_INVARIANT_VIOLATION)
                                    .with_message(
                                    "Verifier failed to verify the deserialization of constants"
                                        .to_owned(),
                                )
                            })?,
                        )?
                    }
                    Bytecode::LdTrue => {
                        gas_status.charge_instr(Opcodes::LD_TRUE)?;
                        interpreter.operand_stack.push(Value::bool(true))?;
                    }
                    Bytecode::LdFalse => {
                        gas_status.charge_instr(Opcodes::LD_FALSE)?;
                        interpreter.operand_stack.push(Value::bool(false))?;
                    }
                    Bytecode::CopyLoc(idx) => {
                        let local = self.locals.copy_loc(*idx as usize)?;
                        gas_status.charge_instr_with_size(Opcodes::COPY_LOC, local.size())?;
                        interpreter.operand_stack.push(local)?;
                    }
                    Bytecode::MoveLoc(idx) => {
                        let local = self.locals.move_loc(*idx as usize)?;
                        gas_status.charge_instr_with_size(Opcodes::MOVE_LOC, local.size())?;

                        interpreter.operand_stack.push(local)?;
                    }
                    Bytecode::StLoc(idx) => {
                        let value_to_store = interpreter.operand_stack.pop()?;
                        gas_status
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
                        gas_status.charge_instr(opcode)?;
                        interpreter
                            .operand_stack
                            .push(self.locals.borrow_loc(*idx as usize)?)?;
                    }
                    Bytecode::ImmBorrowField(fh_idx) | Bytecode::MutBorrowField(fh_idx) => {
                        let opcode = match instruction {
                            Bytecode::MutBorrowField(_) => Opcodes::MUT_BORROW_FIELD,
                            _ => Opcodes::IMM_BORROW_FIELD,
                        };
                        gas_status.charge_instr(opcode)?;

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
                        gas_status.charge_instr(opcode)?;

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
                        gas_status.charge_instr_with_size(Opcodes::PACK, size)?;
                        interpreter
                            .operand_stack
                            .push(Value::struct_(Struct::pack(args)))?;
                    }
                    Bytecode::PackGeneric(si_idx) => {
                        let field_count = resolver.field_instantiation_count(*si_idx);
                        let args = interpreter.operand_stack.popn(field_count)?;
                        let size = args.iter().fold(
                            AbstractMemorySize::new(GasCarrier::from(field_count)),
                            |acc, v| acc.add(v.size()),
                        );
                        gas_status.charge_instr_with_size(Opcodes::PACK_GENERIC, size)?;
                        interpreter
                            .operand_stack
                            .push(Value::struct_(Struct::pack(args)))?;
                    }
                    Bytecode::Unpack(sd_idx) => {
                        let field_count = resolver.field_count(*sd_idx);
                        let struct_ = interpreter.operand_stack.pop_as::<Struct>()?;
                        gas_status.charge_instr_with_size(
                            Opcodes::UNPACK,
                            AbstractMemorySize::new(GasCarrier::from(field_count)),
                        )?;
                        // TODO: Whether or not we want this gas metering in the loop is
                        // questionable.  However, if we don't have it in the loop we could wind up
                        // doing a fair bit of work before charging for it.
                        for value in struct_.unpack()? {
                            gas_status.charge_instr_with_size(Opcodes::UNPACK, value.size())?;
                            interpreter.operand_stack.push(value)?;
                        }
                    }
                    Bytecode::UnpackGeneric(si_idx) => {
                        let field_count = resolver.field_instantiation_count(*si_idx);
                        let struct_ = interpreter.operand_stack.pop_as::<Struct>()?;
                        gas_status.charge_instr_with_size(
                            Opcodes::UNPACK_GENERIC,
                            AbstractMemorySize::new(GasCarrier::from(field_count)),
                        )?;
                        // TODO: Whether or not we want this gas metering in the loop is
                        // questionable.  However, if we don't have it in the loop we could wind up
                        // doing a fair bit of work before charging for it.
                        for value in struct_.unpack()? {
                            gas_status
                                .charge_instr_with_size(Opcodes::UNPACK_GENERIC, value.size())?;
                            interpreter.operand_stack.push(value)?;
                        }
                    }
                    Bytecode::ReadRef => {
                        let reference = interpreter.operand_stack.pop_as::<Reference>()?;
                        let value = reference.read_ref()?;
                        gas_status.charge_instr_with_size(Opcodes::READ_REF, value.size())?;
                        interpreter.operand_stack.push(value)?;
                    }
                    Bytecode::WriteRef => {
                        let reference = interpreter.operand_stack.pop_as::<Reference>()?;
                        let value = interpreter.operand_stack.pop()?;
                        gas_status.charge_instr_with_size(Opcodes::WRITE_REF, value.size())?;
                        reference.write_ref(value)?;
                    }
                    Bytecode::CastU8 => {
                        gas_status.charge_instr(Opcodes::CAST_U8)?;
                        let integer_value = interpreter.operand_stack.pop_as::<IntegerValue>()?;
                        interpreter
                            .operand_stack
                            .push(Value::u8(integer_value.cast_u8()?))?;
                    }
                    Bytecode::CastU64 => {
                        gas_status.charge_instr(Opcodes::CAST_U64)?;
                        let integer_value = interpreter.operand_stack.pop_as::<IntegerValue>()?;
                        interpreter
                            .operand_stack
                            .push(Value::u64(integer_value.cast_u64()?))?;
                    }
                    Bytecode::CastU128 => {
                        gas_status.charge_instr(Opcodes::CAST_U128)?;
                        let integer_value = interpreter.operand_stack.pop_as::<IntegerValue>()?;
                        interpreter
                            .operand_stack
                            .push(Value::u128(integer_value.cast_u128()?))?;
                    }
                    // Arithmetic Operations
                    Bytecode::Add => {
                        gas_status.charge_instr(Opcodes::ADD)?;
                        interpreter.binop_int(IntegerValue::add_checked)?
                    }
                    Bytecode::Sub => {
                        gas_status.charge_instr(Opcodes::SUB)?;
                        interpreter.binop_int(IntegerValue::sub_checked)?
                    }
                    Bytecode::Mul => {
                        gas_status.charge_instr(Opcodes::MUL)?;
                        interpreter.binop_int(IntegerValue::mul_checked)?
                    }
                    Bytecode::Mod => {
                        gas_status.charge_instr(Opcodes::MOD)?;
                        interpreter.binop_int(IntegerValue::rem_checked)?
                    }
                    Bytecode::Div => {
                        gas_status.charge_instr(Opcodes::DIV)?;
                        interpreter.binop_int(IntegerValue::div_checked)?
                    }
                    Bytecode::BitOr => {
                        gas_status.charge_instr(Opcodes::BIT_OR)?;
                        interpreter.binop_int(IntegerValue::bit_or)?
                    }
                    Bytecode::BitAnd => {
                        gas_status.charge_instr(Opcodes::BIT_AND)?;
                        interpreter.binop_int(IntegerValue::bit_and)?
                    }
                    Bytecode::Xor => {
                        gas_status.charge_instr(Opcodes::XOR)?;
                        interpreter.binop_int(IntegerValue::bit_xor)?
                    }
                    Bytecode::Shl => {
                        gas_status.charge_instr(Opcodes::SHL)?;
                        let rhs = interpreter.operand_stack.pop_as::<u8>()?;
                        let lhs = interpreter.operand_stack.pop_as::<IntegerValue>()?;
                        interpreter
                            .operand_stack
                            .push(lhs.shl_checked(rhs)?.into_value())?;
                    }
                    Bytecode::Shr => {
                        gas_status.charge_instr(Opcodes::SHR)?;
                        let rhs = interpreter.operand_stack.pop_as::<u8>()?;
                        let lhs = interpreter.operand_stack.pop_as::<IntegerValue>()?;
                        interpreter
                            .operand_stack
                            .push(lhs.shr_checked(rhs)?.into_value())?;
                    }
                    Bytecode::Or => {
                        gas_status.charge_instr(Opcodes::OR)?;
                        interpreter.binop_bool(|l, r| Ok(l || r))?
                    }
                    Bytecode::And => {
                        gas_status.charge_instr(Opcodes::AND)?;
                        interpreter.binop_bool(|l, r| Ok(l && r))?
                    }
                    Bytecode::Lt => {
                        gas_status.charge_instr(Opcodes::LT)?;
                        interpreter.binop_bool(IntegerValue::lt)?
                    }
                    Bytecode::Gt => {
                        gas_status.charge_instr(Opcodes::GT)?;
                        interpreter.binop_bool(IntegerValue::gt)?
                    }
                    Bytecode::Le => {
                        gas_status.charge_instr(Opcodes::LE)?;
                        interpreter.binop_bool(IntegerValue::le)?
                    }
                    Bytecode::Ge => {
                        gas_status.charge_instr(Opcodes::GE)?;
                        interpreter.binop_bool(IntegerValue::ge)?
                    }
                    Bytecode::Abort => {
                        gas_status.charge_instr(Opcodes::ABORT)?;
                        let error_code = interpreter.operand_stack.pop_as::<u64>()?;
                        return Err(PartialVMError::new(StatusCode::ABORTED)
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
                        gas_status
                            .charge_instr_with_size(Opcodes::EQ, lhs.size().add(rhs.size()))?;
                        interpreter
                            .operand_stack
                            .push(Value::bool(lhs.equals(&rhs)?))?;
                    }
                    Bytecode::Neq => {
                        let lhs = interpreter.operand_stack.pop()?;
                        let rhs = interpreter.operand_stack.pop()?;
                        gas_status
                            .charge_instr_with_size(Opcodes::NEQ, lhs.size().add(rhs.size()))?;
                        interpreter
                            .operand_stack
                            .push(Value::bool(!lhs.equals(&rhs)?))?;
                    }
                    Bytecode::MutBorrowGlobal(sd_idx) | Bytecode::ImmBorrowGlobal(sd_idx) => {
                        let addr = interpreter.operand_stack.pop_as::<AccountAddress>()?;
                        let ty = resolver.get_struct_type(*sd_idx);
                        let size = interpreter.borrow_global(data_store, addr, &ty)?;
                        gas_status.charge_instr_with_size(Opcodes::MUT_BORROW_GLOBAL, size)?;
                    }
                    Bytecode::MutBorrowGlobalGeneric(si_idx)
                    | Bytecode::ImmBorrowGlobalGeneric(si_idx) => {
                        let addr = interpreter.operand_stack.pop_as::<AccountAddress>()?;
                        let ty = resolver.instantiate_generic_type(*si_idx, self.ty_args())?;
                        let size = interpreter.borrow_global(data_store, addr, &ty)?;
                        gas_status
                            .charge_instr_with_size(Opcodes::MUT_BORROW_GLOBAL_GENERIC, size)?;
                    }
                    Bytecode::Exists(sd_idx) => {
                        let addr = interpreter.operand_stack.pop_as::<AccountAddress>()?;
                        let ty = resolver.get_struct_type(*sd_idx);
                        let size = interpreter.exists(data_store, addr, &ty)?;
                        gas_status.charge_instr_with_size(Opcodes::EXISTS, size)?;
                    }
                    Bytecode::ExistsGeneric(si_idx) => {
                        let addr = interpreter.operand_stack.pop_as::<AccountAddress>()?;
                        let ty = resolver.instantiate_generic_type(*si_idx, self.ty_args())?;
                        let size = interpreter.exists(data_store, addr, &ty)?;
                        gas_status.charge_instr_with_size(Opcodes::EXISTS_GENERIC, size)?;
                    }
                    Bytecode::MoveFrom(sd_idx) => {
                        let addr = interpreter.operand_stack.pop_as::<AccountAddress>()?;
                        let ty = resolver.get_struct_type(*sd_idx);
                        let size = interpreter.move_from(data_store, addr, &ty)?;
                        // TODO: Have this calculate before pulling in the data based upon
                        // the size of the data that we are about to read in.
                        gas_status.charge_instr_with_size(Opcodes::MOVE_FROM, size)?;
                    }
                    Bytecode::MoveFromGeneric(si_idx) => {
                        let addr = interpreter.operand_stack.pop_as::<AccountAddress>()?;
                        let ty = resolver.instantiate_generic_type(*si_idx, self.ty_args())?;
                        let size = interpreter.move_from(data_store, addr, &ty)?;
                        // TODO: Have this calculate before pulling in the data based upon
                        // the size of the data that we are about to read in.
                        gas_status.charge_instr_with_size(Opcodes::MOVE_FROM_GENERIC, size)?;
                    }
                    Bytecode::MoveTo(sd_idx) => {
                        let resource = interpreter.operand_stack.pop()?;
                        let signer_reference = interpreter.operand_stack.pop_as::<StructRef>()?;
                        let addr = signer_reference
                            .borrow_field(0)?
                            .value_as::<Reference>()?
                            .read_ref()?
                            .value_as::<AccountAddress>()?;
                        let ty = resolver.get_struct_type(*sd_idx);
                        // REVIEW: Can we simplify Interpreter::move_to?
                        let size = interpreter.move_to(data_store, addr, &ty, resource)?;
                        gas_status.charge_instr_with_size(Opcodes::MOVE_TO, size)?;
                    }
                    Bytecode::MoveToGeneric(si_idx) => {
                        let resource = interpreter.operand_stack.pop()?;
                        let signer_reference = interpreter.operand_stack.pop_as::<StructRef>()?;
                        let addr = signer_reference
                            .borrow_field(0)?
                            .value_as::<Reference>()?
                            .read_ref()?
                            .value_as::<AccountAddress>()?;
                        let ty = resolver.instantiate_generic_type(*si_idx, self.ty_args())?;
                        let size = interpreter.move_to(data_store, addr, &ty, resource)?;
                        gas_status.charge_instr_with_size(Opcodes::MOVE_TO_GENERIC, size)?;
                    }
                    Bytecode::FreezeRef => {
                        gas_status.charge_instr(Opcodes::FREEZE_REF)?;
                        // FreezeRef should just be a null op as we don't distinguish between mut
                        // and immut ref at runtime.
                    }
                    Bytecode::Not => {
                        gas_status.charge_instr(Opcodes::NOT)?;
                        let value = !interpreter.operand_stack.pop_as::<bool>()?;
                        interpreter.operand_stack.push(Value::bool(value))?;
                    }
                    Bytecode::Nop => {
                        gas_status.charge_instr(Opcodes::NOP)?;
                    }
                }
                // invariant: advance to pc +1 is iff instruction at pc executed without aborting
                self.pc += 1;
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
                    return Err(PartialVMError::new(StatusCode::PC_OVERFLOW));
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

    fn location(&self) -> Location {
        match self.function.module_id() {
            None => Location::Script,
            Some(id) => Location::Module(id.clone()),
        }
    }
}
