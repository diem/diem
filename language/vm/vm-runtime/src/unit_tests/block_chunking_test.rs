// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::libra_vm::{chunk_block_transactions, TransactionBlock};
use libra_types::transaction::Transaction;
use proptest::collection::vec;
use proptest::prelude::*;

fn reconstruct_transaction_vec(blocks: Vec<TransactionBlock>) -> Vec<Transaction> {
    let mut txns = vec![];
    for block in blocks {
        match block {
            TransactionBlock::WriteSet(ws) => txns.push(Transaction::WriteSet(ws)),
            TransactionBlock::BlockPrologue(ws) => txns.push(Transaction::BlockMetadata(ws)),
            TransactionBlock::UserTransaction(user_txns) => {
                assert!(!user_txns.is_empty());
                txns.append(
                    &mut user_txns
                        .into_iter()
                        .map(Transaction::UserTransaction)
                        .collect::<Vec<_>>(),
                )
            }
        }
    }
    txns
}

proptest! {
    #[test]
    fn chunking_round_trip(txns in vec(any::<Transaction>(), 1..20)) {
        prop_assert_eq!(reconstruct_transaction_vec(chunk_block_transactions(txns.clone())), txns)
    }
}
