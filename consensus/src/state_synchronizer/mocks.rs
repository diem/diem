// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::state_synchronizer::coordinator::ExecutorProxyTrait;
use crypto::HashValue;
use execution_proto::proto::execution::{ExecuteChunkRequest, ExecuteChunkResponse};
use failure::Result;
use futures::{Future, FutureExt};
use proto_conv::FromProto;
use std::{
    pin::Pin,
    sync::atomic::{AtomicU64, Ordering},
};
use types::{
    account_address::AccountAddress,
    proof::AccumulatorProof,
    test_helpers::transaction_test_helpers::get_test_signed_txn,
    transaction::{SignedTransaction, TransactionInfo, TransactionListWithProof},
};
use vm_genesis::{encode_transfer_program, GENESIS_KEYPAIR};

#[derive(Default)]
pub struct MockExecutorProxy {
    version: AtomicU64,
}

impl ExecutorProxyTrait for MockExecutorProxy {
    fn get_latest_version(&self) -> Pin<Box<dyn Future<Output = Result<u64>> + Send>> {
        let version = self.version.load(Ordering::Relaxed);
        async move { Ok(version) }.boxed()
    }

    fn execute_chunk(
        &self,
        _request: ExecuteChunkRequest,
    ) -> Pin<Box<dyn Future<Output = Result<ExecuteChunkResponse>> + Send>> {
        self.version.fetch_add(1, Ordering::Relaxed);
        async move { Ok(ExecuteChunkResponse::new()) }.boxed()
    }
}

pub fn gen_txn_list(sequence_number: u64) -> TransactionListWithProof {
    let sender = AccountAddress::from(GENESIS_KEYPAIR.1);
    let receiver = AccountAddress::new([0xff; 32]);
    let program = encode_transfer_program(&receiver, 1);
    let transaction = get_test_signed_txn(
        sender.into(),
        sequence_number,
        GENESIS_KEYPAIR.0.clone(),
        GENESIS_KEYPAIR.1,
        Some(program),
    );

    let txn_info = TransactionInfo::new(HashValue::zero(), HashValue::zero(), HashValue::zero(), 0);
    let accumulator_proof = AccumulatorProof::new(vec![]);
    TransactionListWithProof::new(
        vec![(
            SignedTransaction::from_proto(transaction).unwrap(),
            txn_info,
        )],
        None,
        Some(0),
        Some(accumulator_proof),
        None,
    )
}
