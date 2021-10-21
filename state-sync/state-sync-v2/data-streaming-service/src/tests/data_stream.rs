// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    data_notification::{
        DataClientRequest, DataPayload, EpochEndingLedgerInfosRequest, PendingClientResponse,
    },
    data_stream::{DataStream, DataStreamListener},
    streaming_client::{GetAllEpochEndingLedgerInfosRequest, StreamRequest},
    tests::utils::{
        create_data_client_response, create_ledger_info, MockDiemDataClient, MAX_ADVERTISED_EPOCH,
        MAX_NOTIFICATION_TIMEOUT_SECS, MIN_ADVERTISED_EPOCH,
    },
};
use claim::{assert_ge, assert_none};
use diem_data_client::{
    AdvertisedData, DataClientPayload, DataClientResponse, GlobalDataSummary, OptimalChunkSizes,
};
use diem_infallible::Mutex;
use diem_types::ledger_info::LedgerInfoWithSignatures;
use futures::{FutureExt, StreamExt};
use std::{
    cmp::min,
    sync::{atomic::AtomicU64, Arc},
    time::Duration,
};
use storage_service_types::CompleteDataRange;
use tokio::time::timeout;

#[tokio::test]
async fn test_epoch_ending_requests() {
    // Create an epoch ending data stream
    let (mut data_stream, _) = create_epoch_ending_stream(MIN_ADVERTISED_EPOCH);

    // Initialize the data stream
    let epoch_chunk_size = 15;
    data_stream
        .initialize_data_requests(create_global_data_summary(epoch_chunk_size))
        .unwrap();

    // Verify valid client requests have been created
    let (sent_requests, _) = data_stream.get_sent_requests_and_notifications();
    for i in 0..sent_requests.as_ref().unwrap().len() {
        let sent_request = sent_requests.as_ref().unwrap().get(i).unwrap();
        let i = i as u64;

        // Verify the start and end epochs of the request
        assert_eq!(
            sent_request.lock().client_request,
            DataClientRequest::EpochEndingLedgerInfos(EpochEndingLedgerInfosRequest {
                start_epoch: MIN_ADVERTISED_EPOCH + (i * epoch_chunk_size),
                end_epoch: min(
                    MAX_ADVERTISED_EPOCH - 1,
                    MIN_ADVERTISED_EPOCH + ((i + 1) * epoch_chunk_size) - 1
                ),
            })
        );
    }
}

#[tokio::test]
async fn test_epoch_ending_notifications() {
    // Create an epoch ending data stream
    let (mut data_stream, mut stream_listener) = create_epoch_ending_stream(MIN_ADVERTISED_EPOCH);

    // Initialize the data stream
    let epoch_chunk_size = 100;
    data_stream
        .initialize_data_requests(create_global_data_summary(epoch_chunk_size))
        .unwrap();

    // Clear the pending queue and insert a response
    let pending_response = PendingClientResponse {
        client_request: DataClientRequest::EpochEndingLedgerInfos(EpochEndingLedgerInfosRequest {
            start_epoch: MIN_ADVERTISED_EPOCH,
            end_epoch: MIN_ADVERTISED_EPOCH + 1,
        }),
        client_response: Some(Ok(create_data_client_response(
            DataClientPayload::EpochEndingLedgerInfos(vec![create_ledger_info(
                0,
                MIN_ADVERTISED_EPOCH,
                true,
            )]),
        ))),
    };
    insert_response_into_pending_queue(&mut data_stream, pending_response);

    // Process the response and verify a notification is sent to the client
    data_stream
        .process_data_responses(create_global_data_summary(epoch_chunk_size))
        .unwrap();
    verify_epoch_ending_notification(
        &mut stream_listener,
        create_ledger_info(0, MIN_ADVERTISED_EPOCH, true),
    )
    .await;
}

#[tokio::test]
async fn test_stream_initialization() {
    // Create an epoch ending data stream
    let (mut data_stream, _) = create_epoch_ending_stream(MIN_ADVERTISED_EPOCH);

    // Verify the data stream is not initialized
    assert!(!data_stream.data_requests_initialized());

    // Initialize the data stream
    data_stream
        .initialize_data_requests(create_global_data_summary(100))
        .unwrap();

    // Verify the data stream is now initialized
    assert!(data_stream.data_requests_initialized());

    // Verify that client requests have been made
    let (sent_requests, _) = data_stream.get_sent_requests_and_notifications();
    assert_ne!(sent_requests.as_ref().unwrap().len(), 0);
}

#[tokio::test]
async fn test_stream_data_error() {
    // Create an epoch ending data stream
    let (mut data_stream, mut stream_listener) = create_epoch_ending_stream(MIN_ADVERTISED_EPOCH);

    // Initialize the data stream
    let global_data_summary = create_global_data_summary(100);
    data_stream
        .initialize_data_requests(global_data_summary.clone())
        .unwrap();

    // Clear the pending queue and insert an error response
    let client_request = DataClientRequest::EpochEndingLedgerInfos(EpochEndingLedgerInfosRequest {
        start_epoch: MIN_ADVERTISED_EPOCH,
        end_epoch: MIN_ADVERTISED_EPOCH + 1,
    });
    let pending_response = PendingClientResponse {
        client_request: client_request.clone(),
        client_response: Some(Err(diem_data_client::Error::DataIsUnavailable(
            "Missing data!".into(),
        ))),
    };
    insert_response_into_pending_queue(&mut data_stream, pending_response);

    // Process the responses and verify the data client request was resent to the network
    data_stream
        .process_data_responses(global_data_summary)
        .unwrap();
    assert_none!(stream_listener.select_next_some().now_or_never());
    verify_client_request_resubmitted(&mut data_stream, client_request);
}

#[tokio::test]
async fn test_stream_invalid_response() {
    // Create an epoch ending data stream
    let (mut data_stream, mut stream_listener) = create_epoch_ending_stream(MIN_ADVERTISED_EPOCH);

    // Initialize the data stream
    let global_data_summary = create_global_data_summary(100);
    data_stream
        .initialize_data_requests(global_data_summary.clone())
        .unwrap();

    // Clear the pending queue and insert a response with an invalid type
    let client_request = DataClientRequest::EpochEndingLedgerInfos(EpochEndingLedgerInfosRequest {
        start_epoch: MIN_ADVERTISED_EPOCH,
        end_epoch: MIN_ADVERTISED_EPOCH + 1,
    });
    let pending_response = PendingClientResponse {
        client_request: client_request.clone(),
        client_response: Some(Ok(DataClientResponse {
            response_id: 0,
            response_payload: DataClientPayload::NumberOfAccountStates(10),
        })),
    };
    insert_response_into_pending_queue(&mut data_stream, pending_response);

    // Process the responses and verify the data client request was resent to the network
    data_stream
        .process_data_responses(global_data_summary)
        .unwrap();
    assert_none!(stream_listener.select_next_some().now_or_never());
    verify_client_request_resubmitted(&mut data_stream, client_request);
}

#[tokio::test]
async fn test_stream_out_of_order_responses() {
    // Create an epoch ending data stream
    let (mut data_stream, mut stream_listener) = create_epoch_ending_stream(MIN_ADVERTISED_EPOCH);

    // Initialize the data stream
    let global_data_summary = create_global_data_summary(1);
    data_stream
        .initialize_data_requests(global_data_summary.clone())
        .unwrap();

    // Verify at least three requests have been made
    let (sent_requests, _) = data_stream.get_sent_requests_and_notifications();
    assert_ge!(sent_requests.as_ref().unwrap().len(), 3);

    // Set a response for the second request and verify no notifications
    set_epoch_ending_response_in_queue(&mut data_stream, 1);
    data_stream
        .process_data_responses(global_data_summary.clone())
        .unwrap();
    assert_none!(stream_listener.select_next_some().now_or_never());

    // Set a response for the first request and verify two notifications
    set_epoch_ending_response_in_queue(&mut data_stream, 0);
    data_stream
        .process_data_responses(global_data_summary.clone())
        .unwrap();
    for _ in 0..2 {
        verify_epoch_ending_notification(
            &mut stream_listener,
            create_ledger_info(0, MIN_ADVERTISED_EPOCH, true),
        )
        .await;
    }
    assert_none!(stream_listener.select_next_some().now_or_never());

    // Set the response for the first and third request and verify one notification sent
    set_epoch_ending_response_in_queue(&mut data_stream, 0);
    set_epoch_ending_response_in_queue(&mut data_stream, 2);
    data_stream
        .process_data_responses(global_data_summary.clone())
        .unwrap();
    verify_epoch_ending_notification(
        &mut stream_listener,
        create_ledger_info(0, MIN_ADVERTISED_EPOCH, true),
    )
    .await;
    assert_none!(stream_listener.select_next_some().now_or_never());

    // Set the response for the first and third request and verify three notifications sent
    set_epoch_ending_response_in_queue(&mut data_stream, 0);
    set_epoch_ending_response_in_queue(&mut data_stream, 2);
    data_stream
        .process_data_responses(global_data_summary.clone())
        .unwrap();
    for _ in 0..3 {
        verify_epoch_ending_notification(
            &mut stream_listener,
            create_ledger_info(0, MIN_ADVERTISED_EPOCH, true),
        )
        .await;
    }
    assert_none!(stream_listener.select_next_some().now_or_never());
}

/// Creates an epoch ending stream starting at `start_epoch`
fn create_epoch_ending_stream(
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
    let diem_data_client = MockDiemDataClient::new();
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

fn create_global_data_summary(chunk_sizes: u64) -> GlobalDataSummary {
    let mut global_data_summary = GlobalDataSummary::empty();
    global_data_summary.optimal_chunk_sizes = create_optimal_chunk_sizes(chunk_sizes);
    global_data_summary
}

fn create_optimal_chunk_sizes(chunk_sizes: u64) -> OptimalChunkSizes {
    OptimalChunkSizes {
        account_states_chunk_size: chunk_sizes,
        epoch_chunk_size: chunk_sizes,
        transaction_chunk_size: chunk_sizes,
        transaction_output_chunk_size: chunk_sizes,
    }
}

/// Sets the client response at the index in the pending queue to contain an
/// epoch ending data response.
fn set_epoch_ending_response_in_queue(
    data_stream: &mut DataStream<MockDiemDataClient>,
    index: usize,
) {
    // Set the response at the specified index
    let (sent_requests, _) = data_stream.get_sent_requests_and_notifications();
    let pending_response = sent_requests.as_mut().unwrap().get_mut(index).unwrap();
    let client_response = Some(Ok(create_data_client_response(
        DataClientPayload::EpochEndingLedgerInfos(vec![create_ledger_info(
            0,
            MIN_ADVERTISED_EPOCH,
            true,
        )]),
    )));
    pending_response.lock().client_response = client_response;
}

/// Clears the pending queue of the given data stream and inserts a single
/// response into the head of the queue.
fn insert_response_into_pending_queue(
    data_stream: &mut DataStream<MockDiemDataClient>,
    pending_response: PendingClientResponse,
) {
    // Clear the queue
    let (sent_requests, _) = data_stream.get_sent_requests_and_notifications();
    sent_requests.as_mut().unwrap().clear();

    // Insert the pending response
    let pending_response = Arc::new(Mutex::new(Box::new(pending_response)));
    sent_requests.as_mut().unwrap().push_front(pending_response);
}

/// Verifies that a client request was resubmitted (i.e., pushed to the head of the
/// sent request queue)
fn verify_client_request_resubmitted(
    data_stream: &mut DataStream<MockDiemDataClient>,
    client_request: DataClientRequest,
) {
    let (sent_requests, _) = data_stream.get_sent_requests_and_notifications();
    let pending_response = sent_requests.as_mut().unwrap().pop_front().unwrap();
    assert_eq!(pending_response.lock().client_request, client_request);
    assert_none!(pending_response.lock().client_response.as_ref());
}

/// Verifies that a single epoch ending notification is received by the
/// data listener and that it contains the `expected_ledger_info`.
async fn verify_epoch_ending_notification(
    stream_listener: &mut DataStreamListener,
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
