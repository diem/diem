// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module provides helpers to initialize [`LibraDB`] with fake generic state in tests.

use crate::LibraDB;
use anyhow::Result;
use lazy_static::lazy_static;
use libra_crypto::{
    ed25519::*,
    hash::{CryptoHash, ACCUMULATOR_PLACEHOLDER_HASH, GENESIS_BLOCK_ID},
    HashValue,
};
use libra_types::block_info::BlockInfo;
use libra_types::validator_set::ValidatorSet;
use libra_types::{
    account_address::AccountAddress,
    account_state_blob::AccountStateBlob,
    crypto_proxies::LedgerInfoWithSignatures,
    ledger_info::LedgerInfo,
    proof::SparseMerkleLeafNode,
    transaction::{RawTransaction, Script, Transaction, TransactionInfo, TransactionToCommit},
    vm_error::StatusCode,
};
use rand::{
    rngs::{OsRng, StdRng},
    Rng, SeedableRng,
};
use std::collections::{BTreeMap, HashMap};

fn gen_mock_genesis() -> (
    TransactionInfo,
    LedgerInfoWithSignatures,
    TransactionToCommit,
) {
    let mut seed_rng = OsRng::new().expect("can't access OsRng");
    let seed_buf: [u8; 32] = seed_rng.gen();
    let mut rng = StdRng::from_seed(seed_buf);
    let (privkey, pubkey) = compat::generate_keypair(&mut rng);
    let some_addr = AccountAddress::from_public_key(&pubkey);
    let raw_txn = RawTransaction::new_script(
        some_addr,
        /* sequence_number = */ 0,
        Script::new(vec![], vec![]),
        /* max_gas_amount = */ 0,
        /* gas_unit_price = */ 0,
        /* expiration_time = */ std::time::Duration::new(0, 0),
    );
    let genesis_txn = Transaction::UserTransaction(
        raw_txn
            .sign(&privkey, pubkey)
            .expect("Signing failed.")
            .into_inner(),
    );
    let txn_hash = genesis_txn.hash();

    let some_blob = AccountStateBlob::from(vec![1u8]);
    let account_states = vec![(some_addr, some_blob.clone())]
        .into_iter()
        .collect::<HashMap<_, _>>();

    let txn_to_commit = TransactionToCommit::new(
        genesis_txn,
        account_states.clone(),
        vec![], /* events */
        0,      /* gas_used */
        StatusCode::EXECUTED,
    );

    // The genesis state tree has a single leaf node, so the root hash is the hash of that node.
    let state_root_hash = SparseMerkleLeafNode::new(some_addr.hash(), some_blob.hash()).hash();
    let txn_info = TransactionInfo::new(
        txn_hash,
        state_root_hash,
        *ACCUMULATOR_PLACEHOLDER_HASH,
        0,
        StatusCode::EXECUTED,
    );

    let ledger_info = LedgerInfo::new(
        BlockInfo::new(
            0,
            0,
            *GENESIS_BLOCK_ID,
            txn_info.hash(),
            0,
            0,
            Some(ValidatorSet::new(Vec::new())),
        ),
        HashValue::random(),
    );
    let ledger_info_with_sigs =
        LedgerInfoWithSignatures::new(ledger_info, BTreeMap::new() /* signatures */);

    (txn_info, ledger_info_with_sigs, txn_to_commit)
}

lazy_static! {
    /// Tuple containing information about the mock genesis state.
    ///
    /// Tests can use this as input to generate the mock genesis state and verify against it. It is
    /// defined as ([`TransactionInfo`], [`LedgerInfoWithSignatures`],
    /// [`TransactionToCommit`]):
    ///
    ///   - [`TransactionToCommit`] is the mock genesis transaction.
    ///   - [`TransactionInfo`] is calculated out of the mock genesis transaction.
    ///   - [`LedgerInfoWithSignatures`] contains the hash of the above mock transaction info and
    /// other mocked information including validator signatures.
    pub static ref GENESIS_INFO: (
        TransactionInfo,
        LedgerInfoWithSignatures,
        TransactionToCommit
    ) = gen_mock_genesis();
}

/// This creates an empty db at input `dir` and initializes it with mock genesis info.
///
/// The resulting db will have only one transaction at version 0 (the mock genesis transaction) and
/// related outputs (the mock genesis state) in it.
pub fn db_with_mock_genesis<P: AsRef<std::path::Path>>(dir: &P) -> Result<LibraDB> {
    let genesis_ledger_info_with_sigs = GENESIS_INFO.1.clone();
    let genesis_txn = GENESIS_INFO.2.clone();

    let db = LibraDB::new(dir);
    db.save_transactions(
        &[genesis_txn],
        0, /* first_version */
        &Some(genesis_ledger_info_with_sigs),
    )?;
    Ok(db)
}
