// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chunk_request::{GetChunkRequest, TargetType},
    chunk_response::{GetChunkResponse, ResponseLedgerInfo},
    coordinator::SyncCoordinator,
    network::{StateSynchronizerMsg, StateSynchronizerSender},
    tests::{
        helpers::{MockExecutorProxy, SynchronizerEnvHelper},
        mock_storage::MockStorage,
    },
};
use channel::{diem_channel, message_queues::QueueStyle};
use diem_config::{
    config::{NodeConfig, PeerNetworkId, RoleType},
    network_id::{NetworkId, NodeNetworkId},
};
use diem_infallible::RwLock;
use diem_types::{
    ledger_info::LedgerInfoWithSignatures, transaction::TransactionListWithProof,
    waypoint::Waypoint, PeerId,
};
use futures::channel::mpsc;
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
use std::{collections::HashMap, num::NonZeroUsize, sync::Arc};

static PEER_ID: Lazy<PeerId> = Lazy::new(|| PeerId::new([0u8; PeerId::LENGTH]));

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn test_state_sync_msg_fuzzer(input in state_sync_msg_strategy()) {
        test_state_sync_msg_fuzzer_impl(input);
    }
}

pub fn test_state_sync_msg_fuzzer_impl(msg: StateSynchronizerMsg) {
    // start up coordinator
    let (_coordinator_sender, coordinator_receiver) = mpsc::unbounded();
    let (mempool_sender, _mempool_receiver) = mpsc::channel(1_024);
    let config = NodeConfig::default_for_validator();

    let (signers, validator_info, _keys, _addrs) = SynchronizerEnvHelper::initial_setup(1);
    let genesis_li = SynchronizerEnvHelper::genesis_li(&validator_info);
    let storage_inner = MockStorage::new(genesis_li, signers[0].clone());
    let initial_state = storage_inner.get_local_storage_state();
    let storage_proxy = Arc::new(RwLock::new(storage_inner));

    // mock network senders
    let (network_reqs_tx, _network_reqs_rx) =
        diem_channel::new(QueueStyle::FIFO, NonZeroUsize::new(8).unwrap(), None);
    let (connection_reqs_tx, _) =
        diem_channel::new(QueueStyle::FIFO, NonZeroUsize::new(8).unwrap(), None);
    let network_sender = StateSynchronizerSender::new(
        PeerManagerRequestSender::new(network_reqs_tx),
        ConnectionRequestSender::new(connection_reqs_tx),
    );
    let node_network_id = NodeNetworkId::new(NetworkId::Validator, 0);
    let network_senders = vec![(node_network_id.clone(), network_sender)]
        .into_iter()
        .collect::<HashMap<_, _>>();
    let mut coordinator = SyncCoordinator::new(
        coordinator_receiver,
        mempool_sender,
        network_senders,
        RoleType::Validator,
        Waypoint::default(),
        config.state_sync,
        config.upstream,
        MockExecutorProxy::new(SynchronizerEnvHelper::default_handler(), storage_proxy),
        initial_state,
    );
    let mut rt = tokio::runtime::Builder::new()
        .basic_scheduler()
        .build()
        .unwrap();
    rt.block_on(async move {
        coordinator
            .process_one_message(PeerNetworkId(node_network_id, *PEER_ID), msg)
            .await;
    });
}

pub fn state_sync_msg_strategy() -> impl Strategy<Value = StateSynchronizerMsg> {
    prop_oneof![
        (any::<GetChunkRequest>()).prop_map(|get_chunk_request| {
            StateSynchronizerMsg::GetChunkRequest(Box::new(get_chunk_request))
        }),
        (any::<GetChunkResponse>()).prop_map(|get_chunk_response| {
            StateSynchronizerMsg::GetChunkResponse(Box::new(get_chunk_response))
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
            highest_available_strategy(),
            (any::<u64>()).prop_map(TargetType::Waypoint)
        ]
        .boxed()
    }

    type Strategy = BoxedStrategy<Self>;
}

fn highest_available_strategy() -> impl Strategy<Value = TargetType> {
    (option::of(any::<LedgerInfoWithSignatures>()), any::<u64>()).prop_map(
        |(target_li, timeout_ms)| TargetType::HighestAvailable {
            target_li,
            timeout_ms,
        },
    )
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
            progressive_li_strategy(),
            li_for_waypoint_strategy()
        ]
        .boxed()
    }

    type Strategy = BoxedStrategy<Self>;
}

fn progressive_li_strategy() -> impl Strategy<Value = ResponseLedgerInfo> {
    (
        any::<LedgerInfoWithSignatures>(),
        option::of(any::<LedgerInfoWithSignatures>()),
    )
        .prop_map(
            |(target_li, highest_li)| ResponseLedgerInfo::ProgressiveLedgerInfo {
                target_li,
                highest_li,
            },
        )
}

fn li_for_waypoint_strategy() -> impl Strategy<Value = ResponseLedgerInfo> {
    (
        any::<LedgerInfoWithSignatures>(),
        option::of(any::<LedgerInfoWithSignatures>()),
    )
        .prop_map(
            |(waypoint_li, end_of_epoch_li)| ResponseLedgerInfo::LedgerInfoForWaypoint {
                waypoint_li,
                end_of_epoch_li,
            },
        )
}
