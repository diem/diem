// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::txn_executor::convert_txn_args;
use crate::{
    chain_state::{ChainState, SystemExecutionContext, TransactionExecutionContext},
    counters::*,
    data_cache::{BlockDataCache, RemoteCache},
    move_vm::MoveVM,
    system_module_names::*,
    system_txn::block_metadata_processor::process_block_metadata,
    VMExecutor, VMVerifier,
};
use libra_config::config::{VMConfig, VMPublishingOption};
use libra_crypto::HashValue;
use libra_logger::prelude::*;
use libra_state_view::StateView;
use libra_types::{
    block_metadata::BlockMetadata,
    byte_array::ByteArray,
    transaction::{
        ChangeSet, SignatureCheckedTransaction, SignedTransaction, Transaction,
        TransactionArgument, TransactionOutput, TransactionPayload, TransactionStatus,
        MAX_TRANSACTION_SIZE_IN_BYTES,
    },
    vm_error::{sub_status, StatusCode, VMStatus},
    write_set::WriteSet,
};
use rayon::prelude::*;
use std::sync::Arc;
use vm::errors::convert_prologue_runtime_error;
use vm::{
    errors::VMResult,
    gas_schedule::{self, AbstractMemorySize, CostTable, GasAlgebra, GasCarrier, GasUnits},
    transaction_metadata::TransactionMetadata,
};
use vm_runtime_types::value::Value;

#[derive(Clone)]
/// A wrapper to make VMRuntime standalone and thread safe.
pub struct LibraVM {
    move_vm: Arc<MoveVM>,
    gas_schedule: Option<CostTable>,
    config: VMConfig,
}

impl LibraVM {
    pub fn new(config: &VMConfig) -> Self {
        let inner = MoveVM::new();
        Self {
            move_vm: Arc::new(inner),
            gas_schedule: None,
            config: config.clone(),
        }
    }

    fn load_gas_schedule(&mut self, data_cache: &dyn RemoteCache) {
        let mut ctx = SystemExecutionContext::new(data_cache, GasUnits::new(0));
        self.gas_schedule = self.move_vm.load_gas_schedule(&mut ctx, data_cache).ok();
    }

    fn get_gas_schedule(&self) -> VMResult<&CostTable> {
        self.gas_schedule.as_ref().ok_or_else(|| {
            VMStatus::new(StatusCode::VM_STARTUP_FAILURE)
                .with_sub_status(sub_status::VSF_GAS_SCHEDULE_NOT_FOUND)
        })
    }

    fn check_payload(
        &self,
        payload: &TransactionPayload,
        state_view: &dyn StateView,
    ) -> VMResult<()> {
        match payload {
            // TODO: Remove WriteSet from TransactionPayload.
            TransactionPayload::WriteSet(change_set) => {
                self.check_change_set(change_set, state_view)
            }
            TransactionPayload::Script(script) => {
                if !is_allowed_script(&self.config.publishing_options, &script.code()) {
                    warn!("[VM] Custom scripts not allowed: {:?}", &script.code());
                    Err(VMStatus::new(StatusCode::UNKNOWN_SCRIPT))
                } else {
                    Ok(())
                }
            }
            TransactionPayload::Module(_module) => {
                if !&self.config.publishing_options.is_open() {
                    warn!("[VM] Custom modules not allowed");
                    Err(VMStatus::new(StatusCode::UNKNOWN_MODULE))
                } else {
                    Ok(())
                }
            }
            TransactionPayload::Program => Err(VMStatus::new(StatusCode::UNKNOWN_SCRIPT)),
        }
    }

    fn check_change_set(&self, change_set: &ChangeSet, state_view: &dyn StateView) -> VMResult<()> {
        // TODO: Replace this logic with actual checks.
        if state_view.is_genesis() {
            for (_access_path, write_op) in change_set.write_set() {
                // Genesis transactions only add entries, never delete them.
                if write_op.is_deletion() {
                    error!("[VM] Bad genesis block");
                    // TODO: return more detailed error somehow?
                    return Err(VMStatus::new(StatusCode::INVALID_WRITE_SET));
                }
            }
            Ok(())
        } else {
            Err(VMStatus::new(StatusCode::REJECTED_WRITE_SET))
        }
    }

    fn check_gas(&self, txn: &SignedTransaction) -> VMResult<()> {
        // Do not check gas limit for writeset transaction.
        if let TransactionPayload::WriteSet(_) = txn.payload() {
            return Ok(());
        }

        let raw_bytes_len = AbstractMemorySize::new(txn.raw_txn_bytes_len() as GasCarrier);
        // The transaction is too large.
        if txn.raw_txn_bytes_len() > MAX_TRANSACTION_SIZE_IN_BYTES {
            let error_str = format!(
                "max size: {}, txn size: {}",
                MAX_TRANSACTION_SIZE_IN_BYTES,
                raw_bytes_len.get()
            );
            warn!(
                "[VM] Transaction size too big {} (max {})",
                raw_bytes_len.get(),
                MAX_TRANSACTION_SIZE_IN_BYTES
            );
            return Err(
                VMStatus::new(StatusCode::EXCEEDED_MAX_TRANSACTION_SIZE).with_message(error_str)
            );
        }

        // Check is performed on `txn.raw_txn_bytes_len()` which is the same as
        // `raw_bytes_len`
        assume!(raw_bytes_len.get() <= MAX_TRANSACTION_SIZE_IN_BYTES as u64);

        // The submitted max gas units that the transaction can consume is greater than the
        // maximum number of gas units bound that we have set for any
        // transaction.
        if txn.max_gas_amount() > gas_schedule::MAXIMUM_NUMBER_OF_GAS_UNITS.get() {
            let error_str = format!(
                "max gas units: {}, gas units submitted: {}",
                gas_schedule::MAXIMUM_NUMBER_OF_GAS_UNITS.get(),
                txn.max_gas_amount()
            );
            warn!(
                "[VM] Gas unit error; max {}, submitted {}",
                gas_schedule::MAXIMUM_NUMBER_OF_GAS_UNITS.get(),
                txn.max_gas_amount()
            );
            return Err(
                VMStatus::new(StatusCode::MAX_GAS_UNITS_EXCEEDS_MAX_GAS_UNITS_BOUND)
                    .with_message(error_str),
            );
        }

        // The submitted transactions max gas units needs to be at least enough to cover the
        // intrinsic cost of the transaction as calculated against the size of the
        // underlying `RawTransaction`
        let min_txn_fee = gas_schedule::calculate_intrinsic_gas(raw_bytes_len);
        if txn.max_gas_amount() < min_txn_fee.get() {
            let error_str = format!(
                "min gas required for txn: {}, gas submitted: {}",
                min_txn_fee.get(),
                txn.max_gas_amount()
            );
            warn!(
                "[VM] Gas unit error; min {}, submitted {}",
                min_txn_fee.get(),
                txn.max_gas_amount()
            );
            return Err(
                VMStatus::new(StatusCode::MAX_GAS_UNITS_BELOW_MIN_TRANSACTION_GAS_UNITS)
                    .with_message(error_str),
            );
        }

        // The submitted gas price is less than the minimum gas unit price set by the VM.
        // NB: MIN_PRICE_PER_GAS_UNIT may equal zero, but need not in the future. Hence why
        // we turn off the clippy warning.
        #[allow(clippy::absurd_extreme_comparisons)]
        let below_min_bound = txn.gas_unit_price() < gas_schedule::MIN_PRICE_PER_GAS_UNIT.get();
        if below_min_bound {
            let error_str = format!(
                "gas unit min price: {}, submitted price: {}",
                gas_schedule::MIN_PRICE_PER_GAS_UNIT.get(),
                txn.gas_unit_price()
            );
            warn!(
                "[VM] Gas unit error; min {}, submitted {}",
                gas_schedule::MIN_PRICE_PER_GAS_UNIT.get(),
                txn.gas_unit_price()
            );
            return Err(
                VMStatus::new(StatusCode::GAS_UNIT_PRICE_BELOW_MIN_BOUND).with_message(error_str)
            );
        }

        // The submitted gas price is greater than the maximum gas unit price set by the VM.
        if txn.gas_unit_price() > gas_schedule::MAX_PRICE_PER_GAS_UNIT.get() {
            let error_str = format!(
                "gas unit max price: {}, submitted price: {}",
                gas_schedule::MAX_PRICE_PER_GAS_UNIT.get(),
                txn.gas_unit_price()
            );
            warn!(
                "[VM] Gas unit error; min {}, submitted {}",
                gas_schedule::MAX_PRICE_PER_GAS_UNIT.get(),
                txn.gas_unit_price()
            );
            return Err(
                VMStatus::new(StatusCode::GAS_UNIT_PRICE_ABOVE_MAX_BOUND).with_message(error_str)
            );
        }
        Ok(())
    }

    fn verify_transaction_impl(
        &self,
        transaction: &SignatureCheckedTransaction,
        gas_schedule: VMResult<&CostTable>,
        state_view: &dyn StateView,
        remote_cache: &dyn RemoteCache,
    ) -> VMResult<VerifiedTranscationPayload> {
        let mut ctx = SystemExecutionContext::new(remote_cache, GasUnits::new(0));
        self.check_gas(transaction)?;
        self.check_payload(transaction.payload(), state_view)?;
        let txn_data = TransactionMetadata::new(transaction);
        match transaction.payload() {
            TransactionPayload::Program => Err(VMStatus::new(StatusCode::UNKNOWN_SCRIPT)),
            TransactionPayload::Script(script) => {
                self.run_prologue(gas_schedule, &mut ctx, &txn_data)?;
                Ok(VerifiedTranscationPayload::Script(
                    script.code().to_vec(),
                    script.args().to_vec(),
                ))
            }
            TransactionPayload::Module(module) => {
                self.run_prologue(gas_schedule, &mut ctx, &txn_data)?;
                Ok(VerifiedTranscationPayload::Module(module.code().to_vec()))
            }
            TransactionPayload::WriteSet(change_set) => {
                Ok(VerifiedTranscationPayload::ChangeSet(change_set.clone()))
            }
        }
    }

    fn execute_verified_payload(
        &mut self,
        remote_cache: &mut BlockDataCache<'_>,
        txn_data: &TransactionMetadata,
        payload: VerifiedTranscationPayload,
    ) -> TransactionOutput {
        let mut ctx = TransactionExecutionContext::new(txn_data.max_gas_amount(), remote_cache);
        // TODO: The logic for handling falied transaction fee is pretty ugly right now. Fix it later.
        let mut failed_gas_left = GasUnits::new(0);
        match payload {
            VerifiedTranscationPayload::Module(m) => {
                self.move_vm.publish_module(m, &mut ctx, txn_data)
            }
            VerifiedTranscationPayload::Script(s, args) => {
                let gas_schedule = match self.get_gas_schedule() {
                    Ok(s) => s,
                    Err(e) => return discard_error_output(e),
                };
                self.move_vm.execute_script(
                    s,
                    gas_schedule,
                    &mut ctx,
                    txn_data,
                    convert_txn_args(args),
                )
            }
            VerifiedTranscationPayload::ChangeSet(change_set) => {
                return self.process_change_set(remote_cache, change_set);
            }
        }
        .map_err(|err| {
            failed_gas_left = ctx.gas_left();
            err
        })
        .and_then(|_| {
            failed_gas_left = ctx.gas_left();
            let mut gas_free_ctx = SystemExecutionContext::from(ctx);
            self.run_epilogue(&mut gas_free_ctx, txn_data)
                .and_then(|_| gas_free_ctx.get_transaction_output(txn_data, Ok(())))
        })
        .unwrap_or_else(|err| {
            let mut gas_free_ctx = SystemExecutionContext::new(remote_cache, failed_gas_left);
            self.run_epilogue(&mut gas_free_ctx, txn_data)
                .and_then(|_| gas_free_ctx.get_transaction_output(txn_data, Err(err)))
                .unwrap_or_else(discard_error_output)
        })
    }

    fn execute_user_transaction(
        &mut self,
        state_view: &dyn StateView,
        remote_cache: &mut BlockDataCache<'_>,
        txn: &SignatureCheckedTransaction,
    ) -> TransactionOutput {
        let txn_data = TransactionMetadata::new(txn);
        let verified_payload = record_stats! {time_hist | TXN_VERIFICATION_TIME_TAKEN | {
            self.verify_transaction_impl(txn, self.get_gas_schedule(), state_view, remote_cache)
        }};
        let result = verified_payload
            .and_then(|verified_payload| {
                record_stats! {time_hist | TXN_EXECUTION_TIME_TAKEN | {
                Ok(self.execute_verified_payload(
                    remote_cache,
                    &txn_data,
                    verified_payload,
                ))
                }}
            })
            .unwrap_or_else(discard_error_output);
        if let TransactionStatus::Keep(_) = result.status() {
            remote_cache.push_write_set(result.write_set())
        };
        result
    }

    fn process_change_set(
        &mut self,
        remote_cache: &mut BlockDataCache<'_>,
        change_set: ChangeSet,
    ) -> TransactionOutput {
        let (write_set, events) = change_set.into_inner();
        remote_cache.push_write_set(&write_set);
        self.load_gas_schedule(remote_cache);
        TransactionOutput::new(
            write_set,
            events,
            0,
            VMStatus::new(StatusCode::EXECUTED).into(),
        )
    }

    /// Run the prologue of a transaction by calling into `PROLOGUE_NAME` function stored
    /// in the `ACCOUNT_MODULE` on chain.
    fn run_prologue<T: ChainState>(
        &self,
        gas_schedule: VMResult<&CostTable>,
        chain_state: &mut T,
        txn_data: &TransactionMetadata,
    ) -> VMResult<()> {
        let txn_sequence_number = txn_data.sequence_number();
        let txn_public_key = txn_data.public_key().to_bytes().to_vec();
        let txn_gas_price = txn_data.gas_unit_price().get();
        let txn_max_gas_units = txn_data.max_gas_amount().get();
        record_stats! {time_hist | TXN_PROLOGUE_TIME_TAKEN | {
                self.move_vm
                    .execute_function(
                        &ACCOUNT_MODULE,
                        &PROLOGUE_NAME,
                        gas_schedule?,
                        chain_state,
                        &txn_data,
                        vec![
                            Value::u64(txn_sequence_number),
                            Value::byte_array(ByteArray::new(txn_public_key)),
                            Value::u64(txn_gas_price),
                            Value::u64(txn_max_gas_units),
                        ],
                    )
                    .map_err(|err| convert_prologue_runtime_error(&err, &txn_data.sender))
                }
        }
    }

    /// Run the epilogue of a transaction by calling into `EPILOGUE_NAME` function stored
    /// in the `ACCOUNT_MODULE` on chain.
    fn run_epilogue<T: ChainState>(
        &self,
        chain_state: &mut T,
        txn_data: &TransactionMetadata,
    ) -> VMResult<()> {
        let txn_sequence_number = txn_data.sequence_number();
        let txn_gas_price = txn_data.gas_unit_price().get();
        let txn_max_gas_units = txn_data.max_gas_amount().get();
        let gas_remaining = chain_state.remaining_gas().get();
        record_stats! {time_hist | TXN_EPILOGUE_TIME_TAKEN | {
                self.move_vm.execute_function(
                    &ACCOUNT_MODULE,
                    &EPILOGUE_NAME,
                    self.get_gas_schedule()?,
                    chain_state,
                    &txn_data,
                    vec![
                        Value::u64(txn_sequence_number),
                        Value::u64(txn_gas_price),
                        Value::u64(txn_max_gas_units),
                        Value::u64(gas_remaining),
                    ],
                )
            }
        }
    }

    fn execute_block_impl(
        &mut self,
        transactions: Vec<Transaction>,
        state_view: &dyn StateView,
    ) -> VMResult<Vec<TransactionOutput>> {
        let count = transactions.len();
        let mut result = vec![];
        let blocks = chunk_block_transactions(transactions);
        let mut data_cache = BlockDataCache::new(state_view);
        self.load_gas_schedule(&data_cache);
        for block in blocks {
            match block {
                TransactionBlock::UserTransaction(txns) => {
                    let mut outs =
                        self.execute_user_transactions(txns, &mut data_cache, state_view)?;
                    result.append(&mut outs);
                }
                TransactionBlock::BlockPrologue(block_metadata) => {
                    result.push(self.move_vm.execute_runtime(|runtime| {
                        process_block_metadata(block_metadata, runtime, &mut data_cache)
                    })?)
                }
                // TODO: Implement the logic for processing writeset transactions.
                TransactionBlock::WriteSet(_) => unimplemented!(""),
            }
        }
        report_block_count(count);
        Ok(result)
    }

    fn execute_user_transactions(
        &mut self,
        txn_block: Vec<SignedTransaction>,
        data_cache: &mut BlockDataCache<'_>,
        state_view: &dyn StateView,
    ) -> VMResult<Vec<TransactionOutput>> {
        let signature_verified_block: Vec<Result<SignatureCheckedTransaction, VMStatus>> =
            txn_block
                .into_par_iter()
                .map(|txn| {
                    txn.check_signature()
                        .map_err(|_| VMStatus::new(StatusCode::INVALID_SIGNATURE))
                })
                .collect();
        let mut result = vec![];
        for transaction in signature_verified_block {
            record_stats! {time_hist | TXN_TOTAL_TIME_TAKEN | {
                    let output = match transaction {
                        Ok(txn) => self.execute_user_transaction(state_view, data_cache, &txn),
                        Err(e) => discard_error_output(e),
                    };
                    report_execution_status(output.status());

                    // `result` is initially empty, a single element is pushed per loop iteration and
                    // the number of iterations is bound to the max size of `signature_verified_block`
                    assume!(result.len() < usize::max_value());
                    result.push(output);
                }
            }
        }
        Ok(result)
    }
}

pub(crate) fn discard_error_output(err: VMStatus) -> TransactionOutput {
    // Since this transaction will be discarded, no writeset will be included.
    TransactionOutput::new(
        WriteSet::default(),
        vec![],
        0,
        TransactionStatus::Discard(err),
    )
}

// Validators external API
impl VMVerifier for LibraVM {
    /// Determine if a transaction is valid. Will return `None` if the transaction is accepted,
    /// `Some(Err)` if the VM rejects it, with `Err` as an error code. Verification performs the
    /// following steps:
    /// 1. The signature on the `SignedTransaction` matches the public key included in the
    ///    transaction
    /// 2. The script to be executed is under given specific configuration.
    /// 3. Invokes `LibraAccount.prologue`, which checks properties such as the transaction has the
    /// right sequence number and the sender has enough balance to pay for the gas.
    /// TBD:
    /// 1. Transaction arguments matches the main function's type signature.
    ///    We don't check this item for now and would execute the check at execution time.
    fn validate_transaction(
        &self,
        transaction: SignedTransaction,
        state_view: &dyn StateView,
    ) -> Option<VMStatus> {
        let data_cache = BlockDataCache::new(state_view);
        record_stats! {time_hist | TXN_VALIDATION_TIME_TAKEN | {
                let mut ctx = SystemExecutionContext::new(&data_cache, GasUnits::new(0));
                let gas_schedule = self.move_vm.load_gas_schedule(&mut ctx, &data_cache);
                let signature_verified_txn = match transaction.check_signature() {
                    Ok(t) => t,
                    Err(_) => return Some(VMStatus::new(StatusCode::INVALID_SIGNATURE)),
                };
                let res = match self.verify_transaction_impl(&signature_verified_txn, gas_schedule.as_ref().map_err(|err| err.clone()), state_view, &data_cache) {
                    Ok(_) => None,
                    Err(err) => {
                        if err.major_status == StatusCode::SEQUENCE_NUMBER_TOO_NEW {
                            None
                        } else {
                            Some(convert_prologue_runtime_error(&err, &signature_verified_txn.sender()))
                        }
                    }
                };
                report_verification_status(&res);
                res
            }
        }
    }
}

// Executor external API
impl VMExecutor for LibraVM {
    /// Execute a block of `transactions`. The output vector will have the exact same length as the
    /// input vector. The discarded transactions will be marked as `TransactionStatus::Discard` and
    /// have an empty `WriteSet`. Also `state_view` is immutable, and does not have interior
    /// mutability. Writes to be applied to the data view are encoded in the write set part of a
    /// transaction output.
    fn execute_block(
        transactions: Vec<Transaction>,
        config: &VMConfig,
        state_view: &dyn StateView,
    ) -> VMResult<Vec<TransactionOutput>> {
        let mut vm = LibraVM::new(config);
        vm.execute_block_impl(transactions, state_view)
    }
}

/// Transactions divided by transaction flow.
/// Transaction flows are different across different types of transactions.
pub enum TransactionBlock {
    UserTransaction(Vec<SignedTransaction>),
    WriteSet(WriteSet),
    BlockPrologue(BlockMetadata),
}

pub fn chunk_block_transactions(txns: Vec<Transaction>) -> Vec<TransactionBlock> {
    let mut blocks = vec![];
    let mut buf = vec![];
    for txn in txns {
        match txn {
            Transaction::BlockMetadata(data) => {
                if !buf.is_empty() {
                    blocks.push(TransactionBlock::UserTransaction(buf));
                    buf = vec![];
                }
                blocks.push(TransactionBlock::BlockPrologue(data));
            }
            Transaction::WriteSet(ws) => {
                if !buf.is_empty() {
                    blocks.push(TransactionBlock::UserTransaction(buf));
                    buf = vec![];
                }
                blocks.push(TransactionBlock::WriteSet(ws));
            }
            Transaction::UserTransaction(txn) => {
                buf.push(txn);
            }
        }
    }
    if !buf.is_empty() {
        blocks.push(TransactionBlock::UserTransaction(buf));
    }
    blocks
}

enum VerifiedTranscationPayload {
    ChangeSet(ChangeSet),
    Script(Vec<u8>, Vec<TransactionArgument>),
    Module(Vec<u8>),
}

pub fn is_allowed_script(publishing_option: &VMPublishingOption, program: &[u8]) -> bool {
    match publishing_option {
        VMPublishingOption::Open | VMPublishingOption::CustomScripts => true,
        VMPublishingOption::Locked(whitelist) => {
            let hash_value = HashValue::from_sha3_256(program);
            whitelist.contains(hash_value.as_ref())
        }
    }
}

#[test]
fn vm_thread_safe() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_send::<LibraVM>();
    assert_sync::<LibraVM>();
    assert_send::<MoveVM>();
    assert_sync::<MoveVM>();
}
