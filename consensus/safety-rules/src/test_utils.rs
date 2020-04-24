// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::persistent_safety_storage::PersistentSafetyStorage;
use consensus_types::{
    accumulator_extension_proof::AccumulatorExtensionProof,
    block::Block,
    common::{Payload, Round},
    quorum_cert::QuorumCert,
    timeout::Timeout,
    vote::Vote,
    vote_data::VoteData,
    vote_proposal::VoteProposal,
};
use libra_crypto::hash::{CryptoHash, TransactionAccumulatorHasher};
use libra_secure_storage::InMemoryStorage;
use libra_types::{
    block_info::BlockInfo,
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    on_chain_config::ValidatorSet,
    validator_info::ValidatorInfo,
    validator_signer::ValidatorSigner,
    waypoint::Waypoint,
};
use std::{
    collections::BTreeMap,
    time::{SystemTime, UNIX_EPOCH},
};

pub type Proof = AccumulatorExtensionProof<TransactionAccumulatorHasher>;

pub fn empty_proof() -> Proof {
    Proof::new(vec![], 0, vec![])
}

pub fn make_proposal_with_qc_and_proof<P: Payload>(
    payload: P,
    round: Round,
    proof: Proof,
    qc: QuorumCert,
    validator_signer: &ValidatorSigner,
) -> VoteProposal<P> {
    VoteProposal::<P>::new(
        proof,
        Block::<P>::new_proposal(
            payload,
            round,
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            qc,
            validator_signer,
        ),
        None,
    )
}

pub fn make_proposal_with_qc<P: Payload>(
    round: Round,
    qc: QuorumCert,
    validator_signer: &ValidatorSigner,
) -> VoteProposal<P> {
    make_proposal_with_qc_and_proof(P::default(), round, empty_proof(), qc, validator_signer)
}

pub fn make_proposal_with_parent<P: Payload>(
    payload: P,
    round: Round,
    parent: &VoteProposal<P>,
    committed: Option<&VoteProposal<P>>,
    validator_signer: &ValidatorSigner,
) -> VoteProposal<P> {
    let parent_output = parent
        .accumulator_extension_proof()
        .verify(
            parent
                .block()
                .quorum_cert()
                .certified_block()
                .executed_state_id(),
        )
        .unwrap();

    let proof = Proof::new(
        parent_output.frozen_subtree_roots().clone(),
        parent_output.num_leaves(),
        vec![Timeout::new(0, round).hash()],
    );

    let vote_data = VoteData::new(
        parent
            .block()
            .gen_block_info(parent_output.root_hash(), parent_output.version(), None),
        parent.block().quorum_cert().certified_block().clone(),
    );

    let ledger_info = match committed {
        Some(committed) => {
            let tree = committed
                .accumulator_extension_proof()
                .verify(
                    committed
                        .block()
                        .quorum_cert()
                        .certified_block()
                        .executed_state_id(),
                )
                .unwrap();
            let commit_block_info = BlockInfo::new(
                committed.block().epoch(),
                committed.block().round(),
                committed.block().id(),
                tree.root_hash(),
                tree.version(),
                committed.block().timestamp_usecs(),
                None,
            );
            LedgerInfo::new(commit_block_info, vote_data.hash())
        }
        None => LedgerInfo::new(BlockInfo::empty(), vote_data.hash()),
    };

    let vote = Vote::new(
        vote_data.clone(),
        validator_signer.author(),
        ledger_info,
        validator_signer,
    );

    let mut ledger_info_with_signatures =
        LedgerInfoWithSignatures::new(vote.ledger_info().clone(), BTreeMap::new());

    ledger_info_with_signatures.add_signature(vote.author(), vote.signature().clone());

    let qc = QuorumCert::new(vote_data, ledger_info_with_signatures);

    make_proposal_with_qc_and_proof(payload, round, proof, qc, validator_signer)
}

pub fn validator_signers_to_ledger_info(signers: &[&ValidatorSigner]) -> LedgerInfo {
    let infos = signers
        .iter()
        .map(|v| ValidatorInfo::new_with_test_network_keys(v.author(), v.public_key(), 1));
    let validator_set = ValidatorSet::new(infos.collect());
    LedgerInfo::mock_genesis(Some(validator_set))
}

pub fn validator_signers_to_waypoints(signers: &[&ValidatorSigner]) -> Waypoint {
    let li = validator_signers_to_ledger_info(signers);
    Waypoint::new_epoch_boundary(&li).unwrap()
}

pub fn test_storage(signer: &ValidatorSigner) -> PersistentSafetyStorage {
    let waypoint = validator_signers_to_waypoints(&[signer]);
    let storage = InMemoryStorage::new_storage();
    PersistentSafetyStorage::initialize(storage, signer.private_key().clone(), waypoint)
}
