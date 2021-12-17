// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chunk_request::{GetChunkRequest, TargetType},
    chunk_response::{GetChunkResponse, ResponseLedgerInfo},
    coordinator::StateSyncCoordinator,
    executor_proxy::ExecutorProxy,
    network::StateSyncMessage,
    shared_components::test_utils,
};
use diem_config::network_id::NetworkId;
use diem_infallible::Mutex;
use diem_types::{
    ledger_info::LedgerInfoWithSignatures, transaction::TransactionListWithProof, PeerId,
};
use diem_vm::DiemVM;
use executor::chunk_executor::ChunkExecutor;
use futures::executor::block_on;
use mempool_notifications::MempoolNotifier;
use once_cell::sync::Lazy;
use proptest::{
    arbitrary::{any, Arbitrary},
    option,
    prelude::*,
    strategy::Strategy,
};

static STATE_SYNC_COORDINATOR: Lazy<
    Mutex<StateSyncCoordinator<ExecutorProxy<ChunkExecutor<DiemVM>>, MempoolNotifier>>,
> = Lazy::new(|| Mutex::new(test_utils::create_validator_coordinator()));

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn test_state_sync_msg_fuzzer(input in arb_state_sync_msg()) {
        test_state_sync_msg_fuzzer_impl(input);
    }
}

pub fn test_state_sync_msg_fuzzer_impl(message: StateSyncMessage) {
    block_on(async move {
        let _ = STATE_SYNC_COORDINATOR
            .lock()
            .process_chunk_message(
                NetworkId::Validator,
                PeerId::new([0u8; PeerId::LENGTH]),
                message,
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
