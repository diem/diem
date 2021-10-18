// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    data_notification::DataPayload,
    error::Error,
    streaming_client::{
        new_streaming_service_client_listener_pair, DataStreamingClient, PayloadRefetchReason,
        StreamingServiceClient,
    },
    streaming_service::DataStreamingService,
    tests::utils::{
        MockDiemDataClient, MAX_ADVERTISED_EPOCH, MAX_ADVERTISED_TRANSACTION,
        MAX_ADVERTISED_TRANSACTION_OUTPUT, MAX_NOTIFICATION_TIMEOUT_SECS, MIN_ADVERTISED_EPOCH,
        MIN_ADVERTISED_TRANSACTION, MIN_ADVERTISED_TRANSACTION_OUTPUT,
    },
};
use claim::{assert_le, assert_matches, assert_ok, assert_some};
use futures::StreamExt;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test(flavor = "multi_thread")]
async fn test_notifications_epoch_ending() {
    // Create a new streaming client and service
    let (streaming_client, streaming_service) = create_new_streaming_client_and_service();
    tokio::spawn(streaming_service.start_service());

    // Request an epoch ending stream and get a data stream listener
    let mut stream_listener = streaming_client
        .get_all_epoch_ending_ledger_infos(MIN_ADVERTISED_EPOCH)
        .await
        .unwrap();

    // Read the data notifications from the stream and verify epoch ordering
    let mut next_expected_epoch = MIN_ADVERTISED_EPOCH;
    loop {
        if let Ok(data_notification) = timeout(
            Duration::from_secs(MAX_NOTIFICATION_TIMEOUT_SECS),
            stream_listener.select_next_some(),
        )
        .await
        {
            if let DataPayload::EpochEndingLedgerInfos(ledger_infos_with_sigs) =
                data_notification.data_payload
            {
                // Verify the epochs of the ledger infos are contiguous
                for ledger_info_with_sigs in ledger_infos_with_sigs {
                    let epoch = ledger_info_with_sigs.ledger_info().commit_info().epoch();
                    assert_eq!(next_expected_epoch, epoch);
                    assert_le!(epoch, MAX_ADVERTISED_EPOCH);
                    next_expected_epoch += 1;
                }
            } else {
                panic!(
                    "Expected an epoch ending ledger info payload, but got: {:?}",
                    data_notification
                );
            }
        } else {
            if next_expected_epoch == MAX_ADVERTISED_EPOCH {
                return; // We hit the end of the stream!
            }
            panic!(
                "Timed out waiting for a data notification! Next expected epoch: {:?}",
                next_expected_epoch
            );
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_notifications_transaction_outputs() {
    // Create a new streaming client and service
    let (streaming_client, streaming_service) = create_new_streaming_client_and_service();
    tokio::spawn(streaming_service.start_service());

    // Request a transaction output stream and get a data stream listener
    let mut stream_listener = streaming_client
        .get_all_transaction_outputs(
            MIN_ADVERTISED_TRANSACTION_OUTPUT,
            MAX_ADVERTISED_TRANSACTION_OUTPUT,
            MAX_ADVERTISED_TRANSACTION_OUTPUT,
        )
        .await
        .unwrap();

    // Read the data notifications from the stream and verify the payloads
    let mut next_expected_output = MIN_ADVERTISED_TRANSACTION_OUTPUT;
    loop {
        if let Ok(data_notification) = timeout(
            Duration::from_secs(MAX_NOTIFICATION_TIMEOUT_SECS),
            stream_listener.select_next_some(),
        )
        .await
        {
            if let DataPayload::TransactionOutputsWithProof(outputs_with_proof) =
                data_notification.data_payload
            {
                // Verify the transaction output start version matches the expected version
                let first_output_version = outputs_with_proof.first_transaction_output_version;
                assert_eq!(Some(next_expected_output), first_output_version);

                let num_outputs = outputs_with_proof.transaction_outputs.len();
                next_expected_output += num_outputs as u64;
            } else {
                panic!(
                    "Expected a transaction output payload, but got: {:?}",
                    data_notification
                );
            }
        } else {
            if next_expected_output == MAX_ADVERTISED_TRANSACTION_OUTPUT + 1 {
                return; // We hit the end of the stream!
            }
            panic!(
                "Timed out waiting for a data notification! Next expected output: {:?}",
                next_expected_output
            );
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_notifications_transactions() {
    // Create a new streaming client and service
    let (streaming_client, streaming_service) = create_new_streaming_client_and_service();
    tokio::spawn(streaming_service.start_service());

    // Request a transaction stream (with events) and get a data stream listener
    let mut stream_listener = streaming_client
        .get_all_transactions(
            MIN_ADVERTISED_TRANSACTION,
            MAX_ADVERTISED_TRANSACTION,
            MAX_ADVERTISED_TRANSACTION,
            true,
        )
        .await
        .unwrap();

    // Read the data notifications from the stream and verify the payloads
    let mut next_expected_transaction = MIN_ADVERTISED_TRANSACTION;
    loop {
        if let Ok(data_notification) = timeout(
            Duration::from_secs(MAX_NOTIFICATION_TIMEOUT_SECS),
            stream_listener.select_next_some(),
        )
        .await
        {
            if let DataPayload::TransactionsWithProof(transactions_with_proof) =
                data_notification.data_payload
            {
                // Verify the transaction start version matches the expected version
                let first_transaction_version = transactions_with_proof.first_transaction_version;
                assert_eq!(Some(next_expected_transaction), first_transaction_version);

                // Verify the payload contains events
                assert_some!(transactions_with_proof.events);

                let num_transactions = transactions_with_proof.transactions.len();
                next_expected_transaction += num_transactions as u64;
            } else {
                panic!(
                    "Expected a transaction payload, but got: {:?}",
                    data_notification
                );
            }
        } else {
            if next_expected_transaction == MAX_ADVERTISED_TRANSACTION + 1 {
                return; // We hit the end of the stream!
            }
            panic!(
                "Timed out waiting for a data notification! Next expected transaction: {:?}",
                next_expected_transaction
            );
        }
    }
}

#[tokio::test]
async fn test_stream_epoch_ending() {
    // Create a new streaming client and service
    let (streaming_client, streaming_service) = create_new_streaming_client_and_service();
    tokio::spawn(streaming_service.start_service());

    // Request an epoch ending stream and verify we get a data stream listener
    let result = streaming_client
        .get_all_epoch_ending_ledger_infos(MIN_ADVERTISED_EPOCH)
        .await;
    assert_ok!(result);

    // Request a stream where epoch data is missing (we are lower than advertised)
    let result = streaming_client
        .get_all_epoch_ending_ledger_infos(MIN_ADVERTISED_EPOCH - 1)
        .await;
    assert_matches!(result, Err(Error::DataIsUnavailable(_)));

    // Request a stream where epoch data is missing (we are higher than advertised)
    let result = streaming_client
        .get_all_epoch_ending_ledger_infos(MAX_ADVERTISED_EPOCH + 1)
        .await;
    assert_matches!(result, Err(Error::DataIsUnavailable(_)));
}

#[tokio::test]
async fn test_stream_transaction_outputs() {
    // Create a new streaming client and service
    let (streaming_client, streaming_service) = create_new_streaming_client_and_service();
    tokio::spawn(streaming_service.start_service());

    // Request a transaction output stream and verify we get a data stream listener
    let result = streaming_client
        .get_all_transaction_outputs(
            MIN_ADVERTISED_TRANSACTION_OUTPUT,
            MAX_ADVERTISED_TRANSACTION_OUTPUT,
            MAX_ADVERTISED_TRANSACTION_OUTPUT,
        )
        .await;
    assert_ok!(result);

    // Request a stream where outputs are missing (we are higher than advertised)
    let result = streaming_client
        .get_all_transaction_outputs(
            MIN_ADVERTISED_TRANSACTION_OUTPUT,
            MAX_ADVERTISED_TRANSACTION_OUTPUT + 1,
            MAX_ADVERTISED_TRANSACTION_OUTPUT + 1,
        )
        .await;
    assert_matches!(result, Err(Error::DataIsUnavailable(_)));

    // Request a stream where outputs are missing (we are lower than advertised)
    let result = streaming_client
        .get_all_transaction_outputs(
            MIN_ADVERTISED_TRANSACTION_OUTPUT - 1,
            MAX_ADVERTISED_TRANSACTION_OUTPUT,
            MAX_ADVERTISED_TRANSACTION_OUTPUT,
        )
        .await;
    assert_matches!(result, Err(Error::DataIsUnavailable(_)));
}

#[tokio::test]
async fn test_stream_transactions() {
    // Create a new streaming client and service
    let (streaming_client, streaming_service) = create_new_streaming_client_and_service();
    tokio::spawn(streaming_service.start_service());

    // Request a transaction stream and verify we get a data stream listener
    let result = streaming_client
        .get_all_transactions(
            MIN_ADVERTISED_TRANSACTION,
            MAX_ADVERTISED_TRANSACTION,
            MAX_ADVERTISED_TRANSACTION,
            true,
        )
        .await;
    assert_ok!(result);

    // Request a stream where transactions are missing (we are higher than advertised)
    let result = streaming_client
        .get_all_transactions(
            MIN_ADVERTISED_TRANSACTION,
            MAX_ADVERTISED_TRANSACTION + 1,
            MAX_ADVERTISED_TRANSACTION,
            true,
        )
        .await;
    assert_matches!(result, Err(Error::DataIsUnavailable(_)));

    // Request a stream where transactions is missing (we are lower than advertised)
    let result = streaming_client
        .get_all_transactions(
            MIN_ADVERTISED_TRANSACTION - 1,
            MAX_ADVERTISED_TRANSACTION,
            MAX_ADVERTISED_TRANSACTION,
            true,
        )
        .await;
    assert_matches!(result, Err(Error::DataIsUnavailable(_)));
}

#[tokio::test]
async fn test_stream_unsupported() {
    // Create a new streaming client and service
    let (streaming_client, streaming_service) = create_new_streaming_client_and_service();
    tokio::spawn(streaming_service.start_service());

    // Request an account stream and verify it's unsupported
    let result = streaming_client.get_all_accounts(0).await;
    assert_matches!(result, Err(Error::UnsupportedRequestEncountered(_)));

    // Request a continuous transaction stream and verify it's unsupported
    let result = streaming_client
        .continuously_stream_transactions(0, 0, true)
        .await;
    assert_matches!(result, Err(Error::UnsupportedRequestEncountered(_)));

    // Request a continuous transaction output stream and verify it's unsupported
    let result = streaming_client
        .continuously_stream_transaction_outputs(0, 0)
        .await;
    assert_matches!(result, Err(Error::UnsupportedRequestEncountered(_)));

    // Request a refetch notification payload stream and verify it's unsupported
    let result = streaming_client
        .refetch_notification_payload(0, PayloadRefetchReason::InvalidPayloadData)
        .await;
    assert_matches!(result, Err(Error::UnsupportedRequestEncountered(_)));
}

fn create_new_streaming_client_and_service() -> (
    StreamingServiceClient,
    DataStreamingService<MockDiemDataClient>,
) {
    // Create a new streaming client and listener
    let (streaming_client, streaming_service_listener) =
        new_streaming_service_client_listener_pair();

    // Create the streaming service and connect it to the listener
    let diem_data_client = MockDiemDataClient {};
    let streaming_service = DataStreamingService::new(diem_data_client, streaming_service_listener);

    (streaming_client, streaming_service)
}
