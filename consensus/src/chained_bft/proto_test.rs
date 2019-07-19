// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::{
        consensus_types::{block::Block, quorum_cert::QuorumCert},
        liveness::proposer_election::ProposalInfo,
        safety::vote_msg::VoteMsg,
        test_utils::placeholder_ledger_info,
    },
    state_replication::ExecutedState,
};
use crypto::HashValue;
use nextgen_crypto::ed25519::Ed25519PrivateKey;
use proto_conv::test_helper::assert_protobuf_encode_decode;
use types::validator_signer::ValidatorSigner;

#[test]
fn test_proto_convert_block() {
    let block: Block<u64> = Block::make_genesis_block();
    assert_protobuf_encode_decode(&block);
}

#[test]
fn test_proto_convert_proposal() {
    let author = ValidatorSigner::<Ed25519PrivateKey>::random(None).author();
    let proposal = ProposalInfo {
        proposal: Block::<u64>::make_genesis_block(),
        proposer_info: author,
        timeout_certificate: None,
        highest_ledger_info: QuorumCert::certificate_for_genesis(),
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
        signer.author(),
        placeholder_ledger_info(),
        &signer,
    );
    assert_protobuf_encode_decode(&vote);
}
