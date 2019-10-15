// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{quorum_cert::QuorumCert, vote_data::VoteData};
use crypto::{hash::CryptoHash, HashValue};
use executor::ExecutedState;
use libra_types::{
    crypto_proxies::{LedgerInfoWithSignatures, ValidatorSigner},
    ledger_info::LedgerInfo,
};
use std::collections::BTreeMap;

pub fn placeholder_ledger_info() -> LedgerInfo {
    LedgerInfo::new(
        0,
        HashValue::zero(),
        HashValue::zero(),
        HashValue::zero(),
        0,
        0,
        None,
    )
}

pub fn placeholder_certificate_for_block(
    signers: Vec<&ValidatorSigner>,
    certified_block_id: HashValue,
    certified_block_round: u64,
    certified_parent_block_id: HashValue,
    certified_parent_block_round: u64,
    is_commit_guaranteed: bool,
) -> QuorumCert {
    // Assuming executed state to be Genesis state.
    let certified_block_state = ExecutedState::state_for_genesis();
    let consensus_data_hash = VoteData::vote_digest(
        certified_block_id,
        certified_block_state.state_id,
        certified_block_round,
        certified_parent_block_id,
        certified_parent_block_round,
    );

    // This ledger info doesn't carry any meaningful information: it is all zeros except for
    // the consensus data hash that carries the actual vote.
    let mut ledger_info_placeholder = placeholder_ledger_info();
    ledger_info_placeholder.set_consensus_data_hash(consensus_data_hash);

    if is_commit_guaranteed {
        // Required to set consensus block id for the ledger info for testing restartability.
        ledger_info_placeholder.set_consensus_block_id(certified_parent_block_id);
    }

    let mut signatures = BTreeMap::new();
    for signer in signers {
        let li_sig = signer
            .sign_message(ledger_info_placeholder.hash())
            .expect("Failed to sign LedgerInfo");
        signatures.insert(signer.author(), li_sig);
    }

    QuorumCert::new(
        VoteData::new(
            certified_block_id,
            certified_block_state.state_id,
            certified_block_round,
            certified_parent_block_id,
            certified_parent_block_round,
        ),
        LedgerInfoWithSignatures::new(ledger_info_placeholder, signatures),
    )
}
