// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::libra_vm::{chunk_block_transactions, TransactionBlock};
use libra_types::transaction::{Transaction, TransactionPayload};
use proptest::{collection::vec, prelude::*};

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
        let result = reconstruct_transaction_vec(chunk_block_transactions(txns.clone()));
        prop_assert_eq!(result.len(), txns.len());
        let check = txns.iter().zip(result.iter()).all(|(l, r)| {
            if let Transaction::UserTransaction(txn) = l {
                if let TransactionPayload::WriteSet(ws_l) = txn.payload() {
                    return match r {
                        Transaction::WriteSet(ws_r) => ws_l == ws_r,
                        _ => false,
                    }
                }
            }
            l == r
        });
        prop_assert!(check);
    }
}
