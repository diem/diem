// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    data_notification::{DataClientRequest, EpochEndingLedgerInfosRequest, SentDataNotification},
    error::Error,
    stream_progress_tracker::StreamProgressTracker,
    streaming_client::{GetAllEpochEndingLedgerInfosRequest, StreamRequest},
};
use claim::assert_matches;
use diem_data_client::{DataClientPayload, DataClientResponse, GlobalDataSummary};
use std::cmp;
use storage_service_types::CompleteDataRange;

#[test]
fn test_create_epoch_ending_requests() {
    // Create a new data stream progress tracker
    let stream_progress_tracker = create_epoch_ending_progress_tracker(0, 900);
    let StreamProgressTracker::EpochEndingStreamTracker(mut stream_tracker) =
        stream_progress_tracker;

    // Create a batch of large client requests and verify the result
    let client_requests = stream_tracker
        .create_epoch_ending_client_requests(5, 10000)
        .unwrap();
    let expected_requests = vec![DataClientRequest::EpochEndingLedgerInfos(
        EpochEndingLedgerInfosRequest {
            start_epoch: 0,
            end_epoch: 899,
        },
    )];
    assert_eq!(client_requests, expected_requests);

    // Create a batch of regular client requests and verify the result
    let client_requests = stream_tracker
        .create_epoch_ending_client_requests(3, 50)
        .unwrap();
    for (i, client_request) in client_requests.iter().enumerate() {
        let i = i as u64;
        let expected_request =
            DataClientRequest::EpochEndingLedgerInfos(EpochEndingLedgerInfosRequest {
                start_epoch: i * 50,
                end_epoch: ((i + 1) * 50) - 1,
            });
        assert_eq!(*client_request, expected_request);
    }

    // Create a batch of small client requests and verify the result
    let client_requests = stream_tracker
        .create_epoch_ending_client_requests(100, 14)
        .unwrap();
    for (i, client_request) in client_requests.iter().enumerate() {
        let i = i as u64;
        let expected_request =
            DataClientRequest::EpochEndingLedgerInfos(EpochEndingLedgerInfosRequest {
                start_epoch: i * 14,
                end_epoch: cmp::min(((i + 1) * 14) - 1, 899),
            });
        assert_eq!(*client_request, expected_request);
    }
}

#[test]
fn test_create_epoch_ending_requests_dynamic() {
    // Create a new data stream progress tracker
    let stream_progress_tracker = create_epoch_ending_progress_tracker(0, 1000);
    let StreamProgressTracker::EpochEndingStreamTracker(mut stream_tracker) =
        stream_progress_tracker;

    // Update the tracker with a new next request epoch
    stream_tracker.next_request_epoch = 150;

    // Create a batch of client requests and verify the result
    let client_requests = stream_tracker
        .create_epoch_ending_client_requests(5, 700)
        .unwrap();
    let expected_requests = vec![
        DataClientRequest::EpochEndingLedgerInfos(EpochEndingLedgerInfosRequest {
            start_epoch: 150,
            end_epoch: 849,
        }),
        DataClientRequest::EpochEndingLedgerInfos(EpochEndingLedgerInfosRequest {
            start_epoch: 850,
            end_epoch: 999,
        }),
    ];
    assert_eq!(client_requests, expected_requests);

    // Update the tracker with a new next request epoch
    stream_tracker.next_request_epoch = 700;

    // Create a batch of client requests and verify the result
    let client_requests = stream_tracker
        .create_epoch_ending_client_requests(10, 50)
        .unwrap();
    for (i, client_request) in client_requests.iter().enumerate() {
        let i = i as u64;
        let expected_request =
            DataClientRequest::EpochEndingLedgerInfos(EpochEndingLedgerInfosRequest {
                start_epoch: 700 + (i * 50),
                end_epoch: cmp::min(700 + ((i + 1) * 50) - 1, 999),
            });
        assert_eq!(*client_request, expected_request);
    }

    // Update the tracker with a new next request epoch that matches the stream end
    stream_tracker.next_request_epoch = 999;

    // Create a batch of client requests and verify the result
    let client_requests = stream_tracker
        .create_epoch_ending_client_requests(5, 700)
        .unwrap();
    let expected_requests = vec![DataClientRequest::EpochEndingLedgerInfos(
        EpochEndingLedgerInfosRequest {
            start_epoch: 999,
            end_epoch: 999,
        },
    )];
    assert_eq!(client_requests, expected_requests);

    // Update the tracker with a new next request epoch that is at the end
    stream_tracker.next_request_epoch = 1000;

    // Create a batch of client requests and verify an overflow error
    let client_requests = stream_tracker.create_epoch_ending_client_requests(10, 50);
    assert_matches!(client_requests, Err(Error::IntegerOverflow(_)));
}

#[test]
fn test_epoch_ending_stream_tracker() {
    // Create an epoch ending stream request
    let stream_request =
        StreamRequest::GetAllEpochEndingLedgerInfos(GetAllEpochEndingLedgerInfosRequest {
            start_epoch: 0,
        });

    // Try to create a progress stream tracker where there is no advertised data
    // and verify an error is returned.
    let result =
        StreamProgressTracker::new(&stream_request, &GlobalDataSummary::empty().advertised_data);
    assert_matches!(result, Err(Error::DataIsUnavailable(_)));

    // Create a data summary with various advertised epoch ranges (common highest is zero)
    let mut global_data_summary = GlobalDataSummary::empty();
    global_data_summary
        .advertised_data
        .epoch_ending_ledger_infos = vec![
        CompleteDataRange::new(0, 0),
        CompleteDataRange::new(0, 0),
        CompleteDataRange::new(0, 1),
    ];

    // Try to create a progress stream tracker where the highest epoch is zero
    // and verify an error is returned.
    let result = StreamProgressTracker::new(&stream_request, &global_data_summary.advertised_data);
    assert_matches!(result, Err(Error::NoDataToFetch(_)));

    // Create a global data summary with non-zero advertised epoch ranges
    let mut global_data_summary = GlobalDataSummary::empty();
    global_data_summary
        .advertised_data
        .epoch_ending_ledger_infos = vec![
        CompleteDataRange::new(0, 1),
        CompleteDataRange::new(0, 100),
        CompleteDataRange::new(0, 99999999),
        CompleteDataRange::new(0, 100),
    ];

    // Create a new data stream progress tracker and verify the most common highest
    // epoch is chosen.
    let stream_progress_tracker =
        StreamProgressTracker::new(&stream_request, &global_data_summary.advertised_data).unwrap();
    let StreamProgressTracker::EpochEndingStreamTracker(stream_tracker) = stream_progress_tracker;
    assert_eq!(stream_tracker.end_epoch, 99); // End epoch is highest - 1
}

#[test]
fn test_update_epoch_ending_request_progress() {
    // Create a new data stream progress tracker
    let mut stream_progress_tracker = create_epoch_ending_progress_tracker(0, 1000);

    // Update the progress tracker using valid sent request notifications
    for i in 0..10 {
        let start_epoch = i * 100;
        let end_epoch = (i * 100) + 99;
        let client_request =
            DataClientRequest::EpochEndingLedgerInfos(EpochEndingLedgerInfosRequest {
                start_epoch,
                end_epoch,
            });
        stream_progress_tracker
            .update_request_progress(&client_request)
            .unwrap();

        // Verify internal state
        let StreamProgressTracker::EpochEndingStreamTracker(stream_tracker) =
            &stream_progress_tracker;
        assert_eq!(stream_tracker.next_request_epoch, end_epoch + 1);
    }
}

#[test]
#[should_panic(expected = "Updating an epoch ending tracker with an old request!")]
fn test_update_epoch_ending_request_panic() {
    // Create a new data stream progress tracker
    let mut stream_progress_tracker = create_epoch_ending_progress_tracker(0, 1000);

    // Update the tracker with a valid request
    let sent_data_notification =
        DataClientRequest::EpochEndingLedgerInfos(EpochEndingLedgerInfosRequest {
            start_epoch: 0,
            end_epoch: 100,
        });
    stream_progress_tracker
        .update_request_progress(&sent_data_notification)
        .unwrap();

    // Update the tracker with a request that misses data and verify a panic
    let sent_data_notification =
        DataClientRequest::EpochEndingLedgerInfos(EpochEndingLedgerInfosRequest {
            start_epoch: 102,
            end_epoch: 200,
        });
    stream_progress_tracker
        .update_request_progress(&sent_data_notification)
        .unwrap();
}

#[test]
fn test_update_epoch_ending_stream_progress() {
    // Create a new data stream progress tracker
    let mut stream_progress_tracker = create_epoch_ending_progress_tracker(0, 1000);

    // Update the progress tracker using valid sent data notifications
    for i in 0..10 {
        let start_epoch = i * 100;
        let end_epoch = (i * 100) + 99;
        let sent_data_notification = create_sent_data_notification(
            DataClientRequest::EpochEndingLedgerInfos(EpochEndingLedgerInfosRequest {
                start_epoch,
                end_epoch,
            }),
        );
        stream_progress_tracker
            .update_notification_progress(&sent_data_notification)
            .unwrap();

        // Verify internal state
        let StreamProgressTracker::EpochEndingStreamTracker(stream_tracker) =
            &stream_progress_tracker;
        assert_eq!(stream_tracker.next_stream_epoch, end_epoch + 1);
    }
}

#[test]
#[should_panic(expected = "Updating an epoch ending tracker with an old notification!")]
fn test_update_epoch_ending_stream_panic() {
    // Create a new data stream progress tracker
    let mut stream_progress_tracker = create_epoch_ending_progress_tracker(0, 1000);

    // Update the tracker with a valid notification
    let sent_data_notification = create_sent_data_notification(
        DataClientRequest::EpochEndingLedgerInfos(EpochEndingLedgerInfosRequest {
            start_epoch: 0,
            end_epoch: 100,
        }),
    );
    stream_progress_tracker
        .update_notification_progress(&sent_data_notification)
        .unwrap();

    // Update the tracker with an old notification and verify a panic
    let sent_data_notification = create_sent_data_notification(
        DataClientRequest::EpochEndingLedgerInfos(EpochEndingLedgerInfosRequest {
            start_epoch: 50,
            end_epoch: 1100,
        }),
    );
    stream_progress_tracker
        .update_notification_progress(&sent_data_notification)
        .unwrap();
}

fn create_epoch_ending_progress_tracker(
    start_epoch: u64,
    max_advertised_epoch: u64,
) -> StreamProgressTracker {
    // Create an epoch ending stream request
    let stream_request =
        StreamRequest::GetAllEpochEndingLedgerInfos(GetAllEpochEndingLedgerInfosRequest {
            start_epoch,
        });

    // Create a global data summary with a single epoch range
    let mut global_data_summary = GlobalDataSummary::empty();
    global_data_summary
        .advertised_data
        .epoch_ending_ledger_infos =
        vec![CompleteDataRange::new(start_epoch, max_advertised_epoch)];

    // Create a new data stream progress tracker
    StreamProgressTracker::new(&stream_request, &global_data_summary.advertised_data).unwrap()
}

fn create_sent_data_notification(client_request: DataClientRequest) -> SentDataNotification {
    SentDataNotification {
        client_request,
        client_response: DataClientResponse {
            response_id: 0,
            response_payload: DataClientPayload::EpochEndingLedgerInfos(vec![]),
        },
    }
}
