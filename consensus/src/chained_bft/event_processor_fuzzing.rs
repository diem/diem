// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::{
        block_storage::BlockStore,
        event_processor::EventProcessor,
        liveness::{
            pacemaker::{ExponentialTimeInterval, NewRoundEvent, NewRoundReason, Pacemaker},
            proposal_generator::ProposalGenerator,
            rotating_proposer_election::RotatingProposer,
        },
        network::NetworkSender,
        persistent_liveness_storage::{PersistentLivenessStorage, RecoveryData},
        test_utils::{EmptyStateComputer, MockStorage, MockTransactionManager, TestPayload},
    },
    util::mock_time_service::SimulatedTimeService,
};
use channel::{self, libra_channel, message_queues::QueueStyle};
use consensus_types::proposal_msg::ProposalMsg;
use futures::{channel::mpsc, executor::block_on};
use libra_types::crypto_proxies::{LedgerInfoWithSignatures, ValidatorSigner, ValidatorVerifier};
use network::validator_network::ConsensusNetworkSender;
use once_cell::sync::Lazy;
use safety_rules::{PersistentSafetyStorage, SafetyRules};
use std::num::NonZeroUsize;
use std::sync::Arc;
use tokio::runtime::Runtime;

// This generates a proposal for round 1
pub fn generate_corpus_proposal() -> Vec<u8> {
    let mut event_processor = create_node_for_fuzzing();
    block_on(async {
        let proposal = event_processor
            .generate_proposal(NewRoundEvent {
                round: 1,
                reason: NewRoundReason::QCReady,
                timeout: std::time::Duration::new(5, 0),
            })
            .await;
        // serialize and return proposal
        lcs::to_bytes(&proposal.unwrap()).unwrap()
    })
}

// optimization for the fuzzer
static STATIC_RUNTIME: Lazy<Runtime> = Lazy::new(|| Runtime::new().unwrap());
static FUZZING_SIGNER: Lazy<ValidatorSigner> = Lazy::new(|| ValidatorSigner::from_int(1));

// helpers
fn build_empty_store(
    storage: Arc<dyn PersistentLivenessStorage<TestPayload>>,
    initial_data: RecoveryData<TestPayload>,
) -> Arc<BlockStore<TestPayload>> {
    let (_commit_cb_sender, _commit_cb_receiver) = mpsc::unbounded::<LedgerInfoWithSignatures>();

    Arc::new(BlockStore::new(
        storage,
        initial_data,
        Arc::new(EmptyStateComputer),
        10, // max pruned blocks in mem
    ))
}

// TODO: MockStorage -> EmptyStorage
fn create_pacemaker() -> Pacemaker {
    let base_timeout = std::time::Duration::new(60, 0);
    let time_interval = Box::new(ExponentialTimeInterval::fixed(base_timeout));
    let (pacemaker_timeout_sender, _) = channel::new_test(1_024);
    let time_service = Arc::new(SimulatedTimeService::new());
    Pacemaker::new(time_interval, time_service, pacemaker_timeout_sender)
}

// Creates an EventProcessor for fuzzing
fn create_node_for_fuzzing() -> EventProcessor<TestPayload> {
    // signer is re-used accross fuzzing runs
    let signer = FUZZING_SIGNER.clone();

    // TODO: remove
    let validator = ValidatorVerifier::new_single(signer.author(), signer.public_key());
    let validator_set = (&validator).into();

    // TODO: EmptyStorage
    let (initial_data, storage) = MockStorage::<TestPayload>::start_for_testing(validator_set);

    // TODO: remove
    let safety_rules = SafetyRules::new(
        signer.author(),
        PersistentSafetyStorage::in_memory(signer.private_key().clone()),
    );

    // TODO: mock channels
    let (network_reqs_tx, _network_reqs_rx) =
        libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(8).unwrap(), None);
    let (conn_mgr_reqs_tx, _conn_mgr_reqs_rx) = channel::new_test(8);
    let network_sender = ConsensusNetworkSender::new(network_reqs_tx, conn_mgr_reqs_tx);
    let (self_sender, _self_receiver) = channel::new_test(8);
    let network = NetworkSender::new(
        signer.author(),
        network_sender,
        self_sender,
        initial_data.validators(),
    );

    let validators = initial_data.validators();

    // TODO: mock
    let block_store = build_empty_store(storage.clone(), initial_data);

    // TODO: remove
    let time_service = Arc::new(SimulatedTimeService::new());

    // TODO: remove
    let proposal_generator = ProposalGenerator::new(
        signer.author(),
        block_store.clone(),
        Box::new(MockTransactionManager::new().0),
        time_service.clone(),
        1,
    );

    //
    let pacemaker = create_pacemaker();

    // TODO: have two different nodes, one for proposing, one for accepting a proposal
    let proposer_election = Box::new(RotatingProposer::new(vec![signer.author()], 1));

    // event processor
    EventProcessor::new(
        Arc::clone(&block_store),
        None,
        pacemaker,
        proposer_election,
        proposal_generator,
        Box::new(safety_rules),
        Box::new(MockTransactionManager::new().0),
        network,
        storage,
        time_service,
        validators,
    )
}

// This functions fuzzes a Proposal protobuffer (not a ConsensusMsg)
pub fn fuzz_proposal(data: &[u8]) {
    // create node
    let mut event_processor = create_node_for_fuzzing();

    let proposal: ProposalMsg<TestPayload> = match lcs::from_bytes(data) {
        Ok(xx) => xx,
        Err(_) => {
            if cfg!(test) {
                panic!();
            }
            return;
        }
    };

    let proposal = match proposal.verify_well_formed() {
        Ok(xx) => xx,
        Err(_) => {
            if cfg!(test) {
                panic!();
            }
            return;
        }
    };

    block_on(async move {
        // TODO: make sure this obtains a vote when testing
        // TODO: make sure that if this obtains a vote, it's for round 1, etc.
        event_processor.process_proposal_msg(proposal).await;
    });
}

// This test is here so that the fuzzer can be maintained
#[test]
fn test_consensus_proposal_fuzzer() {
    // generate a proposal
    let proposal = generate_corpus_proposal();
    // successfully parse it
    fuzz_proposal(&proposal);
}
