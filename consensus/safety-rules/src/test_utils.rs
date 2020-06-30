// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::persistent_safety_storage::PersistentSafetyStorage;
use consensus_types::{
    block::Block,
    common::{Payload, Round},
    quorum_cert::QuorumCert,
    timeout::Timeout,
    vote::Vote,
    vote_data::VoteData,
    vote_proposal::{MaybeSignedVoteProposal, VoteProposal},
};
use libra_crypto::{
    ed25519::Ed25519PrivateKey,
    hash::{CryptoHash, TransactionAccumulatorHasher},
    traits::SigningKey,
    Uniform,
};
use libra_secure_storage::{InMemoryStorage, Storage};
use libra_types::{
    block_info::BlockInfo,
    epoch_change::EpochChangeProof,
    epoch_state::EpochState,
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    on_chain_config::ValidatorSet,
    proof::AccumulatorExtensionProof,
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

pub fn make_genesis(signer: &ValidatorSigner) -> (EpochChangeProof, QuorumCert) {
    let validator_info =
        ValidatorInfo::new_with_test_network_keys(signer.author(), signer.public_key(), 1);
    let validator_set = ValidatorSet::new(vec![validator_info]);
    let li = LedgerInfo::mock_genesis(Some(validator_set));
    let block = Block::make_genesis_block_from_ledger_info(&li);
    let qc = QuorumCert::certificate_for_genesis_from_ledger_info(&li, block.id());
    let lis = LedgerInfoWithSignatures::new(li, BTreeMap::new());
    let proof = EpochChangeProof::new(vec![lis], false);
    (proof, qc)
}

pub fn make_proposal_with_qc_and_proof(
    payload: Payload,
    round: Round,
    proof: Proof,
    qc: QuorumCert,
    validator_signer: &ValidatorSigner,
    exec_key: Option<&Ed25519PrivateKey>,
) -> MaybeSignedVoteProposal {
    let vote_proposal = VoteProposal::new(
        proof,
        Block::new_proposal(
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
    );
    let signature = exec_key.map(|key| key.sign_message(&vote_proposal.hash()));
    MaybeSignedVoteProposal {
        vote_proposal,
        signature,
    }
}

pub fn make_proposal_with_qc(
    round: Round,
    qc: QuorumCert,
    validator_signer: &ValidatorSigner,
    exec_key: Option<&Ed25519PrivateKey>,
) -> MaybeSignedVoteProposal {
    make_proposal_with_qc_and_proof(vec![], round, empty_proof(), qc, validator_signer, exec_key)
}

pub fn make_proposal_with_parent_and_overrides(
    payload: Payload,
    round: Round,
    parent: &MaybeSignedVoteProposal,
    committed: Option<&MaybeSignedVoteProposal>,
    validator_signer: &ValidatorSigner,
    epoch: Option<u64>,
    next_epoch_state: Option<EpochState>,
    exec_key: Option<&Ed25519PrivateKey>,
) -> MaybeSignedVoteProposal {
    let block_epoch = match epoch {
        Some(e) => e,
        _ => parent.block().epoch(),
    };

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

    let proposed_block = BlockInfo::new(
        block_epoch,
        parent.block().round(),
        parent.block().id(),
        parent_output.root_hash(),
        parent_output.version(),
        parent.block().timestamp_usecs(),
        None,
    );

    let vote_data = VoteData::new(
        proposed_block,
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
                next_epoch_state,
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

    make_proposal_with_qc_and_proof(payload, round, proof, qc, validator_signer, exec_key)
}

pub fn make_proposal_with_parent(
    payload: Payload,
    round: Round,
    parent: &MaybeSignedVoteProposal,
    committed: Option<&MaybeSignedVoteProposal>,
    validator_signer: &ValidatorSigner,
    exec_key: Option<&Ed25519PrivateKey>,
) -> MaybeSignedVoteProposal {
    make_proposal_with_parent_and_overrides(
        payload,
        round,
        parent,
        committed,
        validator_signer,
        None,
        None,
        exec_key,
    )
}

pub fn validator_signers_to_ledger_info(signers: &[&ValidatorSigner]) -> LedgerInfo {
    let infos = signers
        .iter()
        .map(|v| ValidatorInfo::new_with_test_network_keys(v.author(), v.public_key(), 1));
    let validator_set = ValidatorSet::new(infos.collect());
    LedgerInfo::mock_genesis(Some(validator_set))
}

pub fn validator_signers_to_waypoint(signers: &[&ValidatorSigner]) -> Waypoint {
    let li = validator_signers_to_ledger_info(signers);
    Waypoint::new_epoch_boundary(&li).unwrap()
}

pub fn test_storage(signer: &ValidatorSigner) -> PersistentSafetyStorage {
    let waypoint = validator_signers_to_waypoint(&[signer]);
    let storage = Storage::from(InMemoryStorage::new());
    PersistentSafetyStorage::initialize(
        storage,
        signer.author(),
        signer.private_key().clone(),
        Ed25519PrivateKey::generate_for_testing(),
        waypoint,
    )
}
