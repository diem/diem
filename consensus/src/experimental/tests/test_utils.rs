// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    block_storage::BlockStore,
    experimental::{
        commit_phase::{CommitChannelType, CommitPhase},
        execution_phase::ExecutionChannelType,
        ordering_state_computer::OrderingStateComputer,
    },
    metrics_safety_rules::MetricsSafetyRules,
    network::NetworkSender,
    network_interface::{ConsensusMsg, ConsensusNetworkSender},
    round_manager::VerifiedEvent,
    state_replication::StateComputer,
    test_utils::MockStorage,
    util::time_service::ClockTimeService,
};
use channel::{diem_channel, message_queues::QueueStyle, Receiver, Sender};
use consensus_types::{
    block::{block_test_utils::certificate_for_genesis, Block},
    executed_block::ExecutedBlock,
};
use diem_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519Signature},
    hash::ACCUMULATOR_PLACEHOLDER_HASH,
    HashValue, Uniform,
};
use diem_infallible::Mutex;
use diem_secure_storage::Storage;
use diem_types::{
    account_address::AccountAddress,
    block_info::BlockInfo,
    epoch_state::EpochState,
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    validator_signer::ValidatorSigner,
    validator_verifier::{random_validator_verifier, ValidatorVerifier},
    waypoint::Waypoint,
};
use executor_types::StateComputeResult;
use network::{
    peer_manager::{ConnectionRequestSender, PeerManagerRequestSender},
    protocols::network::{Event, NewNetworkSender},
};
use safety_rules::{PersistentSafetyStorage, SafetyRulesManager};
use std::{
    collections::BTreeMap,
    sync::{atomic::AtomicU64, Arc},
};
use tokio::runtime::Runtime;

pub fn prepare_commit_phase_with_block_store_state_computer(
    runtime: &Runtime,
    block_store_state_computer: Arc<dyn StateComputer>,
    channel_size: usize,
) -> (
    Sender<CommitChannelType>,
    Sender<VerifiedEvent>,
    Receiver<ExecutionChannelType>,
    Receiver<Event<ConsensusMsg>>,
    Arc<Mutex<MetricsSafetyRules>>,
    Vec<ValidatorSigner>,
    Arc<OrderingStateComputer>,
    ValidatorVerifier,
    CommitPhase,
    Arc<BlockStore>,
) {
    let num_nodes = 1;

    // constants
    let back_pressure = Arc::new(AtomicU64::new(0));

    // environment setup
    let (signers, validators) = random_validator_verifier(num_nodes, None, false);
    let validator_set = (&validators).into();
    let signer = &signers[0];

    let waypoint =
        Waypoint::new_epoch_boundary(&LedgerInfo::mock_genesis(Some(validator_set))).unwrap();

    let safety_storage = PersistentSafetyStorage::initialize(
        Storage::from(diem_secure_storage::InMemoryStorage::new()),
        signer.author(),
        signer.private_key().clone(),
        Ed25519PrivateKey::generate_for_testing(),
        waypoint,
        true,
    );
    let safety_rules_manager = SafetyRulesManager::new_local(safety_storage, false, false, true);

    let (initial_data, storage) = MockStorage::start_for_testing((&validators).into());
    let epoch_state = EpochState {
        epoch: 1,
        verifier: storage.get_validator_set().into(),
    };
    let validators = epoch_state.verifier.clone();
    let (network_reqs_tx, _network_reqs_rx) = diem_channel::new(QueueStyle::FIFO, 8, None);
    let (connection_reqs_tx, _) = diem_channel::new(QueueStyle::FIFO, 8, None);

    let network_sender = ConsensusNetworkSender::new(
        PeerManagerRequestSender::new(network_reqs_tx),
        ConnectionRequestSender::new(connection_reqs_tx),
    );
    let author = signer.author();

    let (self_loop_tx, self_loop_rx) = channel::new_test(1000);
    let network = NetworkSender::new(author, network_sender, self_loop_tx, validators);

    let (commit_result_tx, commit_result_rx) =
        channel::new_test::<ExecutionChannelType>(channel_size);
    let state_computer = Arc::new(OrderingStateComputer::new(
        commit_result_tx,
        block_store_state_computer.clone(),
    ));

    let time_service = Arc::new(ClockTimeService::new(runtime.handle().clone()));

    let block_store = Arc::new(BlockStore::new(
        storage.clone(),
        initial_data,
        block_store_state_computer,
        0, // max pruned blocks in mem
        time_service,
    ));

    let mut safety_rules = MetricsSafetyRules::new(safety_rules_manager.client(), storage);
    safety_rules.perform_initialize().unwrap();

    let safety_rules_container = Arc::new(Mutex::new(safety_rules));

    // setting up channels
    let (commit_tx, commit_rx) = channel::new_test::<CommitChannelType>(channel_size);

    let (msg_tx, msg_rx) = channel::new_test::<VerifiedEvent>(channel_size);

    let commit_phase = CommitPhase::new(
        commit_rx,
        state_computer.clone(),
        msg_rx,
        epoch_state.verifier.clone(),
        safety_rules_container.clone(),
        author,
        back_pressure,
        network,
    );

    (
        commit_tx,        // channel to pass executed blocks into the commit phase
        msg_tx,           // channel to pass commit messages into the commit phase
        commit_result_rx, // channel to receive commit result from the commit phase
        self_loop_rx,     // channel to receive message from the commit phase itself
        safety_rules_container,
        signers,
        state_computer,
        epoch_state.verifier,
        commit_phase,
        block_store,
    )
}

pub fn prepare_executed_blocks_with_ledger_info(
    signer: &ValidatorSigner,
    executed_hash: HashValue,
    consensus_hash: HashValue,
) -> (Vec<ExecutedBlock>, LedgerInfoWithSignatures) {
    let genesis_qc = certificate_for_genesis();
    let block = Block::new_proposal(vec![], 1, 1, genesis_qc, signer);
    let compute_result = StateComputeResult::new(
        executed_hash,
        vec![], // dummy subtree
        0,
        vec![],
        0,
        None,
        vec![],
        vec![],
        vec![],
    );

    let li = LedgerInfo::new(
        block.gen_block_info(
            compute_result.root_hash(),
            compute_result.version(),
            compute_result.epoch_state().clone(),
        ),
        consensus_hash,
    );

    let mut li_sig = LedgerInfoWithSignatures::new(
        li.clone(),
        BTreeMap::<AccountAddress, Ed25519Signature>::new(),
    );

    li_sig.add_signature(signer.author(), signer.sign(&li));

    let executed_block = ExecutedBlock::new(block, compute_result);

    (vec![executed_block], li_sig)
}

pub fn prepare_executed_blocks_with_executed_ledger_info(
    signer: &ValidatorSigner,
) -> (Vec<ExecutedBlock>, LedgerInfoWithSignatures) {
    prepare_executed_blocks_with_ledger_info(
        signer,
        HashValue::random(),
        HashValue::from_u64(0xbeef),
    )
}

pub fn prepare_executed_blocks_with_ordered_ledger_info(
    signer: &ValidatorSigner,
) -> (Vec<ExecutedBlock>, LedgerInfoWithSignatures) {
    prepare_executed_blocks_with_ledger_info(
        signer,
        *ACCUMULATOR_PLACEHOLDER_HASH,
        *ACCUMULATOR_PLACEHOLDER_HASH,
    )
}

pub fn new_executed_ledger_info_with_empty_signature(
    block_info: BlockInfo,
    li: &LedgerInfo,
) -> LedgerInfoWithSignatures {
    LedgerInfoWithSignatures::new(
        LedgerInfo::new(block_info, li.consensus_data_hash()),
        BTreeMap::<AccountAddress, Ed25519Signature>::new(), //empty
    )
}
