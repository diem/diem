// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::proto::transaction::SignedTransaction;
use crypto::{hash::TestOnlyHash, HashValue};

/// Used to get the digest of a set of signed transactions.  This is used by a validator
/// to sign a block and to verify the signatures of other validators on a block
pub fn get_signed_transactions_digest(signed_txns: &[SignedTransaction]) -> HashValue {
    let mut signatures = vec![];
    for transaction in signed_txns {
        signatures.extend_from_slice(&transaction.sender_signature);
    }
    signatures.test_only_hash()
}
