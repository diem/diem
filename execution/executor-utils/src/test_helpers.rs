// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use executor_types::StateComputeResult;
use libra_config::config::NodeConfig;
use libra_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    hash::CryptoHash,
    HashValue,
};
use libra_types::{
    account_address::AccountAddress,
    block_info::BlockInfo,
    block_metadata::BlockMetadata,
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    test_helpers::transaction_test_helpers::get_test_signed_txn,
    transaction::{Script, Transaction},
    validator_signer::ValidatorSigner,
};

pub fn gen_block_id(index: u8) -> HashValue {
    HashValue::new([index; HashValue::LENGTH])
}

pub fn gen_ledger_info_with_sigs(
    epoch: u64,
    output: StateComputeResult,
    commit_block_id: HashValue,
    signer: Vec<&ValidatorSigner>,
) -> LedgerInfoWithSignatures {
    let ledger_info = LedgerInfo::new(
        BlockInfo::new(
            epoch,
            0, /* round */
            commit_block_id,
            output.root_hash(),
            output.version(),
            0, /* timestamp */
            output.validators().clone(),
        ),
        HashValue::zero(),
    );
    let signatures = signer
        .iter()
        .map(|s| (s.author(), s.sign_message(ledger_info.hash())))
        .collect();
    LedgerInfoWithSignatures::new(ledger_info, signatures)
}

pub fn extract_signer(config: &mut NodeConfig) -> ValidatorSigner {
    ValidatorSigner::new(
        config.validator_network.as_ref().unwrap().peer_id,
        config
            .test
            .as_mut()
            .unwrap()
            .consensus_keypair
            .as_mut()
            .unwrap()
            .take_private()
            .unwrap(),
    )
}

pub fn gen_block_metadata(index: u8, proposer: AccountAddress) -> BlockMetadata {
    BlockMetadata::new(
        gen_block_id(index),
        index as u64,
        index as u64,
        vec![],
        proposer,
    )
}

pub fn get_test_signed_transaction(
    sender: AccountAddress,
    sequence_number: u64,
    private_key: Ed25519PrivateKey,
    public_key: Ed25519PublicKey,
    program: Option<Script>,
) -> Transaction {
    Transaction::UserTransaction(get_test_signed_txn(
        sender,
        sequence_number,
        &private_key,
        public_key,
        program,
    ))
}
