// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    data_notification::{
        DataClientRequest, DataPayload, EpochEndingLedgerInfosRequest, PendingClientResponse,
    },
    data_stream,
    data_stream::{DataStream, DataStreamListener},
    streaming_client::{GetAllEpochEndingLedgerInfosRequest, StreamRequest},
    tests::utils::{
        create_ledger_info, MockDiemDataClient, MAX_ADVERTISED_EPOCH,
        MAX_NOTIFICATION_TIMEOUT_SECS, MIN_ADVERTISED_EPOCH,
    },
};
use diem_data_client::{AdvertisedData, DataClientPayload, DataClientResponse, OptimalChunkSizes};
use diem_infallible::Mutex;
use diem_types::ledger_info::LedgerInfoWithSignatures;
use futures::StreamExt;
use std::{
    collections::VecDeque,
    sync::{atomic::AtomicU64, Arc},
    time::Duration,
};
use storage_service_types::CompleteDataRange;
use tokio::time::timeout;

#[tokio::test]
async fn test_epoch_ending_stream_initialization() {
    // Create an epoch ending data stream
    let (mut data_stream, _) = create_epoch_ending_data_stream(0);

    // Verify the data stream is not initialized
    assert!(!data_stream.data_requests_initialized());

    // Initialize the data stream
    let optimal_chunk_sizes = create_optimal_chunk_sizes(100);
    data_stream
        .initialize_data_requests(optimal_chunk_sizes)
        .unwrap();

    // Verify the data stream is now initialized and client requests have been created
    assert!(data_stream.data_requests_initialized());
    let (sent_requests, _) = data_stream.get_sent_requests_and_notifications();
    assert_ne!(sent_requests.as_ref().unwrap().len(), 0);
}

#[tokio::test]
async fn test_epoch_ending_stream_notifications() {
    // Create an epoch ending data stream
    let (mut data_stream, stream_listener) = create_epoch_ending_data_stream(100);

    // Initialize the data stream
    let optimal_chunk_sizes = create_optimal_chunk_sizes(80);
    data_stream
        .initialize_data_requests(optimal_chunk_sizes.clone())
        .unwrap();

    // Verify client requests have been created in the pending queue
    let (sent_requests, _) = data_stream.get_sent_requests_and_notifications();
    for i in 0..sent_requests.as_ref().unwrap().len() {
        let sent_request = sent_requests.as_ref().unwrap().get(i).unwrap();
        let pending_response = sent_request.lock();
        let i = i as u64;

        assert_eq!(
            pending_response.client_request,
            DataClientRequest::EpochEndingLedgerInfos(EpochEndingLedgerInfosRequest {
                start_epoch: 100 + (i * 80),
                end_epoch: 100 + ((i + 1) * 80) - 1,
            })
        );
    }

    // Clear the queue and manually insert a response
    let pending_client_response = Arc::new(Mutex::new(Box::new(PendingClientResponse {
        client_request: DataClientRequest::EpochEndingLedgerInfos(EpochEndingLedgerInfosRequest {
            start_epoch: 100,
            end_epoch: 179,
        }),
        client_response: Some(Ok(DataClientResponse {
            response_id: 0,
            response_payload: DataClientPayload::EpochEndingLedgerInfos(vec![create_ledger_info(
                100,
            )]),
        })),
    })));
    overwrite_sent_requests(sent_requests, pending_client_response);

    // Process the responses and verify a notification is sent to the client
    data_stream
        .process_data_responses(optimal_chunk_sizes)
        .unwrap();
    verify_single_epoch_ending_notification(stream_listener, create_ledger_info(100)).await;
}

fn overwrite_sent_requests(
    sent_requests: &mut Option<VecDeque<data_stream::PendingClientResponse>>,
    pending_client_response: data_stream::PendingClientResponse,
) {
    sent_requests.as_mut().unwrap().clear();
    sent_requests
        .as_mut()
        .unwrap()
        .push_front(pending_client_response);
}

fn create_epoch_ending_data_stream(
    start_epoch: u64,
) -> (DataStream<MockDiemDataClient>, DataStreamListener) {
    // Create an epoch ending stream request
    let stream_request =
        StreamRequest::GetAllEpochEndingLedgerInfos(GetAllEpochEndingLedgerInfosRequest {
            start_epoch,
        });

    // Create an advertised data containing only epoch ending ledger infos
    let advertised_data = AdvertisedData {
        account_states: vec![],
        epoch_ending_ledger_infos: vec![CompleteDataRange::new(
            MIN_ADVERTISED_EPOCH,
            MAX_ADVERTISED_EPOCH,
        )],
        synced_ledger_infos: vec![],
        transactions: vec![],
        transaction_outputs: vec![],
    };

    // Create a diem data client mock and notification generator
    let diem_data_client = MockDiemDataClient {};
    let notification_generator = Arc::new(AtomicU64::new(0));

    // Return the data stream and listener pair
    DataStream::new(
        &stream_request,
        diem_data_client,
        notification_generator,
        &advertised_data,
    )
    .unwrap()
}

fn create_optimal_chunk_sizes(chunk_size: u64) -> OptimalChunkSizes {
    OptimalChunkSizes {
        account_states_chunk_size: chunk_size,
        epoch_chunk_size: chunk_size,
        transaction_chunk_size: chunk_size,
        transaction_output_chunk_size: chunk_size,
    }
}

async fn verify_single_epoch_ending_notification(
    mut stream_listener: DataStreamListener,
    expected_ledger_info: LedgerInfoWithSignatures,
) {
    if let Ok(data_notification) = timeout(
        Duration::from_secs(MAX_NOTIFICATION_TIMEOUT_SECS),
        stream_listener.select_next_some(),
    )
    .await
    {
        if let DataPayload::EpochEndingLedgerInfos(ledger_infos) = data_notification.data_payload {
            assert_eq!(ledger_infos[0], expected_ledger_info);
        } else {
            panic!(
                "Expected an epoch ending ledger info payload, but got: {:?}",
                data_notification
            );
        }
    } else {
        panic!("Timed out waiting for a data notification!");
    }
}
