// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
//! Processor for a single transaction.

use crate::{
    code_cache::module_cache::{ModuleCache, VMModuleCache},
    counters::*,
    data_cache::{RemoteCache, TransactionDataCache},
    execution_stack::ExecutionStack,
    gas_meter::GasMeter,
    identifier::{create_access_path, resource_storage_key},
    loaded_data::{
        function::{FunctionRef, FunctionReference},
        loaded_module::LoadedModule,
    },
};
use bytecode_verifier::{VerifiedModule, VerifiedScript};
use std::{collections::VecDeque, convert::TryFrom};
use types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_config,
    byte_array::ByteArray,
    contract_event::ContractEvent,
    event::EventKey,
    identifier::{IdentStr, Identifier},
    language_storage::ModuleId,
    transaction::{
        TransactionArgument, TransactionOutput, TransactionStatus, MAX_TRANSACTION_SIZE_IN_BYTES,
    },
    vm_error::{StatusCode, StatusType, VMStatus},
    write_set::WriteSet,
};
use vm::{
    access::ModuleAccess,
    errors::*,
    file_format::{Bytecode, CodeOffset, CompiledScript, StructDefinitionIndex},
    gas_schedule::{AbstractMemorySize, GasAlgebra, GasUnits},
    transaction_metadata::TransactionMetadata,
    vm_string::VMString,
};
use vm_cache_map::Arena;
use vm_runtime_types::{
    native_functions::dispatch::{dispatch_native_function, NativeReturnStatus},
    value::{ReferenceValue, Struct, Value},
};

// Metadata needed for resolving the account module.
lazy_static! {
    /// The ModuleId for the Account module
    pub static ref ACCOUNT_MODULE: ModuleId =
        { ModuleId::new(account_config::core_code_address(), Identifier::new("LibraAccount").unwrap()) };
    /// The ModuleId for the Account module
    pub static ref BLOCK_MODULE: ModuleId =
        { ModuleId::new(account_config::core_code_address(), Identifier::new("Block").unwrap()) };
    /// The ModuleId for the LibraCoin module
    pub static ref COIN_MODULE: ModuleId =
        { ModuleId::new(account_config::core_code_address(), Identifier::new("LibraCoin").unwrap()) };
    /// The ModuleId for the Event
    pub static ref EVENT_MODULE: ModuleId =
        { ModuleId::new(account_config::core_code_address(), Identifier::new("Event").unwrap()) };

    /// The ModuleId for the validator set
    pub static ref VALIDATOR_SET_MODULE: ModuleId =
        { ModuleId::new(account_config::core_code_address(), Identifier::new("ValidatorSet").unwrap()) };
}

// Names for special functions.
lazy_static! {
    static ref PROLOGUE_NAME: Identifier = Identifier::new("prologue").unwrap();
    static ref EPILOGUE_NAME: Identifier = Identifier::new("epilogue").unwrap();
    static ref CREATE_ACCOUNT_NAME: Identifier = Identifier::new("make").unwrap();
    static ref ACCOUNT_STRUCT_NAME: Identifier = Identifier::new("T").unwrap();
    static ref EMIT_EVENT_NAME: Identifier = Identifier::new("write_to_event_store").unwrap();
}

fn make_access_path(
    module: &impl ModuleAccess,
    idx: StructDefinitionIndex,
    address: AccountAddress,
) -> AccessPath {
    let struct_tag = resource_storage_key(module, idx);
    create_access_path(&address, struct_tag)
}

/// A struct that executes one single transaction.
/// 'alloc is the lifetime for the code cache, which is the argument type P here. Hence the P should
/// live as long as alloc.
/// 'txn is the lifetime of one single transaction.
/// `execution_stack` contains the call stack and value stack of current execution.
/// `txn_data` contains the information of this transaction, such as sender, sequence number, etc.
/// `event_data` is the vector that stores all events emitted during execution.
/// `data_view` is the scratchpad for the local writes emitted by this transaction.
pub struct TransactionExecutor<'alloc, 'txn, P>
where
    'alloc: 'txn,
    P: ModuleCache<'alloc>,
{
    #[cfg(any(test, feature = "instruction_synthesis"))]
    pub execution_stack: ExecutionStack<'alloc, 'txn, P>,

    #[cfg(not(any(test, feature = "instruction_synthesis")))]
    execution_stack: ExecutionStack<'alloc, 'txn, P>,
    gas_meter: GasMeter,
    txn_data: TransactionMetadata,
    event_data: Vec<ContractEvent>,
    data_view: TransactionDataCache<'txn>,
}

impl<'alloc, 'txn, P> TransactionExecutor<'alloc, 'txn, P>
where
    'alloc: 'txn,
    P: ModuleCache<'alloc>,
{
    /// Create a new `TransactionExecutor` to execute a single transaction. `module_cache` is the
    /// cache that stores the modules previously read from the blockchain. `data_cache` is the cache
    /// that holds read-only connection to the state store as well as the changes made by previous
    /// transactions within the same block.
    pub fn new(
        module_cache: P,
        data_cache: &'txn dyn RemoteCache,
        txn_data: TransactionMetadata,
    ) -> Self {
        TransactionExecutor {
            execution_stack: ExecutionStack::new(module_cache),
            gas_meter: GasMeter::new(txn_data.max_gas_amount()),
            txn_data,
            event_data: Vec::new(),
            data_view: TransactionDataCache::new(data_cache),
        }
    }

    /// Returns the module cache for this executor.
    pub fn module_cache(&self) -> &P {
        &self.execution_stack.module_cache
    }

    /// Perform a binary operation to two values at the top of the stack.
    fn binop<F, T>(&mut self, f: F) -> VMResult<()>
    where
        Option<T>: From<Value>,
        F: FnOnce(T, T) -> Option<Value>,
    {
        let rhs = self.execution_stack.pop_as::<T>()?;
        let lhs = self.execution_stack.pop_as::<T>()?;
        let result = f(lhs, rhs);
        if let Some(v) = result {
            self.execution_stack.push(v)?;
            Ok(())
        } else {
            Err(vm_error(
                self.execution_stack.location()?,
                StatusCode::ARITHMETIC_ERROR,
            ))
        }
    }

    fn binop_int<F, T>(&mut self, f: F) -> VMResult<()>
    where
        Option<T>: From<Value>,
        F: FnOnce(T, T) -> Option<u64>,
    {
        self.binop(|lhs, rhs| f(lhs, rhs).map(Value::u64))
    }

    fn binop_bool<F, T>(&mut self, f: F) -> VMResult<()>
    where
        Option<T>: From<Value>,
        F: FnOnce(T, T) -> bool,
    {
        self.binop(|lhs, rhs| Some(Value::bool(f(lhs, rhs))))
    }

    /// This function will execute the code sequence starting from the beginning_offset, and return
    /// Ok(offset) when the instruction sequence hit a branch, either by calling into a new
    /// function, branches, function return, etc. The return value will be the pc for the next
    /// instruction to be executed.
    #[allow(clippy::cognitive_complexity)]
    pub fn execute_block(
        &mut self,
        code: &[Bytecode],
        beginning_offset: CodeOffset,
    ) -> VMResult<CodeOffset> {
        let mut pc = beginning_offset;
        for instruction in &code[beginning_offset as usize..] {
            // FIXME: Once we add in memory ops, we will need to pass in the current memory size to
            // this function.
            self.gas_meter.calculate_and_consume(
                &instruction,
                &self.execution_stack,
                AbstractMemorySize::new(1),
            )?;

            match instruction {
                Bytecode::Pop => {
                    self.execution_stack.pop()?;
                }
                Bytecode::Ret => {
                    self.execution_stack.pop_call()?;
                    if self.execution_stack.is_call_stack_empty() {
                        return Ok(0);
                    } else {
                        return Ok(self.execution_stack.top_frame()?.get_pc() + 1);
                    }
                }
                Bytecode::BrTrue(offset) => {
                    if self.execution_stack.pop_as::<bool>()? {
                        return Ok(*offset);
                    }
                }
                Bytecode::BrFalse(offset) => {
                    let stack_top = self.execution_stack.pop_as::<bool>()?;
                    if !stack_top {
                        return Ok(*offset);
                    }
                }
                Bytecode::Branch(offset) => return Ok(*offset),
                Bytecode::LdConst(int_const) => {
                    self.execution_stack.push(Value::u64(*int_const))?;
                }
                Bytecode::LdAddr(idx) => {
                    let top_frame = self.execution_stack.top_frame()?;
                    let addr_ref = top_frame.module().address_at(*idx);
                    self.execution_stack.push(Value::address(*addr_ref))?;
                }
                Bytecode::LdStr(idx) => {
                    let top_frame = self.execution_stack.top_frame()?;
                    let string_ref = top_frame.module().user_string_at(*idx);
                    self.execution_stack
                        .push(Value::string(string_ref.into()))?;
                }
                Bytecode::LdByteArray(idx) => {
                    let top_frame = self.execution_stack.top_frame()?;
                    let byte_array = top_frame.module().byte_array_at(*idx);
                    self.execution_stack
                        .push(Value::byte_array(byte_array.clone()))?;
                }
                Bytecode::LdTrue => {
                    self.execution_stack.push(Value::bool(true))?;
                }
                Bytecode::LdFalse => {
                    self.execution_stack.push(Value::bool(false))?;
                }
                Bytecode::CopyLoc(idx) => {
                    let value = self.execution_stack.top_frame()?.copy_loc(*idx)?;
                    self.execution_stack.push(value)?;
                }
                Bytecode::MoveLoc(idx) => {
                    let value = self.execution_stack.top_frame_mut()?.move_loc(*idx)?;
                    self.execution_stack.push(value)?;
                }
                Bytecode::StLoc(idx) => {
                    let value = self.execution_stack.pop()?;
                    self.execution_stack
                        .top_frame_mut()?
                        .store_loc(*idx, value)?;
                }
                Bytecode::Call(idx, _) => {
                    let self_module = &self.execution_stack.top_frame()?.module();
                    let callee_function_ref = self
                        .execution_stack
                        .module_cache
                        .resolve_function_ref(self_module, *idx)?
                        .ok_or_else(|| VMStatus::new(StatusCode::LINKER_ERROR))?;

                    if callee_function_ref.is_native() {
                        let module = callee_function_ref.module();
                        let module_id = module.self_id();
                        let function_name = callee_function_ref.name();
                        let native_function =
                            match dispatch_native_function(&module_id, function_name) {
                                None => return Err(VMStatus::new(StatusCode::LINKER_ERROR)),
                                Some(native_function) => native_function,
                            };
                        if module_id == *EVENT_MODULE
                            && function_name == EMIT_EVENT_NAME.as_ident_str()
                        {
                            let msg = self
                                .execution_stack
                                .pop()?
                                .simple_serialize()
                                .ok_or_else(|| VMStatus::new(StatusCode::DATA_FORMAT_ERROR))?;
                            let count = self.execution_stack.pop_as::<u64>()?;
                            let key = self.execution_stack.pop_as::<ByteArray>()?;
                            let guid = EventKey::try_from(key.as_bytes())
                                .map_err(|_| VMStatus::new(StatusCode::EVENT_KEY_MISMATCH))?;

                            // TODO:
                            // 1. Rename the AccessPath here to a new type that represents such
                            //    globally unique id for event streams.
                            // 2. Charge gas for the msg emitted.
                            self.event_data.push(ContractEvent::new(guid, count, msg))
                        } else {
                            let mut arguments = VecDeque::new();
                            let expected_args = native_function.num_args();
                            if callee_function_ref.arg_count() != expected_args {
                                // Should not be possible due to bytecode verifier but this
                                // assertion is here to make sure
                                // the view the type checker had lines up with the
                                // execution of the native function
                                return Err(VMStatus::new(StatusCode::LINKER_ERROR));
                            }
                            for _ in 0..expected_args {
                                arguments.push_front(self.execution_stack.pop()?);
                            }
                            let (cost, return_values) = match (native_function.dispatch)(arguments)
                            {
                                NativeReturnStatus::InvalidArguments => {
                                    // TODO: better error
                                    return Err(VMStatus::new(StatusCode::LINKER_ERROR));
                                }
                                NativeReturnStatus::Aborted { cost, error_code } => {
                                    self.gas_meter
                                        .consume_gas(GasUnits::new(cost), &self.execution_stack)?;
                                    return Err(vm_error(
                                        self.execution_stack.location()?,
                                        StatusCode::NATIVE_FUNCTION_ERROR,
                                    )
                                    .with_sub_status(error_code));
                                }
                                NativeReturnStatus::Success {
                                    cost,
                                    return_values,
                                } => (cost, return_values),
                            };
                            self.gas_meter
                                .consume_gas(GasUnits::new(cost), &self.execution_stack)?;
                            for value in return_values {
                                self.execution_stack.push(value)?;
                            }
                        }
                    // Call stack is not reconstructed for a native call, so we just
                    // proceed on to next instruction.
                    } else {
                        self.execution_stack.top_frame_mut()?.save_pc(pc);
                        self.execution_stack.push_call(callee_function_ref)?;
                        // Call stack is reconstructed, the next instruction to execute will be the
                        // first instruction of the callee function. Thus we should break here to
                        // restart the instruction sequence from there.
                        return Ok(0);
                    }
                }
                Bytecode::MutBorrowLoc(idx) | Bytecode::ImmBorrowLoc(idx) => {
                    let local_ref = self.execution_stack.top_frame_mut()?.borrow_loc(*idx)?;
                    self.execution_stack.push(local_ref)?;
                }
                Bytecode::ImmBorrowField(fd_idx) | Bytecode::MutBorrowField(fd_idx) => {
                    let field_offset = self
                        .execution_stack
                        .top_frame()?
                        .module()
                        .get_field_offset(*fd_idx)?;
                    let reference = self.execution_stack.pop_as::<ReferenceValue>()?;
                    let field_ref = reference.borrow_field(field_offset as usize)?;
                    self.execution_stack.push(field_ref)?;
                }
                Bytecode::Pack(sd_idx, _) => {
                    let self_module = self.execution_stack.top_frame()?.module();
                    let struct_def = self_module.struct_def_at(*sd_idx);
                    let field_count = struct_def.declared_field_count()?;
                    let args = self.execution_stack.popn(field_count)?;
                    self.execution_stack
                        .push(Value::struct_(Struct::new(args)))?;
                }
                Bytecode::Unpack(sd_idx, _) => {
                    let self_module = self.execution_stack.top_frame()?.module();
                    let struct_def = self_module.struct_def_at(*sd_idx);
                    let field_count = struct_def.declared_field_count()?;
                    let struct_ = self.execution_stack.pop_as::<Struct>()?;
                    for idx in 0..field_count {
                        self.execution_stack
                            .push(struct_.get_field_value(idx as usize)?)?;
                    }
                }
                Bytecode::ReadRef => {
                    let reference = self.execution_stack.pop_as::<ReferenceValue>()?;
                    let value = reference.read_ref()?;
                    self.execution_stack.push(value)?;
                }
                Bytecode::WriteRef => {
                    let reference = self.execution_stack.pop_as::<ReferenceValue>()?;
                    let value = self.execution_stack.pop()?;
                    reference.write_ref(value);
                }
                // Arithmetic Operations
                Bytecode::Add => self.binop_int(u64::checked_add)?,
                Bytecode::Sub => self.binop_int(u64::checked_sub)?,
                Bytecode::Mul => self.binop_int(u64::checked_mul)?,
                Bytecode::Mod => self.binop_int(u64::checked_rem)?,
                Bytecode::Div => self.binop_int(u64::checked_div)?,
                Bytecode::BitOr => self.binop_int(|l: u64, r| Some(l | r))?,
                Bytecode::BitAnd => self.binop_int(|l: u64, r| Some(l & r))?,
                Bytecode::Xor => self.binop_int(|l: u64, r| Some(l ^ r))?,
                Bytecode::Or => self.binop_bool(|l, r| l || r)?,
                Bytecode::And => self.binop_bool(|l, r| l && r)?,
                Bytecode::Lt => self.binop_bool(|l: u64, r| l < r)?,
                Bytecode::Gt => self.binop_bool(|l: u64, r| l > r)?,
                Bytecode::Le => self.binop_bool(|l: u64, r| l <= r)?,
                Bytecode::Ge => self.binop_bool(|l: u64, r| l >= r)?,
                Bytecode::Abort => {
                    let error_code = self.execution_stack.pop_as::<u64>()?;
                    return Err(
                        vm_error(self.execution_stack.location()?, StatusCode::ABORTED)
                            .with_sub_status(error_code),
                    );
                }

                // TODO: Should we emit different eq for different primitive type values?
                // How should equality between references be defined? Should we just panic
                // on reference values?
                Bytecode::Eq => {
                    let lhs = self.execution_stack.pop()?;
                    let rhs = self.execution_stack.pop()?;
                    self.execution_stack.push(Value::bool(lhs.equals(&rhs)?))?;
                }
                Bytecode::Neq => {
                    let lhs = self.execution_stack.pop()?;
                    let rhs = self.execution_stack.pop()?;
                    self.execution_stack
                        .push(Value::bool(lhs.not_equals(&rhs)?))?;
                }
                Bytecode::GetTxnGasUnitPrice => {
                    self.execution_stack
                        .push(Value::u64(self.txn_data.gas_unit_price().get()))?;
                }
                Bytecode::GetTxnMaxGasUnits => {
                    self.execution_stack
                        .push(Value::u64(self.txn_data.max_gas_amount().get()))?;
                }
                Bytecode::GetTxnSequenceNumber => {
                    self.execution_stack
                        .push(Value::u64(self.txn_data.sequence_number()))?;
                }
                Bytecode::GetTxnSenderAddress => {
                    self.execution_stack
                        .push(Value::address(self.txn_data.sender()))?;
                }
                Bytecode::GetTxnPublicKey => {
                    self.execution_stack.push(Value::byte_array(ByteArray::new(
                        self.txn_data.public_key().to_bytes().to_vec(),
                    )))?;
                }
                Bytecode::MutBorrowGlobal(idx, _) | Bytecode::ImmBorrowGlobal(idx, _) => {
                    let address = self.execution_stack.pop_as::<AccountAddress>()?;
                    let curr_module = self.execution_stack.top_frame()?.module();
                    let ap = make_access_path(curr_module, *idx, address);
                    if let Some(struct_def) = self.execution_stack.module_cache.resolve_struct_def(
                        curr_module,
                        *idx,
                        &self.gas_meter,
                    )? {
                        let global_ref = self.data_view.borrow_global(&ap, struct_def)?;
                        self.gas_meter.calculate_and_consume(
                            &instruction,
                            &self.execution_stack,
                            global_ref.size(),
                        )?;
                        self.execution_stack.push(Value::global_ref(global_ref))?;
                    } else {
                        return Err(VMStatus::new(StatusCode::LINKER_ERROR));
                    }
                }
                Bytecode::Exists(idx, _) => {
                    let address = self.execution_stack.pop_as::<AccountAddress>()?;
                    let curr_module = self.execution_stack.top_frame()?.module();
                    let ap = make_access_path(curr_module, *idx, address);
                    if let Some(struct_def) = self.execution_stack.module_cache.resolve_struct_def(
                        curr_module,
                        *idx,
                        &self.gas_meter,
                    )? {
                        let (exists, mem_size) = self.data_view.resource_exists(&ap, struct_def)?;
                        self.gas_meter.calculate_and_consume(
                            &instruction,
                            &self.execution_stack,
                            mem_size,
                        )?;
                        self.execution_stack.push(Value::bool(exists))?;
                    } else {
                        return Err(VMStatus::new(StatusCode::LINKER_ERROR));
                    }
                }
                Bytecode::MoveFrom(idx, _) => {
                    let address = self.execution_stack.pop_as::<AccountAddress>()?;
                    let curr_module = self.execution_stack.top_frame()?.module();
                    let ap = make_access_path(curr_module, *idx, address);
                    if let Some(struct_def) = self.execution_stack.module_cache.resolve_struct_def(
                        curr_module,
                        *idx,
                        &self.gas_meter,
                    )? {
                        let resource = self.data_view.move_resource_from(&ap, struct_def)?;
                        self.gas_meter.calculate_and_consume(
                            &instruction,
                            &self.execution_stack,
                            resource.size(),
                        )?;
                        self.execution_stack.push(resource)?;
                    } else {
                        return Err(VMStatus::new(StatusCode::LINKER_ERROR));
                    }
                }
                Bytecode::MoveToSender(idx, _) => {
                    let curr_module = self.execution_stack.top_frame()?.module();
                    let ap = make_access_path(curr_module, *idx, self.txn_data.sender());
                    if let Some(struct_def) = self.execution_stack.module_cache.resolve_struct_def(
                        curr_module,
                        *idx,
                        &self.gas_meter,
                    )? {
                        let resource = self.execution_stack.pop_as::<Struct>()?;
                        self.gas_meter.calculate_and_consume(
                            &instruction,
                            &self.execution_stack,
                            resource.size(),
                        )?;
                        self.data_view.move_resource_to(&ap, struct_def, resource)?;
                    } else {
                        return Err(VMStatus::new(StatusCode::LINKER_ERROR));
                    }
                }
                Bytecode::CreateAccount => {
                    let addr = self.execution_stack.pop_as::<AccountAddress>()?;
                    self.create_account(addr)?;
                }
                Bytecode::FreezeRef => {
                    // FreezeRef should just be a null op as we don't distinguish between mut and
                    // immut ref at runtime.
                }
                Bytecode::Not => {
                    let top = self.execution_stack.pop_as::<bool>()?;
                    self.execution_stack.push(Value::bool(!top))?;
                }
                Bytecode::GetGasRemaining => {
                    self.execution_stack
                        .push(Value::u64(self.gas_meter.remaining_gas().get()))?;
                }
            }
            pc += 1;
        }

        if cfg!(test) || cfg!(feature = "instruction_synthesis") {
            // In order to test the behavior of an instruction stream, hitting end of the code
            // should report no error so that we can check the locals.
            Ok(code.len() as CodeOffset)
        } else {
            Err(VMStatus::new(StatusCode::PC_OVERFLOW))
        }
    }

    /// Convert the transaction arguments into move values and push them to the top of the stack.
    pub(crate) fn setup_main_args(&mut self, args: Vec<TransactionArgument>) {
        for arg in args.into_iter() {
            let push_result = self.execution_stack.push(match arg {
                TransactionArgument::U64(i) => Value::u64(i),
                TransactionArgument::Address(a) => Value::address(a),
                TransactionArgument::ByteArray(b) => Value::byte_array(b),
                TransactionArgument::String(s) => Value::string(VMString::new(s)),
            });
            assume!(push_result.is_ok());
            push_result.expect("Stack should be empty at beginning of function");
        }
    }

    /// Create an account on the blockchain by calling into `CREATE_ACCOUNT_NAME` function stored
    /// in the `ACCOUNT_MODULE` on chain.
    pub fn create_account(&mut self, addr: AccountAddress) -> VMResult<()> {
        let account_module = self
            .execution_stack
            .module_cache
            .get_loaded_module(&ACCOUNT_MODULE)?
            .ok_or_else(|| VMStatus::new(StatusCode::LINKER_ERROR))?;

        // TODO: Currently the event counter will cause the gas cost for create account be flexible.
        //       We either need to fix the gas stability test cases in tests or we need to come up
        //       with some better ideas for the event counter creation.
        self.gas_meter.disable_metering();
        // Address will be used as the initial authentication key.
        self.execute_function(
            &ACCOUNT_MODULE,
            &CREATE_ACCOUNT_NAME,
            vec![Value::byte_array(ByteArray::new(addr.to_vec()))],
        )?;
        self.gas_meter.enable_metering();

        let account_resource = self.execution_stack.pop_as::<Struct>()?;
        let account_struct_id = account_module
            .struct_defs_table
            .get(&*ACCOUNT_STRUCT_NAME)
            .ok_or_else(|| VMStatus::new(StatusCode::LINKER_ERROR))?;
        let account_struct_def = self
            .execution_stack
            .module_cache
            .resolve_struct_def(account_module, *account_struct_id, &self.gas_meter)?
            .ok_or_else(|| VMStatus::new(StatusCode::LINKER_ERROR))?;

        // TODO: Adding the freshly created account's expiration date to the TransactionOutput here.
        let account_path = make_access_path(account_module, *account_struct_id, addr);
        self.data_view
            .move_resource_to(&account_path, account_struct_def, account_resource)
    }

    /// Run the prologue of a transaction by calling into `PROLOGUE_NAME` function stored
    /// in the `ACCOUNT_MODULE` on chain.
    pub(crate) fn run_prologue(&mut self) -> VMResult<()> {
        record_stats! {time_hist | TXN_PROLOGUE_TIME_TAKEN | {
                self.gas_meter.disable_metering();
                let result = self.execute_function(&ACCOUNT_MODULE, &PROLOGUE_NAME, vec![]);
                self.gas_meter.enable_metering();
                result
            }
        }
    }

    /// Run the epilogue of a transaction by calling into `EPILOGUE_NAME` function stored
    /// in the `ACCOUNT_MODULE` on chain.
    fn run_epilogue(&mut self) -> VMResult<()> {
        record_stats! {time_hist | TXN_EPILOGUE_TIME_TAKEN | {
                self.gas_meter.disable_metering();
                let result = self.execute_function(&ACCOUNT_MODULE, &EPILOGUE_NAME, vec![]);
                self.gas_meter.enable_metering();
                result
            }
        }
    }

    /// Generate the TransactionOutput on failure. There can be two possibilities:
    /// 1. The transaction encounters some runtime error, such as out of gas, arithmetic overflow,
    /// etc. In this scenario, we are going to keep this transaction and charge proper gas to the
    /// sender. 2. The transaction encounters VM invariant violation error type which indicates some
    /// properties should have been guaranteed failed. Such transaction should be discarded for
    /// sanity but this implies a bug in the VM that we should take care of.
    pub(crate) fn failed_transaction_cleanup(&mut self, result: VMResult<()>) -> TransactionOutput {
        // Discard all the local writes, restart execution from a clean state.
        self.clear();
        match self.run_epilogue() {
            Ok(_) => match self.make_write_set(vec![], result) {
                Ok(trans_out) => trans_out,
                Err(err) => error_output(err),
            },
            // Running epilogue shouldn't fail here as we've already checked for enough balance in
            // the prologue
            Err(err) => error_output(err),
        }
    }

    /// Clear all the writes local to this transaction.
    fn clear(&mut self) {
        self.data_view.clear();
        self.event_data.clear();
    }

    /// Generate the TransactionOutput for a successful transaction
    pub(crate) fn transaction_cleanup(
        &mut self,
        to_be_published_modules: Vec<(ModuleId, Vec<u8>)>,
    ) -> TransactionOutput {
        // First run the epilogue
        match self.run_epilogue() {
            // If epilogue runs successfully, try to emit the writeset.
            Ok(_) => match self.make_write_set(to_be_published_modules, Ok(())) {
                // This step could fail if the program has dangling global reference
                Ok(trans_out) => trans_out,
                // In case of failure, run the cleanup code.
                Err(err) => self.failed_transaction_cleanup(Err(err)),
            },
            // If the sender depleted its balance and can't pay for the gas, run the cleanup code.
            Err(err) => match err.status_type() {
                StatusType::InvariantViolation => error_output(err),
                _ => self.failed_transaction_cleanup(Err(err)),
            },
        }
    }

    /// Entrypoint into the interpreter. All external calls need to be routed through this
    /// function.
    pub(crate) fn interpeter_entrypoint(&mut self, func: FunctionRef<'txn>) -> VMResult<()> {
        // We charge an intrinsic amount of gas based upon the size of the transaction submitted
        // (in raw bytes).
        let txn_size = self.txn_data.transaction_size;
        // The callers of this function verify the transaction before executing it. Transaction
        // verification ensures the following condition.
        assume!(txn_size.get() <= (MAX_TRANSACTION_SIZE_IN_BYTES as u64));
        // We count the intrinsic cost of the transaction here, since that needs to also cover the
        // setup of the function.
        let starting_gas = self.gas_meter.remaining_gas().get();
        self.gas_meter
            .charge_transaction_gas(txn_size, &self.execution_stack)?;
        let ret = self.execute_function_impl(func);
        record_stats!(observe | TXN_EXECUTION_GAS_USAGE | starting_gas);
        ret
    }

    /// Execute a function given a FunctionRef.
    fn execute_function_impl(&mut self, func: FunctionRef<'txn>) -> VMResult<()> {
        let beginning_height = self.execution_stack.call_stack_height();
        self.execution_stack.push_call(func)?;
        // We always start execution from the first instruction.
        let mut pc = 0;

        // Execute code until the stack goes back to its original height. At that time we will know
        // this function has terminated.
        while self.execution_stack.call_stack_height() != beginning_height {
            let code = self.execution_stack.top_frame()?.code_definition();

            // Get the pc for the next instruction to be executed.
            pc = self.execute_block(code, pc)?;

            if self.execution_stack.call_stack_height() == beginning_height {
                return Ok(());
            }
        }

        Ok(())
    }

    /// Execute a function.
    /// `module` is an identifier for the name the module is stored in. `function_name` is the name
    /// of the function. If such function is found, the VM will execute this function with arguments
    /// `args`. The return value will be placed on the top of the value stack and abort if an error
    /// occurs.
    pub fn execute_function(
        &mut self,
        module: &ModuleId,
        function_name: &IdentStr,
        args: Vec<Value>,
    ) -> VMResult<()> {
        let loaded_module = match self
            .execution_stack
            .module_cache
            .get_loaded_module(module)?
        {
            Some(module) => module,
            None => return Err(VMStatus::new(StatusCode::LINKER_ERROR)),
        };
        let func_idx = loaded_module
            .function_defs_table
            .get(function_name)
            .ok_or_else(|| VMStatus::new(StatusCode::LINKER_ERROR))?;
        let func = FunctionRef::new(loaded_module, *func_idx);

        for arg in args.into_iter() {
            self.execution_stack.push(arg)?;
        }

        self.execute_function_impl(func)
    }

    /// Execute a function with the sender set to `sender`, restoring the original sender afterward.
    /// This should only be used in the logic for generating the genesis block.
    #[allow(non_snake_case)]
    pub fn execute_function_with_sender_FOR_GENESIS_ONLY(
        &mut self,
        address: AccountAddress,
        module: &ModuleId,
        function_name: &IdentStr,
        args: Vec<Value>,
    ) -> VMResult<()> {
        let old_sender = self.txn_data.sender();
        self.txn_data.sender = address;

        let res = self.execute_function(module, function_name, args);
        self.txn_data.sender = old_sender;
        res
    }

    /// Get the value on the top of the value stack.
    pub fn pop_stack(&mut self) -> VMResult<Value> {
        self.execution_stack.pop()
    }

    /// Produce a write set at the end of a transaction. This will clear all the local states in
    /// the TransactionProcessor and turn them into a writeset.
    pub fn make_write_set(
        &mut self,
        to_be_published_modules: Vec<(ModuleId, Vec<u8>)>,
        result: VMResult<()>,
    ) -> VMResult<TransactionOutput> {
        // This should only be used for bookkeeping. The gas is already deducted from the sender's
        // account in the account module's epilogue.
        let gas_used: u64 = self
            .txn_data
            .max_gas_amount
            .sub(self.gas_meter.remaining_gas())
            .mul(self.txn_data.gas_unit_price)
            .get();
        let write_set = self.data_view.make_write_set(to_be_published_modules)?;

        record_stats!(observe | TXN_TOTAL_GAS_USAGE | gas_used);

        Ok(TransactionOutput::new(
            write_set,
            self.event_data.clone(),
            gas_used,
            match result {
                Ok(()) => TransactionStatus::from(VMStatus::new(StatusCode::EXECUTED)),
                Err(err) => TransactionStatus::from(err),
            },
        ))
    }
}

#[inline]
fn error_output(err: VMStatus) -> TransactionOutput {
    // Since this transaction will be discarded, no writeset will be included.
    TransactionOutput::new(
        WriteSet::default(),
        vec![],
        0,
        TransactionStatus::Discard(err),
    )
}

/// A helper function for executing a single script. Will be deprecated once we have a better
/// testing framework for executing arbitrary script.
pub fn execute_function(
    caller_script: VerifiedScript,
    modules: Vec<VerifiedModule>,
    _args: Vec<TransactionArgument>,
    data_cache: &dyn RemoteCache,
) -> VMResult<()> {
    let allocator = Arena::new();
    let module_cache = VMModuleCache::new(&allocator);
    let main_module = caller_script.into_module();
    let loaded_main = LoadedModule::new(main_module);
    let entry_func = FunctionRef::new(&loaded_main, CompiledScript::MAIN_INDEX);
    let txn_metadata = TransactionMetadata::default();
    for m in modules {
        module_cache.cache_module(m);
    }
    let mut vm = TransactionExecutor {
        execution_stack: ExecutionStack::new(&module_cache),
        gas_meter: GasMeter::new(txn_metadata.max_gas_amount()),
        txn_data: txn_metadata,
        event_data: Vec::new(),
        data_view: TransactionDataCache::new(data_cache),
    };
    vm.execute_function_impl(entry_func)
}

#[cfg(feature = "instruction_synthesis")]
impl<'alloc, 'txn, P> TransactionExecutor<'alloc, 'txn, P>
where
    'alloc: 'txn,
    P: ModuleCache<'alloc>,
{
    /// Clear all the writes local to this transaction.
    pub fn clear_writes(&mut self) {
        self.data_view.clear();
        self.event_data.clear();
    }

    /// During cost synthesis, turn off gas metering so that we don't run out of gas.
    pub fn turn_off_gas_metering(&mut self) {
        self.gas_meter.disable_metering();
    }
}
