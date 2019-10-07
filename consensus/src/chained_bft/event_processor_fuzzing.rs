use crate::{
    chained_bft::{
        block_storage::BlockStore,
        consensus_types::proposal_msg::{ProposalMsg, ProposalUncheckedSignatures},
        epoch_manager::EpochManager,
        event_processor::EventProcessor,
        liveness::{
            pacemaker::{ExponentialTimeInterval, NewRoundEvent, NewRoundReason, Pacemaker},
            pacemaker_timeout_manager::HighestTimeoutCertificates,
            proposal_generator::ProposalGenerator,
            rotating_proposer_election::RotatingProposer,
        },
        network::ConsensusNetworkImpl,
        persistent_storage::{PersistentStorage, RecoveryData},
        safety::safety_rules::SafetyRules,
        test_utils::{EmptyStateComputer, MockStorage, MockTransactionManager, TestPayload},
    },
    util::mock_time_service::SimulatedTimeService,
};
use futures::{channel::mpsc, executor::block_on};
use lazy_static::lazy_static;
use libra_types::crypto_proxies::{LedgerInfoWithSignatures, ValidatorSigner, ValidatorVerifier};
use network::{
    proto::Proposal,
    validator_network::{ConsensusNetworkEvents, ConsensusNetworkSender},
};
use prost::Message as _;
use prost_ext::MessageExt;
use std::convert::TryFrom;
use std::sync::Arc;
use tokio::runtime::Runtime;

// This generates a proposal for round 1
pub fn generate_corpus_proposal() -> Vec<u8> {
    let event_processor = create_node_for_fuzzing();
    block_on(async {
        let proposal = event_processor
            .generate_proposal(NewRoundEvent {
                round: 1,
                reason: NewRoundReason::QCReady,
                timeout: std::time::Duration::new(5, 0),
            })
            .await;
        // serialize and return proposal
        let proposal = proposal.unwrap();
        Proposal::from(proposal).to_bytes().unwrap().to_vec()
    })
}

// optimization for the fuzzer
lazy_static! {
    static ref STATIC_RUNTIME: Runtime = Runtime::new().unwrap();
    static ref FUZZING_SIGNER: ValidatorSigner = ValidatorSigner::from_int(1);
}

// helpers
fn build_empty_store(
    signer: ValidatorSigner,
    storage: Arc<dyn PersistentStorage<TestPayload>>,
    initial_data: RecoveryData<TestPayload>,
) -> Arc<BlockStore<TestPayload>> {
    let (_commit_cb_sender, _commit_cb_receiver) = mpsc::unbounded::<LedgerInfoWithSignatures>();

    Arc::new(block_on(BlockStore::new(
        storage,
        initial_data,
        signer,
        Arc::new(EmptyStateComputer),
        true,
        10, // max pruned blocks in mem
    )))
}

// TODO: MockStorage -> EmptyStorage
fn create_pacemaker() -> Pacemaker {
    let base_timeout = std::time::Duration::new(60, 0);
    let time_interval = Box::new(ExponentialTimeInterval::fixed(base_timeout));
    let (pacemaker_timeout_sender, _) = channel::new_test(1_024);
    let time_service = Arc::new(SimulatedTimeService::new());
    Pacemaker::new(
        MockStorage::<TestPayload>::start_for_testing()
            .0
            .persistent_liveness_storage(),
        time_interval,
        time_service,
        pacemaker_timeout_sender,
        HighestTimeoutCertificates::default(),
    )
}

// Creates an EventProcessor for fuzzing
fn create_node_for_fuzzing() -> EventProcessor<TestPayload> {
    // signer is re-used accross fuzzing runs
    let signer = FUZZING_SIGNER.clone();

    // TODO: remove
    let validator = ValidatorVerifier::new_single(signer.author(), signer.public_key());

    // EpochManager
    let epoch_mgr = Arc::new(EpochManager::new(0, validator));

    // TODO: EmptyStorage
    let (storage, initial_data) = MockStorage::<TestPayload>::start_for_testing();
    let consensus_state = initial_data.state();

    // TODO: remove
    let safety_rules = SafetyRules::new(consensus_state);

    // TODO: mock channels
    let (network_reqs_tx, _network_reqs_rx) = channel::new_test(8);
    let (_consensus_tx, consensus_rx) = channel::new_test(8);
    let network_sender = ConsensusNetworkSender::new(network_reqs_tx);
    let network_events = ConsensusNetworkEvents::new(consensus_rx);
    let network = ConsensusNetworkImpl::new(
        signer.author(),
        network_sender,
        network_events,
        Arc::clone(&epoch_mgr),
    );

    // TODO: mock
    let block_store = build_empty_store(signer.clone(), storage.clone(), initial_data);

    // TODO: remove
    let time_service = Arc::new(SimulatedTimeService::new());

    // TODO: remove
    let proposal_generator = ProposalGenerator::new(
        block_store.clone(),
        Arc::new(MockTransactionManager::new()),
        time_service.clone(),
        1,
        true,
    );

    //
    let pacemaker = create_pacemaker();

    // TODO: have two different nodes, one for proposing, one for accepting a proposal
    let proposer_election = Box::new(RotatingProposer::new(vec![signer.author()], 1));

    // TODO: do we want to fuzz the real StateComputer as well?
    let empty_state_computer = Arc::new(EmptyStateComputer);

    // We do not want to care about the time
    let enforce_increasing_timestamps = false;

    // event processor
    EventProcessor::new(
        Arc::clone(&block_store),
        None,
        pacemaker,
        proposer_election,
        proposal_generator,
        safety_rules,
        empty_state_computer,
        Arc::new(MockTransactionManager::new()),
        network,
        storage.clone(),
        time_service,
        enforce_increasing_timestamps,
        Arc::clone(&epoch_mgr),
    )
}

// This functions fuzzes a Proposal protobuffer (not a ConsensusMsg)
pub fn fuzz_proposal(data: &[u8]) {
    // create node
    let mut event_processor = create_node_for_fuzzing();

    let proposal = match Proposal::decode(data) {
        Ok(xx) => xx,
        Err(_) => {
            if cfg!(test) {
                panic!();
            }
            return;
        }
    };

    let proposal = match ProposalUncheckedSignatures::<TestPayload>::try_from(proposal) {
        Ok(xx) => xx,
        Err(_) => {
            if cfg!(test) {
                panic!();
            }
            return;
        }
    };

    let proposal: ProposalMsg<TestPayload> = proposal.into();

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
