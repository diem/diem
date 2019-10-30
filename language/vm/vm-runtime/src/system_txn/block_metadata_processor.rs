use crate::{
    code_cache::module_cache::ModuleCache,
    data_cache::BlockDataCache,
    process_txn::execute::ExecutedTransaction,
    txn_executor::{TransactionExecutor, LIBRA_SYSTEM_MODULE},
};
use lazy_static::lazy_static;
use libra_types::{
    block_metadata::BlockMetadata,
    identifier::Identifier,
    transaction::TransactionOutput,
    vm_error::{StatusCode, VMStatus},
};
use vm::transaction_metadata::TransactionMetadata;
use vm_runtime_types::value::Value;

lazy_static! {
    static ref BLOCK_PROLOGUE: Identifier = Identifier::new("block_prologue").unwrap();
}

pub(crate) fn process_block_metadata<'alloc, P>(
    block_metadata: BlockMetadata,
    module_cache: P,
    data_cache: &mut BlockDataCache<'_>,
) -> TransactionOutput
where
    P: ModuleCache<'alloc>,
{
    // TODO: How should we setup the metadata here?
    let txn_data = TransactionMetadata::default();

    let mut txn_executor = TransactionExecutor::new(&module_cache, data_cache, txn_data);
    let result = if let Ok((id, timestamp, previous_vote, proposer)) = block_metadata.into_inner() {
        let args = vec![
            Value::u64(timestamp),
            Value::byte_array(id),
            Value::byte_array(previous_vote),
            Value::address(proposer),
        ];
        txn_executor.execute_function(&LIBRA_SYSTEM_MODULE, &BLOCK_PROLOGUE, args)
    } else {
        Err(VMStatus::new(StatusCode::MALFORMED))
    };
    match result {
        Ok(_) => txn_executor.transaction_cleanup(vec![]),
        Err(err) => ExecutedTransaction::discard_error_output(err),
    }
}
