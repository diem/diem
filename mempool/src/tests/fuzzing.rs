// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    core_mempool::{CoreMempool, TimelineState},
    shared_mempool::{peer_manager::PeerManager, tasks, types::SharedMempool},
};
use diem_config::config::NodeConfig;
use diem_infallible::{Mutex, RwLock};
use diem_types::transaction::SignedTransaction;
use proptest::{
    arbitrary::any,
    prelude::*,
    strategy::{Just, Strategy},
};
use std::{collections::HashMap, sync::Arc};
use storage_interface::mock::MockDbReader;
use vm_validator::mocks::mock_vm_validator::MockVMValidator;

/// Proptest strategy for fuzzing input to mempool incoming transactions endpoint
pub fn mempool_incoming_transactions_strategy(
) -> impl Strategy<Value = (Vec<SignedTransaction>, TimelineState)> {
    (
        proptest::collection::vec(any::<SignedTransaction>(), 0..100),
        prop_oneof![
            Just(TimelineState::NotReady),
            Just(TimelineState::NonQualified)
        ],
    )
}

/// Test that takes in fuzzer-generated inputs to mempool's `process_incoming_transactions_impl` endpoint
pub fn test_mempool_process_incoming_transactions_impl(
    txns: Vec<SignedTransaction>,
    timeline_state: TimelineState,
) {
    // set up mock Shared Mempool
    let config = NodeConfig::default();
    let mock_db = MockDbReader;
    let vm_validator = Arc::new(RwLock::new(MockVMValidator));
    let smp = SharedMempool {
        mempool: Arc::new(Mutex::new(CoreMempool::new(&config))),
        config: config.mempool.clone(),
        network_senders: HashMap::new(),
        db: Arc::new(mock_db),
        validator: vm_validator,
        peer_manager: Arc::new(PeerManager::new(config.mempool, config.upstream)),
        subscribers: vec![],
    };

    let _ = tasks::process_incoming_transactions(&smp, txns, timeline_state);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn test_mempool_process_incoming_transactions((txns, timeline_state) in mempool_incoming_transactions_strategy()) {
        test_mempool_process_incoming_transactions_impl(txns, timeline_state);
    }
}
