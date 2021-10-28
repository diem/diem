// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    data_notification::DataPayload,
    error::Error,
    streaming_client::{
        new_streaming_service_client_listener_pair, DataStreamingClient, PayloadFeedback,
        StreamingServiceClient,
    },
    streaming_service::DataStreamingService,
    tests::utils::{
        initialize_logger, MockDiemDataClient, MAX_ADVERTISED_ACCOUNTS, MAX_ADVERTISED_EPOCH,
        MAX_ADVERTISED_TRANSACTION, MAX_ADVERTISED_TRANSACTION_OUTPUT,
        MAX_NOTIFICATION_TIMEOUT_SECS, MIN_ADVERTISED_ACCOUNTS, MIN_ADVERTISED_EPOCH,
        MIN_ADVERTISED_TRANSACTION, MIN_ADVERTISED_TRANSACTION_OUTPUT, TOTAL_NUM_ACCOUNTS,
    },
};
use claim::{assert_le, assert_matches, assert_ok, assert_some};
use futures::StreamExt;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test(flavor = "multi_thread")]
async fn test_notifications_accounts() {
    // Create a new streaming client and service
    let (streaming_client, streaming_service) = create_new_streaming_client_and_service();
    tokio::spawn(streaming_service.start_service());

    // Request an account stream and get a data stream listener
    let mut stream_listener = streaming_client
        .get_all_accounts(MAX_ADVERTISED_ACCOUNTS, None)
        .await
        .unwrap();

    // Read the data notifications from the stream and verify index ordering
    let mut next_expected_index = 0;
    loop {
        if let Ok(data_notification) = timeout(
            Duration::from_secs(MAX_NOTIFICATION_TIMEOUT_SECS),
            stream_listener.select_next_some(),
        )
        .await
        {
            match data_notification.data_payload {
                DataPayload::AccountStatesWithProof(accounts_with_proof) => {
                    // Verify the account start index matches the expected index
                    assert_eq!(accounts_with_proof.first_index, next_expected_index);

                    // Verify the last account index matches the account list length
                    let num_accounts = accounts_with_proof.account_blobs.len() as u64;
                    assert_eq!(
                        accounts_with_proof.last_index,
                        next_expected_index + num_accounts - 1,
                    );

                    // Verify the number of account blobs is as expected
                    assert_eq!(accounts_with_proof.account_blobs.len() as u64, num_accounts);

                    next_expected_index += num_accounts;
                }
                DataPayload::EndOfStream => {
                    assert_eq!(next_expected_index, TOTAL_NUM_ACCOUNTS);
                    return;
                }
                data_payload => {
                    panic!(
                        "Expected an account ledger info payload, but got: {:?}",
                        data_payload
                    );
                }
            }
        } else {
            panic!(
                "Timed out waiting for a data notification! Next expected index: {:?}",
                next_expected_index
            );
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_notifications_accounts_multiple_streams() {
    // Create a new streaming client and service
    let (streaming_client, streaming_service) = create_new_streaming_client_and_service();
    tokio::spawn(streaming_service.start_service());

    // Request a new account stream starting at the next expected index.
    let mut next_expected_index = 0;
    let mut stream_listener = streaming_client
        .get_all_accounts(MAX_ADVERTISED_ACCOUNTS, Some(next_expected_index))
        .await
        .unwrap();

    // Terminate and request new account streams at increasing versions
    loop {
        if let Ok(data_notification) = timeout(
            Duration::from_secs(MAX_NOTIFICATION_TIMEOUT_SECS),
            stream_listener.select_next_some(),
        )
        .await
        {
            match data_notification.data_payload {
                DataPayload::AccountStatesWithProof(accounts_with_proof) => {
                    // Verify the indices
                    assert_eq!(accounts_with_proof.first_index, next_expected_index);
                    next_expected_index += accounts_with_proof.account_blobs.len() as u64;

                    if next_expected_index < TOTAL_NUM_ACCOUNTS {
                        // Terminate the stream
                        streaming_client
                            .terminate_stream_with_feedback(
                                data_notification.notification_id,
                                PayloadFeedback::InvalidPayloadData,
                            )
                            .await
                            .unwrap();

                        // Fetch a new stream
                        stream_listener = streaming_client
                            .get_all_accounts(MAX_ADVERTISED_ACCOUNTS, Some(next_expected_index))
                            .await
                            .unwrap();
                    }
                }
                DataPayload::EndOfStream => {
                    assert_eq!(next_expected_index, TOTAL_NUM_ACCOUNTS);
                    return;
                }
                data_payload => {
                    panic!(
                        "Expected an account ledger info payload, but got: {:?}",
                        data_payload
                    );
                }
            }
        } else {
            panic!(
                "Timed out waiting for a data notification! Next expected index: {:?}",
                next_expected_index
            );
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_notifications_continuous_outputs() {
    // Create a new streaming client and service
    let (streaming_client, streaming_service) = create_new_streaming_client_and_service();
    tokio::spawn(streaming_service.start_service());

    // Request a continuous output stream and get a data stream listener
    let mut stream_listener = streaming_client
        .continuously_stream_transaction_outputs(
            MIN_ADVERTISED_TRANSACTION_OUTPUT,
            MIN_ADVERTISED_EPOCH,
        )
        .await
        .unwrap();

    // Read the data notifications from the stream and verify the payloads
    let mut next_expected_epoch = MIN_ADVERTISED_EPOCH;
    let mut next_expected_version = MIN_ADVERTISED_TRANSACTION_OUTPUT;
    loop {
        if let Ok(data_notification) = timeout(
            Duration::from_secs(MAX_NOTIFICATION_TIMEOUT_SECS),
            stream_listener.select_next_some(),
        )
        .await
        {
            match data_notification.data_payload {
                DataPayload::ContinuousTransactionOutputsWithProof(
                    ledger_info_with_sigs,
                    outputs_with_proofs,
                ) => {
                    let ledger_info = ledger_info_with_sigs.ledger_info();
                    // Verify the epoch of the ledger info
                    assert_eq!(ledger_info.epoch(), next_expected_epoch);

                    // Verify the output start version matches the expected version
                    let first_output_version = outputs_with_proofs.first_transaction_output_version;
                    assert_eq!(Some(next_expected_version), first_output_version);

                    let num_outputs = outputs_with_proofs.transactions_and_outputs.len() as u64;
                    next_expected_version += num_outputs;

                    // Update epochs if we've hit the epoch end
                    let last_output_version = first_output_version.unwrap() + num_outputs - 1;
                    if ledger_info.version() == last_output_version && ledger_info.ends_epoch() {
                        next_expected_epoch += 1;
                    }
                }
                data_payload => {
                    panic!(
                        "Expected a continuous output payload, but got: {:?}",
                        data_payload
                    );
                }
            }
        } else {
            if next_expected_epoch == MAX_ADVERTISED_EPOCH
                && next_expected_version == MAX_ADVERTISED_TRANSACTION_OUTPUT + 1
            {
                return; // We hit the end of the stream!
            }
            panic!(
                "Timed out waiting for a data notification! Next expected output: {:?}",
                next_expected_version
            );
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_notifications_continuous_transactions() {
    // Create a new streaming client and service
    let (streaming_client, streaming_service) = create_new_streaming_client_and_service();
    tokio::spawn(streaming_service.start_service());

    // Request a continuous transaction stream and get a data stream listener
    let mut stream_listener = streaming_client
        .continuously_stream_transactions(MIN_ADVERTISED_TRANSACTION, MIN_ADVERTISED_EPOCH, true)
        .await
        .unwrap();

    // Read the data notifications from the stream and verify the payloads
    let mut next_expected_epoch = MIN_ADVERTISED_EPOCH;
    let mut next_expected_version = MIN_ADVERTISED_TRANSACTION;
    loop {
        if let Ok(data_notification) = timeout(
            Duration::from_secs(MAX_NOTIFICATION_TIMEOUT_SECS),
            stream_listener.select_next_some(),
        )
        .await
        {
            match data_notification.data_payload {
                DataPayload::ContinuousTransactionsWithProof(
                    ledger_info_with_sigs,
                    transactions_with_proof,
                ) => {
                    let ledger_info = ledger_info_with_sigs.ledger_info();
                    // Verify the epoch of the ledger info
                    assert_eq!(ledger_info.epoch(), next_expected_epoch);

                    // Verify the transaction start version matches the expected version
                    let first_transaction_version =
                        transactions_with_proof.first_transaction_version;
                    assert_eq!(Some(next_expected_version), first_transaction_version);

                    // Verify the payload contains events
                    assert_some!(transactions_with_proof.events);

                    let num_transactions = transactions_with_proof.transactions.len() as u64;
                    next_expected_version += num_transactions;

                    // Update epochs if we've hit the epoch end
                    let last_transaction_version =
                        first_transaction_version.unwrap() + num_transactions - 1;
                    if ledger_info.version() == last_transaction_version && ledger_info.ends_epoch()
                    {
                        next_expected_epoch += 1;
                    }
                }
                data_payload => {
                    panic!(
                        "Expected a continuous transaction payload, but got: {:?}",
                        data_payload
                    );
                }
            }
        } else {
            if next_expected_epoch == MAX_ADVERTISED_EPOCH
                && next_expected_version == MAX_ADVERTISED_TRANSACTION + 1
            {
                return; // We hit the end of the stream!
            }
            panic!(
                "Timed out waiting for a data notification! Next expected transaction: {:?}",
                next_expected_version
            );
        }
    }
}

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
            match data_notification.data_payload {
                DataPayload::EpochEndingLedgerInfos(ledger_infos_with_sigs) => {
                    // Verify the epochs of the ledger infos are contiguous
                    for ledger_info_with_sigs in ledger_infos_with_sigs {
                        let epoch = ledger_info_with_sigs.ledger_info().commit_info().epoch();
                        assert_eq!(next_expected_epoch, epoch);
                        assert_le!(epoch, MAX_ADVERTISED_EPOCH - 1);
                        next_expected_epoch += 1;
                    }
                }
                DataPayload::EndOfStream => {
                    assert_eq!(next_expected_epoch, MAX_ADVERTISED_EPOCH);
                    return;
                }
                data_payload => {
                    panic!(
                        "Expected an epoch ending ledger info payload, but got: {:?}",
                        data_payload
                    );
                }
            }
        } else {
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
            match data_notification.data_payload {
                DataPayload::TransactionOutputsWithProof(outputs_with_proof) => {
                    // Verify the transaction output start version matches the expected version
                    let first_output_version = outputs_with_proof.first_transaction_output_version;
                    assert_eq!(Some(next_expected_output), first_output_version);

                    let num_outputs = outputs_with_proof.transactions_and_outputs.len();
                    next_expected_output += num_outputs as u64;
                }
                DataPayload::EndOfStream => {
                    assert_eq!(next_expected_output, MAX_ADVERTISED_TRANSACTION_OUTPUT + 1);
                    return;
                }
                data_payload => {
                    panic!(
                        "Expected a transaction output payload, but got: {:?}",
                        data_payload
                    );
                }
            }
        } else {
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
            match data_notification.data_payload {
                DataPayload::TransactionsWithProof(transactions_with_proof) => {
                    // Verify the transaction start version matches the expected version
                    let first_transaction_version =
                        transactions_with_proof.first_transaction_version;
                    assert_eq!(Some(next_expected_transaction), first_transaction_version);

                    // Verify the payload contains events
                    assert_some!(transactions_with_proof.events);

                    let num_transactions = transactions_with_proof.transactions.len();
                    next_expected_transaction += num_transactions as u64;
                }
                DataPayload::EndOfStream => {
                    assert_eq!(next_expected_transaction, MAX_ADVERTISED_TRANSACTION + 1);
                    return;
                }
                data_payload => {
                    panic!(
                        "Expected a transaction payload, but got: {:?}",
                        data_payload
                    );
                }
            }
        } else {
            panic!(
                "Timed out waiting for a data notification! Next expected transaction: {:?}",
                next_expected_transaction
            );
        }
    }
}

#[tokio::test]
async fn test_stream_accounts() {
    // Create a new streaming client and service
    let (streaming_client, streaming_service) = create_new_streaming_client_and_service();
    tokio::spawn(streaming_service.start_service());

    // Request an account stream and verify we get a data stream listener
    let result = streaming_client
        .get_all_accounts(MAX_ADVERTISED_ACCOUNTS - 1, None)
        .await;
    assert_ok!(result);

    // Request a stream where accounts are missing (we are lower than advertised)
    let result = streaming_client
        .get_all_accounts(MIN_ADVERTISED_ACCOUNTS - 1, None)
        .await;
    assert_matches!(result, Err(Error::DataIsUnavailable(_)));

    // Request a stream where accounts are missing (we are lower than advertised)
    let result = streaming_client
        .get_all_accounts(MAX_ADVERTISED_EPOCH + 1, None)
        .await;
    assert_matches!(result, Err(Error::DataIsUnavailable(_)));
}

#[tokio::test]
async fn test_stream_continuous_outputs() {
    // Create a new streaming client and service
    let (streaming_client, streaming_service) = create_new_streaming_client_and_service();
    tokio::spawn(streaming_service.start_service());

    // Request a continuous output stream and verify we get a data stream listener
    let result = streaming_client
        .continuously_stream_transaction_outputs(
            MIN_ADVERTISED_TRANSACTION_OUTPUT,
            MIN_ADVERTISED_EPOCH,
        )
        .await;
    assert_ok!(result);

    // Request a stream where data is missing (we are lower than advertised)
    let result = streaming_client
        .continuously_stream_transaction_outputs(
            MIN_ADVERTISED_TRANSACTION_OUTPUT - 1,
            MIN_ADVERTISED_EPOCH,
        )
        .await;
    assert_matches!(result, Err(Error::DataIsUnavailable(_)));

    // Request a stream where data is missing (we are higher than advertised)
    let result = streaming_client
        .continuously_stream_transaction_outputs(
            MAX_ADVERTISED_TRANSACTION_OUTPUT + 1,
            MIN_ADVERTISED_EPOCH,
        )
        .await;
    assert_matches!(result, Err(Error::DataIsUnavailable(_)));
}

#[tokio::test]
async fn test_stream_continuous_transactions() {
    // Create a new streaming client and service
    let (streaming_client, streaming_service) = create_new_streaming_client_and_service();
    tokio::spawn(streaming_service.start_service());

    // Request a continuous transaction stream and verify we get a data stream listener
    let result = streaming_client
        .continuously_stream_transactions(MIN_ADVERTISED_TRANSACTION, MIN_ADVERTISED_EPOCH, true)
        .await;
    assert_ok!(result);

    // Request a stream where data is missing (we are lower than advertised)
    let result = streaming_client
        .continuously_stream_transactions(
            MIN_ADVERTISED_TRANSACTION - 1,
            MIN_ADVERTISED_EPOCH,
            true,
        )
        .await;
    assert_matches!(result, Err(Error::DataIsUnavailable(_)));

    // Request a stream where data is missing (we are higher than advertised)
    let result = streaming_client
        .continuously_stream_transactions(
            MAX_ADVERTISED_TRANSACTION + 1,
            MIN_ADVERTISED_EPOCH,
            true,
        )
        .await;
    assert_matches!(result, Err(Error::DataIsUnavailable(_)));
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

#[tokio::test(flavor = "multi_thread")]
#[should_panic(expected = "SelectNextSome polled after terminated")]
async fn test_terminate_stream() {
    // Create a new streaming client and service
    let (streaming_client, streaming_service) = create_new_streaming_client_and_service();
    tokio::spawn(streaming_service.start_service());

    // Request an account stream
    let mut stream_listener = streaming_client
        .get_all_accounts(MAX_ADVERTISED_ACCOUNTS - 1, None)
        .await
        .unwrap();

    // Fetch the first account notification and then terminate the stream
    let mut next_expected_index = 0;
    if let Ok(data_notification) = timeout(
        Duration::from_secs(MAX_NOTIFICATION_TIMEOUT_SECS),
        stream_listener.select_next_some(),
    )
    .await
    {
        match data_notification.data_payload {
            DataPayload::AccountStatesWithProof(accounts_with_proof) => {
                next_expected_index += accounts_with_proof.account_blobs.len() as u64;
            }
            data_payload => {
                panic!(
                    "Expected an account ledger info payload, but got: {:?}",
                    data_payload
                );
            }
        }

        // Terminate the stream
        let result = streaming_client
            .terminate_stream_with_feedback(
                data_notification.notification_id,
                PayloadFeedback::InvalidPayloadData,
            )
            .await;
        assert_ok!(result);
    } else {
        panic!("Timed out waiting for a data notification!");
    }

    // Verify the streaming service has removed the stream
    loop {
        if let Ok(data_notification) = timeout(
            Duration::from_secs(MAX_NOTIFICATION_TIMEOUT_SECS),
            stream_listener.select_next_some(),
        )
        .await
        {
            match data_notification.data_payload {
                DataPayload::AccountStatesWithProof(accounts_with_proof) => {
                    next_expected_index += accounts_with_proof.account_blobs.len() as u64;
                }
                DataPayload::EndOfStream => {
                    panic!("The stream should have terminated!");
                }
                data_payload => {
                    panic!(
                        "Expected an account ledger info payload, but got: {:?}",
                        data_payload
                    );
                }
            }
        } else if next_expected_index >= TOTAL_NUM_ACCOUNTS {
            panic!(
                "The stream should have terminated! Next expected index: {:?}",
                next_expected_index
            );
        }
    }
}

fn create_new_streaming_client_and_service() -> (
    StreamingServiceClient,
    DataStreamingService<MockDiemDataClient>,
) {
    initialize_logger();

    // Create a new streaming client and listener
    let (streaming_client, streaming_service_listener) =
        new_streaming_service_client_listener_pair();

    // Create the streaming service and connect it to the listener
    let diem_data_client = MockDiemDataClient::new();
    let streaming_service = DataStreamingService::new(diem_data_client, streaming_service_listener);

    (streaming_client, streaming_service)
}
