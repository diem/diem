// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chain_state::TransactionExecutionContext;
use crate::{
    counters::*, data_cache::BlockDataCache, runtime::VMRuntime,
    system_module_names::LIBRA_SYSTEM_MODULE,
};
use lazy_static::lazy_static;
use libra_types::transaction::TransactionStatus;
use libra_types::{
    block_metadata::BlockMetadata,
    identifier::Identifier,
    transaction::TransactionOutput,
    vm_error::{StatusCode, VMStatus},
};
use vm::{
    errors::VMResult,
    gas_schedule::{CostTable, GasAlgebra, GasUnits},
    transaction_metadata::TransactionMetadata,
};
use vm_runtime_types::value::Value;

lazy_static! {
    static ref BLOCK_PROLOGUE: Identifier = Identifier::new("block_prologue").unwrap();
}

pub(crate) fn process_block_metadata(
    block_metadata: BlockMetadata,
    runtime: &VMRuntime,
    data_cache: &mut BlockDataCache<'_>,
) -> Result<TransactionOutput, VMStatus> {
    // TODO: How should we setup the metadata here? A couple of thoughts here:
    // 1. We might make the txn_data to be poisoned so that reading anything will result in a panic.
    // 2. The most important consideration is figuring out the sender address.  Having a notion of a
    //    "null address" (probably 0x0...0) that is prohibited from containing modules or resources
    //    might be useful here.
    // 3. We set the max gas to a big number just to get rid of the potential out of gas error.
    let mut txn_data = TransactionMetadata::default();

    txn_data.max_gas_amount = GasUnits::new(std::u64::MAX);
    let mut interpreter_context =
        TransactionExecutionContext::new(txn_data.max_gas_amount(), data_cache);
    // TODO: We might need a non zero cost table here so that we can at least bound the execution
    //       time by a reasonable amount.
    let gas_schedule = CostTable::zero();

    let result = if let Ok((id, timestamp, previous_vote, proposer)) = block_metadata.into_inner() {
        let args = vec![
            Value::u64(timestamp),
            Value::byte_array(id),
            Value::byte_array(previous_vote),
            Value::address(proposer),
        ];
        runtime.execute_function(
            &mut interpreter_context,
            &txn_data,
            &gas_schedule,
            &LIBRA_SYSTEM_MODULE,
            &BLOCK_PROLOGUE,
            args,
        )
    } else {
        Err(VMStatus::new(StatusCode::MALFORMED))
    };
    result
        .and_then(|_| make_write_set(&mut interpreter_context, &txn_data))
        .and_then(|output| {
            data_cache.push_write_set(output.write_set());
            Ok(output)
        })
}

fn make_write_set(
    interpreter_context: &mut TransactionExecutionContext,
    txn_data: &TransactionMetadata,
) -> VMResult<TransactionOutput> {
    // This should only be used for bookkeeping. The gas is already deducted from the sender's
    // account in the account module's epilogue.
    let gas_used: u64 = txn_data
        .max_gas_amount()
        .sub(interpreter_context.gas_left())
        .mul(txn_data.gas_unit_price())
        .get();
    let write_set = interpreter_context.make_write_set()?;

    record_stats!(observe | TXN_TOTAL_GAS_USAGE | gas_used);

    Ok(TransactionOutput::new(
        write_set,
        interpreter_context.events().to_vec(),
        gas_used,
        TransactionStatus::from(VMStatus::new(StatusCode::EXECUTED)),
    ))
}
