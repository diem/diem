// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    data_notification::DataNotification,
    data_stream::DataStreamListener,
    error::Error,
    streaming_client::{
        new_streaming_service_client_listener_pair, ContinuouslyStreamTransactionOutputsRequest,
        ContinuouslyStreamTransactionsRequest, DataStreamingClient, GetAllAccountsRequest,
        GetAllEpochEndingLedgerInfosRequest, GetAllTransactionOutputsRequest,
        GetAllTransactionsRequest, PayloadFeedback, StreamRequest, StreamingServiceListener,
        TerminateStreamRequest,
    },
    tests::utils::initialize_logger,
};
use channel::{diem_channel, message_queues::QueueStyle};
use claim::assert_ok;
use futures::{executor::block_on, FutureExt, StreamExt};
use std::thread::JoinHandle;

#[test]
fn test_client_service_error() {
    // Create a new streaming service client and listener
    let (streaming_service_client, streaming_service_listener) =
        new_streaming_service_client_listener_pair();

    // Spawn a new server thread to handle any requests and respond with an error
    let response_error = Error::UnexpectedErrorEncountered("Oops! Something went wrong!".into());
    let _handler =
        spawn_service_and_respond_with_error(streaming_service_listener, response_error.clone());

    // Send an epoch ending stream request and verify the expected error is returned
    let response = block_on(streaming_service_client.get_all_epoch_ending_ledger_infos(0));
    assert_eq!(response.unwrap_err(), response_error);
}

#[test]
fn test_get_all_accounts() {
    // Create a new streaming service client and listener
    let (streaming_service_client, streaming_service_listener) =
        new_streaming_service_client_listener_pair();

    // Note the request we expect to receive on the streaming service side
    let request_version = 100;
    let expected_request = StreamRequest::GetAllAccounts(GetAllAccountsRequest {
        version: request_version,
    });

    // Spawn a new server thread to handle any account stream requests
    let _handler = spawn_service_and_expect_request(streaming_service_listener, expected_request);

    // Send an account stream request and verify we get a data stream listener
    let response = block_on(streaming_service_client.get_all_accounts(request_version));
    assert_ok!(response);
}

#[test]
fn test_get_all_epoch_ending_ledger_infos() {
    // Create a new streaming service client and listener
    let (streaming_service_client, streaming_service_listener) =
        new_streaming_service_client_listener_pair();

    // Note the request we expect to receive on the streaming service side
    let request_start_epoch = 10;
    let expected_request =
        StreamRequest::GetAllEpochEndingLedgerInfos(GetAllEpochEndingLedgerInfosRequest {
            start_epoch: request_start_epoch,
        });

    // Spawn a new server thread to handle any epoch ending stream requests
    let _handler = spawn_service_and_expect_request(streaming_service_listener, expected_request);

    // Send an epoch ending stream request and verify we get a data stream listener
    let response =
        block_on(streaming_service_client.get_all_epoch_ending_ledger_infos(request_start_epoch));
    assert_ok!(response);
}

#[test]
fn test_get_all_transactions() {
    // Create a new streaming service client and listener
    let (streaming_service_client, streaming_service_listener) =
        new_streaming_service_client_listener_pair();

    // Note the request we expect to receive on the streaming service side
    let request_start_version = 101;
    let request_end_version = 200;
    let request_max_proof_version = 300;
    let request_include_events = true;
    let expected_request = StreamRequest::GetAllTransactions(GetAllTransactionsRequest {
        start_version: request_start_version,
        end_version: request_end_version,
        max_proof_version: request_max_proof_version,
        include_events: request_include_events,
    });

    // Spawn a new server thread to handle any transaction stream requests
    let _handler = spawn_service_and_expect_request(streaming_service_listener, expected_request);

    // Send a transaction stream request and verify we get a data stream listener
    let response = block_on(streaming_service_client.get_all_transactions(
        request_start_version,
        request_end_version,
        request_max_proof_version,
        request_include_events,
    ));
    assert_ok!(response);
}

#[test]
fn test_get_all_transaction_outputs() {
    // Create a new streaming service client and listener
    let (streaming_service_client, streaming_service_listener) =
        new_streaming_service_client_listener_pair();

    // Note the request we expect to receive on the streaming service side
    let request_start_version = 101;
    let request_end_version = 200;
    let request_max_proof_version = 300;
    let expected_request =
        StreamRequest::GetAllTransactionOutputs(GetAllTransactionOutputsRequest {
            start_version: request_start_version,
            end_version: request_end_version,
            max_proof_version: request_max_proof_version,
        });

    // Spawn a new server thread to handle any transaction output stream requests
    let _handler = spawn_service_and_expect_request(streaming_service_listener, expected_request);

    // Send a transaction output stream request and verify we get a data stream listener
    let response = block_on(streaming_service_client.get_all_transaction_outputs(
        request_start_version,
        request_end_version,
        request_max_proof_version,
    ));
    assert_ok!(response);
}

#[test]
fn test_continuously_stream_transactions() {
    // Create a new streaming service client and listener
    let (streaming_service_client, streaming_service_listener) =
        new_streaming_service_client_listener_pair();

    // Note the request we expect to receive on the streaming service side
    let request_start_version = 101;
    let request_start_epoch = 2;
    let request_include_events = false;
    let expected_request =
        StreamRequest::ContinuouslyStreamTransactions(ContinuouslyStreamTransactionsRequest {
            start_version: request_start_version,
            start_epoch: request_start_epoch,
            include_events: request_include_events,
        });

    // Spawn a new server thread to handle any continuous transaction stream requests
    let _handler = spawn_service_and_expect_request(streaming_service_listener, expected_request);

    // Send a continuous transaction stream request and verify we get a data stream listener
    let response = block_on(streaming_service_client.continuously_stream_transactions(
        request_start_version,
        request_start_epoch,
        request_include_events,
    ));
    assert_ok!(response);
}

#[test]
fn test_continuously_stream_transaction_outputs() {
    // Create a new streaming service client and listener
    let (streaming_service_client, streaming_service_listener) =
        new_streaming_service_client_listener_pair();

    // Note the request we expect to receive on the streaming service side
    let request_start_version = 101;
    let request_start_epoch = 2;
    let expected_request = StreamRequest::ContinuouslyStreamTransactionOutputs(
        ContinuouslyStreamTransactionOutputsRequest {
            start_version: request_start_version,
            start_epoch: request_start_epoch,
        },
    );

    // Spawn a new server thread to handle any continuous transaction output stream requests
    let _handler = spawn_service_and_expect_request(streaming_service_listener, expected_request);

    // Send a continuous transaction output stream request and verify we get a data stream listener
    let response = block_on(
        streaming_service_client
            .continuously_stream_transaction_outputs(request_start_version, request_start_epoch),
    );
    assert_ok!(response);
}

#[test]
fn test_terminate_stream() {
    // Create a new streaming service client and listener
    let (streaming_service_client, streaming_service_listener) =
        new_streaming_service_client_listener_pair();

    // Note the request we expect to receive on the streaming service side
    let request_notification_id = 19478;
    let payload_feedback = PayloadFeedback::InvalidPayloadData;
    let expected_request = StreamRequest::TerminateStream(TerminateStreamRequest {
        notification_id: request_notification_id,
        payload_feedback: payload_feedback.clone(),
    });

    // Spawn a new server thread to handle any feedback requests
    let _handler = spawn_service_and_expect_request(streaming_service_listener, expected_request);

    // Provide payload feedback and verify no error is returned
    let result = block_on(
        streaming_service_client
            .terminate_stream_with_feedback(request_notification_id, payload_feedback),
    );
    assert_ok!(result);
}

/// Spawns a new thread that listens to the given streaming service listener and
/// responds successfully to any requests that match the specified `expected_request`.
/// Otherwise, an error is returned.
fn spawn_service_and_expect_request(
    mut streaming_service_listener: StreamingServiceListener,
    expected_request: StreamRequest,
) -> JoinHandle<()> {
    initialize_logger();

    std::thread::spawn(move || loop {
        if let Some(stream_request_message) =
            streaming_service_listener.select_next_some().now_or_never()
        {
            // Create a new data stream sender and listener pair
            let (_, listener) = new_data_stream_sender_listener();

            // Verify the client request is as expected and respond appropriately
            let stream_request = stream_request_message.stream_request;
            let response = if stream_request == expected_request {
                Ok(listener)
            } else {
                Err(Error::UnexpectedErrorEncountered(format!(
                    "Unexpected stream request! Got: {:?} but expected: {:?}",
                    stream_request, expected_request
                )))
            };

            // Send the response to the client
            let _send_result = stream_request_message.response_sender.send(response);
        }
    })
}

/// Spawns a new thread that listens to the given streaming service listener and
/// responds with the specified error.
fn spawn_service_and_respond_with_error(
    mut streaming_service_listener: StreamingServiceListener,
    response_error: Error,
) -> JoinHandle<()> {
    initialize_logger();

    std::thread::spawn(move || loop {
        if let Some(stream_request_message) =
            streaming_service_listener.select_next_some().now_or_never()
        {
            let _result = stream_request_message
                .response_sender
                .send(Err(response_error.clone()));
        }
    })
}

/// Creates and returns a new data stream sender and listener pair.
fn new_data_stream_sender_listener() -> (
    channel::diem_channel::Sender<(), DataNotification>,
    DataStreamListener,
) {
    let (notification_sender, notification_receiver) =
        diem_channel::new(QueueStyle::KLAST, 1, None);
    let data_stream_listener = DataStreamListener::new(notification_receiver);

    (notification_sender, data_stream_listener)
}
