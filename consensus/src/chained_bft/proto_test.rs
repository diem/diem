// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chained_bft::{
    consensus_types::{
        block::Block,
        proposal_msg::{ProposalMsg, ProposalUncheckedSignatures},
        quorum_cert::QuorumCert,
        sync_info::SyncInfo,
        vote_data::VoteData,
        vote_msg::VoteMsg,
    },
    test_utils::placeholder_ledger_info,
};
use crypto::HashValue;
use executor::ExecutedState;
use proto_conv::{
    test_helper::assert_protobuf_encode_decode, FromProto, FromProtoBytes, IntoProto,
    IntoProtoBytes,
};
use types::validator_signer::ValidatorSigner;

#[test]
fn test_proto_convert_block() {
    let block: Block<u64> = Block::make_genesis_block();
    assert_protobuf_encode_decode(&block);
}

#[test]
fn test_proto_convert_proposal() {
    let genesis_qc = QuorumCert::certificate_for_genesis();
    let proposal = ProposalMsg::new(
        Block::<u64>::make_genesis_block(),
        SyncInfo::new(genesis_qc.clone(), genesis_qc.clone(), None),
    );
    //
    let protoed = proposal.clone().into_proto();
    let unprotoed: ProposalMsg<u64> = ProposalUncheckedSignatures::<u64>::from_proto(protoed)
        .expect("Should convert.")
        .into();
    assert_eq!(proposal, unprotoed);
    //
    let protoed = proposal
        .clone()
        .into_proto_bytes()
        .expect("Should convert.");
    let unprotoed: ProposalMsg<u64> =
        ProposalUncheckedSignatures::<u64>::from_proto_bytes(&protoed)
            .expect("Should convert.")
            .into();
    assert_eq!(proposal, unprotoed);
}

#[test]
fn test_proto_convert_vote() {
    let signer = ValidatorSigner::random(None);
    let vote = VoteMsg::new(
        VoteData::new(
            HashValue::random(),
            ExecutedState::state_for_genesis().state_id,
            1,
            HashValue::random(),
            0,
            HashValue::random(),
            0,
        ),
        signer.author(),
        placeholder_ledger_info(),
        &signer,
    );
    assert_protobuf_encode_decode(&vote);
}
