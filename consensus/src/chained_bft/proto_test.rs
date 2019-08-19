// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::{
        consensus_types::{
            block::Block, proposal_msg::ProposalMsg, quorum_cert::QuorumCert, sync_info::SyncInfo,
        },
        safety::vote_msg::VoteMsg,
        test_utils::placeholder_ledger_info,
    },
    state_replication::ExecutedState,
};
use crypto::{ed25519::Ed25519PrivateKey, HashValue};
use proto_conv::test_helper::assert_protobuf_encode_decode;
use types::validator_signer::ValidatorSigner;

#[test]
fn test_proto_convert_block() {
    let block: Block<u64> = Block::make_genesis_block();
    assert_protobuf_encode_decode(&block);
}

#[test]
fn test_proto_convert_proposal() {
    let genesis_qc = QuorumCert::certificate_for_genesis();
    let proposal = ProposalMsg {
        proposal: Block::<u64>::make_genesis_block(),
        sync_info: SyncInfo::new(genesis_qc.clone(), genesis_qc.clone(), None),
    };
    assert_protobuf_encode_decode(&proposal);
}

#[test]
fn test_proto_convert_vote() {
    let signer = ValidatorSigner::<Ed25519PrivateKey>::random(None);
    let vote = VoteMsg::new(
        HashValue::random(),
        ExecutedState::state_for_genesis(),
        1,
        HashValue::random(),
        0,
        HashValue::random(),
        0,
        signer.author(),
        placeholder_ledger_info(),
        &signer,
    );
    assert_protobuf_encode_decode(&vote);
}
