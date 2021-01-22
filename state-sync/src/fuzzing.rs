// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chunk_request::{GetChunkRequest, TargetType},
    chunk_response::{GetChunkResponse, ResponseLedgerInfo},
    coordinator::StateSyncCoordinator,
    executor_proxy::{ExecutorProxy, ExecutorProxyTrait},
    network::{StateSyncMessage, StateSyncSender},
};
use channel::{diem_channel, message_queues::QueueStyle};
use diem_config::{
    config::{PeerNetworkId, RoleType, StateSyncConfig, UpstreamConfig},
    network_id::{NetworkId, NodeNetworkId},
};
use diem_infallible::Mutex;
use diem_types::{
    ledger_info::LedgerInfoWithSignatures,
    transaction::{Transaction, TransactionListWithProof, WriteSetPayload},
    waypoint::Waypoint,
    PeerId,
};
use diem_vm::DiemVM;
use diemdb::DiemDB;
use executor::Executor;
use executor_test_helpers::bootstrap_genesis;
use futures::{channel::mpsc, executor::block_on};
use network::{
    peer_manager::{ConnectionRequestSender, PeerManagerRequestSender},
    protocols::network::NewNetworkSender,
};
use once_cell::sync::Lazy;
use proptest::{
    arbitrary::{any, Arbitrary},
    option,
    prelude::*,
    strategy::Strategy,
};
use std::collections::HashMap;
use storage_interface::DbReaderWriter;

static STATE_SYNC_COORDINATOR: Lazy<Mutex<StateSyncCoordinator<ExecutorProxy>>> = Lazy::new(|| {
    // Generate a genesis change set
    let (genesis, _) = vm_genesis::test_genesis_change_set_and_validators(Some(1));

    // Create test diem database
    let db_path = diem_temppath::TempPath::new();
    db_path.create_as_dir().unwrap();
    let (db, db_rw) = DbReaderWriter::wrap(DiemDB::new_for_test(db_path.path()));

    // Bootstrap the genesis transaction
    let genesis_txn = Transaction::GenesisTransaction(WriteSetPayload::Direct(genesis));
    bootstrap_genesis::<DiemVM>(&db_rw, &genesis_txn).unwrap();

    // Create executor proxy
    let chunk_executor = Box::new(Executor::<DiemVM>::new(db_rw));
    let executor_proxy = ExecutorProxy::new(db, chunk_executor, vec![]);

    // Get initial state
    let initial_state = executor_proxy.get_local_storage_state().unwrap();

    // Setup network senders
    let (network_reqs_tx, _network_reqs_rx) = diem_channel::new(QueueStyle::FIFO, 8, None);
    let (connection_reqs_tx, _) = diem_channel::new(QueueStyle::FIFO, 8, None);
    let network_sender = StateSyncSender::new(
        PeerManagerRequestSender::new(network_reqs_tx),
        ConnectionRequestSender::new(connection_reqs_tx),
    );
    let node_network_id = NodeNetworkId::new(NetworkId::Validator, 0);
    let network_senders = vec![(node_network_id, network_sender)]
        .into_iter()
        .collect::<HashMap<_, _>>();

    // Create channel senders and receivers
    let (_coordinator_sender, coordinator_receiver) = mpsc::unbounded();
    let (mempool_sender, _mempool_receiver) = mpsc::channel(1);

    // Start up coordinator
    let coordinator = StateSyncCoordinator::new(
        coordinator_receiver,
        mempool_sender,
        network_senders,
        RoleType::Validator,
        Waypoint::default(),
        StateSyncConfig::default(),
        UpstreamConfig::default(),
        executor_proxy,
        initial_state,
    )
    .unwrap();

    Mutex::new(coordinator)
});

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn test_state_sync_msg_fuzzer(input in arb_state_sync_msg()) {
        test_state_sync_msg_fuzzer_impl(input);
    }
}

pub fn test_state_sync_msg_fuzzer_impl(msg: StateSyncMessage) {
    block_on(async move {
        STATE_SYNC_COORDINATOR
            .lock()
            .process_one_message(
                PeerNetworkId(
                    NodeNetworkId::new(NetworkId::Validator, 0),
                    PeerId::new([0u8; PeerId::LENGTH]),
                ),
                msg,
            )
            .await;
    });
}

pub fn arb_state_sync_msg() -> impl Strategy<Value = StateSyncMessage> {
    prop_oneof![
        (any::<GetChunkRequest>()).prop_map(|chunk_request| {
            StateSyncMessage::GetChunkRequest(Box::new(chunk_request))
        }),
        (any::<GetChunkResponse>()).prop_map(|chunk_response| {
            StateSyncMessage::GetChunkResponse(Box::new(chunk_response))
        })
    ]
}

impl Arbitrary for GetChunkRequest {
    type Parameters = ();
    fn arbitrary_with(_args: ()) -> Self::Strategy {
        (
            any::<u64>(),
            any::<u64>(),
            any::<u64>(),
            any::<TargetType>(),
        )
            .prop_map(|(known_version, current_epoch, limit, target)| {
                GetChunkRequest::new(known_version, current_epoch, limit, target)
            })
            .boxed()
    }
    type Strategy = BoxedStrategy<Self>;
}

impl Arbitrary for TargetType {
    type Parameters = ();
    fn arbitrary_with(_args: ()) -> Self::Strategy {
        prop_oneof![
            (any::<LedgerInfoWithSignatures>()).prop_map(TargetType::TargetLedgerInfo),
            (option::of(any::<LedgerInfoWithSignatures>()), any::<u64>()).prop_map(
                |(target_li, timeout_ms)| TargetType::HighestAvailable {
                    target_li,
                    timeout_ms
                }
            ),
            (any::<u64>()).prop_map(TargetType::Waypoint)
        ]
        .boxed()
    }
    type Strategy = BoxedStrategy<Self>;
}

impl Arbitrary for GetChunkResponse {
    type Parameters = ();
    fn arbitrary_with(_args: ()) -> Self::Strategy {
        (
            any::<ResponseLedgerInfo>(),
            any::<TransactionListWithProof>(),
        )
            .prop_map(|(response_li, txn_list_with_proof)| {
                GetChunkResponse::new(response_li, txn_list_with_proof)
            })
            .boxed()
    }
    type Strategy = BoxedStrategy<Self>;
}

impl Arbitrary for ResponseLedgerInfo {
    type Parameters = ();
    fn arbitrary_with(_args: ()) -> Self::Strategy {
        prop_oneof![
            (any::<LedgerInfoWithSignatures>()).prop_map(ResponseLedgerInfo::VerifiableLedgerInfo),
            (
                any::<LedgerInfoWithSignatures>(),
                option::of(any::<LedgerInfoWithSignatures>())
            )
                .prop_map(|(target_li, highest_li)| {
                    ResponseLedgerInfo::ProgressiveLedgerInfo {
                        target_li,
                        highest_li,
                    }
                }),
            (
                any::<LedgerInfoWithSignatures>(),
                option::of(any::<LedgerInfoWithSignatures>()),
            )
                .prop_map(|(waypoint_li, end_of_epoch_li)| {
                    ResponseLedgerInfo::LedgerInfoForWaypoint {
                        waypoint_li,
                        end_of_epoch_li,
                    }
                },)
        ]
        .boxed()
    }
    type Strategy = BoxedStrategy<Self>;
}
