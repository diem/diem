// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    data_notification::{
        DataClientRequest, DataPayload, EpochEndingLedgerInfosRequest, PendingClientResponse,
    },
    data_stream::{DataStream, DataStreamListener},
    streaming_client::{
        GetAllAccountsRequest, GetAllEpochEndingLedgerInfosRequest, GetAllTransactionsRequest,
        NotificationFeedback, StreamRequest,
    },
    tests::utils::{
        create_data_client_response, create_ledger_info, create_random_u64,
        create_transaction_list_with_proof, get_data_notification, initialize_logger,
        MockDiemDataClient, NoopResponseCallback, MAX_ADVERTISED_ACCOUNTS,
        MAX_ADVERTISED_EPOCH_END, MAX_ADVERTISED_TRANSACTION_OUTPUT, MAX_NOTIFICATION_TIMEOUT_SECS,
        MIN_ADVERTISED_ACCOUNTS, MIN_ADVERTISED_EPOCH_END, MIN_ADVERTISED_TRANSACTION_OUTPUT,
    },
};
use claim::{assert_err, assert_ge, assert_none, assert_ok};
use diem_config::config::DataStreamingServiceConfig;
use diem_data_client::{
    AdvertisedData, GlobalDataSummary, OptimalChunkSizes, Response, ResponseContext,
    ResponsePayload,
};
use diem_id_generator::U64IdGenerator;
use diem_infallible::Mutex;
use diem_types::{ledger_info::LedgerInfoWithSignatures, transaction::Version};
use futures::{FutureExt, StreamExt};
use std::{sync::Arc, time::Duration};
use storage_service_types::CompleteDataRange;
use tokio::time::timeout;

#[tokio::test]
async fn test_stream_blocked() {
    // Create an account stream
    let streaming_service_config = DataStreamingServiceConfig::default();
    let (mut data_stream, mut stream_listener) =
        create_account_stream(streaming_service_config, MIN_ADVERTISED_ACCOUNTS);

    // Initialize the data stream
    let global_data_summary = create_global_data_summary(100);
    data_stream
        .initialize_data_requests(global_data_summary.clone())
        .unwrap();

    let mut number_of_refetches = 0;
    loop {
        // Clear the pending queue and insert a response with an invalid type
        let client_request =
            DataClientRequest::EpochEndingLedgerInfos(EpochEndingLedgerInfosRequest {
                start_epoch: 0,
                end_epoch: 0,
            });
        let context = ResponseContext {
            id: 0,
            response_callback: Box::new(NoopResponseCallback),
        };
        let pending_response = PendingClientResponse {
            client_request: client_request.clone(),
            client_response: Some(Ok(Response {
                context,
                payload: ResponsePayload::NumberOfAccountStates(10),
            })),
        };
        insert_response_into_pending_queue(&mut data_stream, pending_response);

        // Process the data responses and force a data re-fetch
        data_stream
            .process_data_responses(global_data_summary.clone())
            .unwrap();

        // If we're sent a data notification, verify it's an end of stream notification!
        if let Ok(data_notification) = timeout(
            Duration::from_secs(MAX_NOTIFICATION_TIMEOUT_SECS),
            stream_listener.select_next_some(),
        )
        .await
        {
            match data_notification.data_payload {
                DataPayload::EndOfStream => {
                    assert_eq!(
                        number_of_refetches,
                        streaming_service_config.max_request_retry
                    );

                    // Provide incorrect feedback for the notification
                    assert_err!(data_stream.handle_notification_feedback(
                        &data_notification.notification_id,
                        &NotificationFeedback::PayloadTypeIsIncorrect
                    ));

                    // Provide valid feedback for the notification
                    assert_ok!(data_stream.handle_notification_feedback(
                        &data_notification.notification_id,
                        &NotificationFeedback::EndOfStream,
                    ));
                    return;
                }
                data_payload => panic!("Unexpected payload type: {:?}", data_payload),
            }
        }
        number_of_refetches += 1;
    }
}

#[tokio::test]
async fn test_stream_garbage_collection() {
    // Create a transaction stream
    let streaming_service_config = DataStreamingServiceConfig::default();
    let (mut data_stream, mut stream_listener) = create_transaction_stream(
        streaming_service_config,
        MIN_ADVERTISED_TRANSACTION_OUTPUT,
        MAX_ADVERTISED_TRANSACTION_OUTPUT,
    );

    // Initialize the data stream
    let global_data_summary = create_global_data_summary(1);
    data_stream
        .initialize_data_requests(global_data_summary.clone())
        .unwrap();

    loop {
        // Insert a transaction response into the queue
        set_transaction_response_at_queue_head(&mut data_stream);

        // Process the data response
        data_stream
            .process_data_responses(global_data_summary.clone())
            .unwrap();

        // Process the data response
        let data_notification = get_data_notification(&mut stream_listener).await.unwrap();
        if matches!(data_notification.data_payload, DataPayload::EndOfStream) {
            return;
        }

        // Verify the notification to response map is garbage collected
        let (_, sent_notifications) = data_stream.get_sent_requests_and_notifications();
        assert!(
            (sent_notifications.len() as u64)
                <= streaming_service_config.max_notification_id_mappings
        );
    }
}

#[tokio::test]
async fn test_stream_initialization() {
    // Create an epoch ending data stream
    let streaming_service_config = DataStreamingServiceConfig::default();
    let (mut data_stream, _) =
        create_epoch_ending_stream(streaming_service_config, MIN_ADVERTISED_EPOCH_END);

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
    let streaming_service_config = DataStreamingServiceConfig::default();
    let (mut data_stream, mut stream_listener) =
        create_epoch_ending_stream(streaming_service_config, MIN_ADVERTISED_EPOCH_END);

    // Initialize the data stream
    let global_data_summary = create_global_data_summary(100);
    data_stream
        .initialize_data_requests(global_data_summary.clone())
        .unwrap();

    // Clear the pending queue and insert an error response
    let client_request = DataClientRequest::EpochEndingLedgerInfos(EpochEndingLedgerInfosRequest {
        start_epoch: MIN_ADVERTISED_EPOCH_END,
        end_epoch: MIN_ADVERTISED_EPOCH_END + 1,
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
    let streaming_service_config = DataStreamingServiceConfig::default();
    let (mut data_stream, mut stream_listener) =
        create_epoch_ending_stream(streaming_service_config, MIN_ADVERTISED_EPOCH_END);

    // Initialize the data stream
    let global_data_summary = create_global_data_summary(100);
    data_stream
        .initialize_data_requests(global_data_summary.clone())
        .unwrap();

    // Clear the pending queue and insert a response with an invalid type
    let client_request = DataClientRequest::EpochEndingLedgerInfos(EpochEndingLedgerInfosRequest {
        start_epoch: MIN_ADVERTISED_EPOCH_END,
        end_epoch: MIN_ADVERTISED_EPOCH_END + 1,
    });
    let context = ResponseContext {
        id: 0,
        response_callback: Box::new(NoopResponseCallback),
    };
    let client_response = Response::new(context, ResponsePayload::NumberOfAccountStates(10));
    let pending_response = PendingClientResponse {
        client_request: client_request.clone(),
        client_response: Some(Ok(client_response)),
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
    let streaming_service_config = DataStreamingServiceConfig::default();
    let (mut data_stream, mut stream_listener) =
        create_epoch_ending_stream(streaming_service_config, MIN_ADVERTISED_EPOCH_END);

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
            create_ledger_info(0, MIN_ADVERTISED_EPOCH_END, true),
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
        create_ledger_info(0, MIN_ADVERTISED_EPOCH_END, true),
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
            create_ledger_info(0, MIN_ADVERTISED_EPOCH_END, true),
        )
        .await;
    }
    assert_none!(stream_listener.select_next_some().now_or_never());
}

/// Creates an account stream for the given `version`.
fn create_account_stream(
    streaming_service_config: DataStreamingServiceConfig,
    version: Version,
) -> (DataStream<MockDiemDataClient>, DataStreamListener) {
    // Create an account stream request
    let stream_request = StreamRequest::GetAllAccounts(GetAllAccountsRequest {
        version,
        start_index: 0,
    });
    create_data_stream(streaming_service_config, stream_request)
}

/// Creates an epoch ending stream starting at `start_epoch`
fn create_epoch_ending_stream(
    streaming_service_config: DataStreamingServiceConfig,
    start_epoch: u64,
) -> (DataStream<MockDiemDataClient>, DataStreamListener) {
    // Create an epoch ending stream request
    let stream_request =
        StreamRequest::GetAllEpochEndingLedgerInfos(GetAllEpochEndingLedgerInfosRequest {
            start_epoch,
        });
    create_data_stream(streaming_service_config, stream_request)
}

/// Creates a transaction output stream for the given `version`.
fn create_transaction_stream(
    streaming_service_config: DataStreamingServiceConfig,
    start_version: Version,
    end_version: Version,
) -> (DataStream<MockDiemDataClient>, DataStreamListener) {
    // Create a transaction output stream
    let stream_request = StreamRequest::GetAllTransactions(GetAllTransactionsRequest {
        start_version,
        end_version,
        proof_version: end_version,
        include_events: false,
    });
    create_data_stream(streaming_service_config, stream_request)
}

fn create_data_stream(
    streaming_service_config: DataStreamingServiceConfig,
    stream_request: StreamRequest,
) -> (DataStream<MockDiemDataClient>, DataStreamListener) {
    initialize_logger();

    // Create an advertised data
    let advertised_data = AdvertisedData {
        account_states: vec![CompleteDataRange::new(
            MIN_ADVERTISED_ACCOUNTS,
            MAX_ADVERTISED_ACCOUNTS,
        )
        .unwrap()],
        epoch_ending_ledger_infos: vec![CompleteDataRange::new(
            MIN_ADVERTISED_EPOCH_END,
            MAX_ADVERTISED_EPOCH_END,
        )
        .unwrap()],
        synced_ledger_infos: vec![],
        transactions: vec![],
        transaction_outputs: vec![CompleteDataRange::new(
            MIN_ADVERTISED_TRANSACTION_OUTPUT,
            MAX_ADVERTISED_TRANSACTION_OUTPUT,
        )
        .unwrap()],
    };

    // Create a diem data client mock and notification generator
    let diem_data_client = MockDiemDataClient::new();
    let notification_generator = Arc::new(U64IdGenerator::new());

    // Return the data stream and listener pair
    DataStream::new(
        streaming_service_config,
        create_random_u64(10000),
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
        ResponsePayload::EpochEndingLedgerInfos(vec![create_ledger_info(
            0,
            MIN_ADVERTISED_EPOCH_END,
            true,
        )]),
    )));
    pending_response.lock().client_response = client_response;
}

/// Sets the client response at the head of the pending queue to contain an
/// transaction response.
fn set_transaction_response_at_queue_head(data_stream: &mut DataStream<MockDiemDataClient>) {
    // Set the response at the specified index
    let (sent_requests, _) = data_stream.get_sent_requests_and_notifications();
    if !sent_requests.as_mut().unwrap().is_empty() {
        let pending_response = sent_requests.as_mut().unwrap().get_mut(0).unwrap();
        let client_response = Some(Ok(create_data_client_response(
            ResponsePayload::TransactionsWithProof(create_transaction_list_with_proof(0, 0, false)),
        )));
        pending_response.lock().client_response = client_response;
    }
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
    let data_notification = get_data_notification(stream_listener).await.unwrap();
    if let DataPayload::EpochEndingLedgerInfos(ledger_infos) = data_notification.data_payload {
        assert_eq!(ledger_infos[0], expected_ledger_info);
    } else {
        panic!(
            "Expected an epoch ending ledger info payload, but got: {:?}",
            data_notification
        );
    }
}
