// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chained_bft::test_utils::{self, placeholder_ledger_info};
use consensus_types::{
    block::Block,
    block_info::BlockInfo,
    proposal_msg::{ProposalMsg, ProposalUncheckedSignatures},
    quorum_cert::QuorumCert,
    sync_info::SyncInfo,
    vote_data::VoteData,
    vote_msg::VoteMsg,
};
use libra_types::validator_signer::ValidatorSigner;
use prost::Message;
use prost_ext::MessageExt;
use std::convert::{TryFrom, TryInto};

#[test]
fn test_proto_convert_block() {
    let block: Block<u64> = Block::make_genesis_block();
    let block_proto = network::proto::Block::from(block.clone());
    assert_eq!(block, block_proto.try_into().unwrap());
}

#[test]
fn test_proto_convert_proposal() {
    let genesis_qc = QuorumCert::certificate_for_genesis();
    let proposal = ProposalMsg::new(
        Block::<u64>::make_genesis_block(),
        SyncInfo::new(genesis_qc.clone(), genesis_qc.clone(), None),
    );
    //
    let protoed: network::proto::Proposal = proposal.clone().into();
    let unprotoed: ProposalMsg<u64> = ProposalUncheckedSignatures::<u64>::try_from(protoed)
        .expect("Should convert.")
        .into();
    assert_eq!(proposal, unprotoed);
    //
    let protoed = network::proto::Proposal::from(proposal.clone())
        .to_bytes()
        .unwrap();
    let unprotoed: ProposalMsg<u64> = ProposalUncheckedSignatures::<u64>::try_from(
        network::proto::Proposal::decode(protoed).unwrap(),
    )
    .expect("Should convert.")
    .into();
    assert_eq!(proposal, unprotoed);
}

#[test]
fn test_proto_convert_vote() {
    let signer = ValidatorSigner::random(None);
    let vote_msg = VoteMsg::new(
        VoteData::new(BlockInfo::random(1), BlockInfo::random(0)),
        signer.author(),
        placeholder_ledger_info(),
        &signer,
        test_utils::placeholder_sync_info(),
    );
    let vote_msg_proto = network::proto::VoteMsg::from(vote_msg.clone());
    assert_eq!(vote_msg, vote_msg_proto.try_into().unwrap());
}
