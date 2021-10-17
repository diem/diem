// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    data_notification::{
        DataClientRequest,
        DataClientRequest::{EpochEndingLedgerInfos, TransactionsWithProof},
        EpochEndingLedgerInfosRequest, SentDataNotification, TransactionsWithProofRequest,
    },
    error::Error,
    streaming_client::{
        Epoch, GetAllEpochEndingLedgerInfosRequest, GetAllTransactionsRequest, StreamRequest,
    },
};
use diem_data_client::{AdvertisedData, OptimalChunkSizes};
use diem_types::transaction::Version;
use enum_dispatch::enum_dispatch;
use itertools::Itertools;
use std::cmp;

/// The interface offered by each stream tracker.
#[enum_dispatch]
pub trait DataStreamTracker {
    /// Creates a batch of data client requests (up to `max_number_of_requests`)
    /// that can be sent to the diem data client to progress the stream.
    fn create_data_client_requests(
        &self,
        max_number_of_requests: u64,
        optimal_chunk_sizes: &OptimalChunkSizes,
    ) -> Result<Vec<DataClientRequest>, Error>;

    /// Returns true iff all remaining data required to satisfy the stream is
    /// available in the given advertised data.
    fn is_remaining_data_available(&self, advertised_data: &AdvertisedData) -> bool;

    /// Updates the last sent notification for the stream ( i.e., the last
    /// notification that was sent to the client). This keeps track of what data
    /// has actually been received by the stream listener.
    fn update_notification_tracking(
        &mut self,
        sent_data_notification: &SentDataNotification,
    ) -> Result<(), Error>;

    /// Updates the last sent request for the stream ( i.e., the last client
    /// request that was created and sent to the network). This keeps
    /// track of what data has already been requested.
    fn update_request_tracking(&mut self, client_request: &DataClientRequest) -> Result<(), Error>;
}

/// A single progress tracker that allows each data stream type to track and
/// update progress through the `DataStreamTracker` interface.
#[enum_dispatch(DataStreamTracker)]
#[derive(Debug)]
pub enum StreamProgressTracker {
    EpochEndingStreamTracker,
    TransactionStreamTracker,
}

impl StreamProgressTracker {
    pub fn new(
        stream_request: &StreamRequest,
        advertised_data: &AdvertisedData,
    ) -> Result<Self, Error> {
        // Identify the type of stream tracker we need based on the stream request
        match stream_request {
            StreamRequest::GetAllEpochEndingLedgerInfos(request) => {
                Ok(EpochEndingStreamTracker::new(request, advertised_data)?.into())
            }
            StreamRequest::GetAllTransactions(request) => {
                Ok(TransactionStreamTracker::new(request)?.into())
            }
            _ => Err(Error::UnsupportedRequestEncountered(format!(
                "Stream request not currently supported: {:?}",
                stream_request
            ))),
        }
    }
}

#[derive(Clone, Debug)]
pub struct EpochEndingStreamTracker {
    // The original epoch ending ledger infos request made by the client
    pub request: GetAllEpochEndingLedgerInfosRequest,

    // The last epoch ending ledger info that this stream will send to the client
    pub end_epoch: Epoch,

    // The next epoch that we're waiting to send to the client along the
    // stream. All epochs before this have already been sent.
    pub next_stream_epoch: Epoch,

    // The next epoch that we're waiting to request from the network. All epochs
    // before this have already been requested.
    pub next_request_epoch: Epoch,
}

impl EpochEndingStreamTracker {
    fn new(
        request: &GetAllEpochEndingLedgerInfosRequest,
        advertised_data: &AdvertisedData,
    ) -> Result<Self, Error> {
        let end_epoch = match most_common_highest_epoch(advertised_data) {
            Some(max_advertised_epoch) => {
                if max_advertised_epoch == 0 {
                    return Err(Error::NoDataToFetch(
                        "The maximum advertised epoch is 0. No epoch changes have occurred!".into(),
                    ));
                } else {
                    max_advertised_epoch.checked_sub(1).ok_or_else(|| {
                        Error::IntegerOverflow("Maximum advertised epoch has underflow!".into())
                    })?
                }
            }
            None => {
                return Err(Error::DataIsUnavailable(format!(
                    "Unable to find any maximum advertised epoch in the network: {:?}",
                    advertised_data
                )));
            }
        };

        if end_epoch < request.start_epoch {
            return Err(Error::DataIsUnavailable(format!(
                "The epoch to start syncing from is higher than any advertised highest epoch! Highest: {:?}, start: {:?}",
                end_epoch, request.start_epoch
            )));
        }

        Ok(EpochEndingStreamTracker {
            request: request.clone(),
            end_epoch,
            next_stream_epoch: request.start_epoch,
            next_request_epoch: request.start_epoch,
        })
    }
}

impl DataStreamTracker for EpochEndingStreamTracker {
    fn create_data_client_requests(
        &self,
        max_number_of_requests: u64,
        optimal_chunk_sizes: &OptimalChunkSizes,
    ) -> Result<Vec<DataClientRequest>, Error> {
        create_data_client_requests(
            self.next_request_epoch,
            self.end_epoch,
            max_number_of_requests,
            optimal_chunk_sizes.epoch_chunk_size,
            self.clone().into(),
        )
    }

    fn is_remaining_data_available(&self, advertised_data: &AdvertisedData) -> bool {
        let start_epoch = self.next_stream_epoch;
        let end_epoch = self.end_epoch;
        AdvertisedData::contains_range(
            start_epoch,
            end_epoch,
            &advertised_data.epoch_ending_ledger_infos,
        )
    }

    fn update_notification_tracking(
        &mut self,
        sent_data_notification: &SentDataNotification,
    ) -> Result<(), Error> {
        match &sent_data_notification.client_request {
            EpochEndingLedgerInfos(request) => {
                verify_client_request_indices(
                    self.next_stream_epoch,
                    request.start_epoch,
                    request.end_epoch,
                );
                self.next_stream_epoch = request.end_epoch.checked_add(1).ok_or_else(|| {
                    Error::IntegerOverflow("Next stream epoch has overflown!".into())
                })?;
            }
            client_request => {
                invalid_client_request(client_request, self.clone().into());
            }
        }
        Ok(())
    }

    fn update_request_tracking(&mut self, client_request: &DataClientRequest) -> Result<(), Error> {
        match client_request {
            EpochEndingLedgerInfos(request) => {
                verify_client_request_indices(
                    self.next_request_epoch,
                    request.start_epoch,
                    request.end_epoch,
                );
                self.next_request_epoch = request.end_epoch.checked_add(1).ok_or_else(|| {
                    Error::IntegerOverflow("Next request epoch has overflown!".into())
                })?;
            }
            client_request => {
                invalid_client_request(client_request, self.clone().into());
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct TransactionStreamTracker {
    // The original transaction request made by the client
    pub request: GetAllTransactionsRequest,

    // The next transaction version that we're waiting to send to the client
    // along the stream. All transactions before this have been sent.
    pub next_stream_version: Version,

    // The next transaction version that we're waiting to request from the
    // network. All transactions before this have already been requested.
    pub next_request_version: Epoch,
}

impl TransactionStreamTracker {
    fn new(request: &GetAllTransactionsRequest) -> Result<Self, Error> {
        Ok(TransactionStreamTracker {
            request: request.clone(),
            next_stream_version: request.start_version,
            next_request_version: request.start_version,
        })
    }
}

impl DataStreamTracker for TransactionStreamTracker {
    fn create_data_client_requests(
        &self,
        max_number_of_requests: u64,
        optimal_chunk_sizes: &OptimalChunkSizes,
    ) -> Result<Vec<DataClientRequest>, Error> {
        create_data_client_requests(
            self.next_request_version,
            self.request.end_version,
            max_number_of_requests,
            optimal_chunk_sizes.transaction_chunk_size,
            self.clone().into(),
        )
    }

    fn is_remaining_data_available(&self, advertised_data: &AdvertisedData) -> bool {
        let start_version = self.next_stream_version;
        let end_version = self.request.end_version;
        AdvertisedData::contains_range(start_version, end_version, &advertised_data.transactions)
    }

    fn update_notification_tracking(
        &mut self,
        sent_data_notification: &SentDataNotification,
    ) -> Result<(), Error> {
        match &sent_data_notification.client_request {
            TransactionsWithProof(request) => {
                verify_client_request_indices(
                    self.next_stream_version,
                    request.start_version,
                    request.end_version,
                );
                self.next_stream_version = request.end_version.checked_add(1).ok_or_else(|| {
                    Error::IntegerOverflow("Next stream version has overflown!".into())
                })?;
            }
            client_request => {
                invalid_client_request(client_request, self.clone().into());
            }
        }
        Ok(())
    }

    fn update_request_tracking(&mut self, client_request: &DataClientRequest) -> Result<(), Error> {
        match client_request {
            TransactionsWithProof(request) => {
                verify_client_request_indices(
                    self.next_request_version,
                    request.start_version,
                    request.end_version,
                );
                self.next_request_version =
                    request.end_version.checked_add(1).ok_or_else(|| {
                        Error::IntegerOverflow("Next request version has overflown!".into())
                    })?;
            }
            client_request => {
                invalid_client_request(client_request, self.clone().into());
            }
        }
        Ok(())
    }
}

/// Verifies that the `expected_next_index` matches the `start_index` and that
/// the `end_index` is greater than or equal to `expected_next_index`.
fn verify_client_request_indices(expected_next_index: u64, start_index: u64, end_index: u64) {
    if start_index != expected_next_index {
        panic!(
            "The start index did not match the expected next index! Given: {:?}, expected: {:?}",
            start_index, expected_next_index
        );
    }
    if end_index < expected_next_index {
        panic!(
            "The end index was less than the expected next index! Given: {:?}, expected: {:?}",
            end_index, expected_next_index
        );
    }
}

fn invalid_client_request(
    client_request: &DataClientRequest,
    stream_progress_tracker: StreamProgressTracker,
) {
    panic!(
        "Invalid client request {:?} found for the data stream tracker {:?}",
        client_request, stream_progress_tracker
    );
}

/// Creates a batch of data client requests for the given stream progress tracker
fn create_data_client_requests(
    start_index: u64,
    end_index: u64,
    max_number_of_requests: u64,
    optimal_chunk_size: u64,
    stream_progress_tracker: StreamProgressTracker,
) -> Result<Vec<DataClientRequest>, Error> {
    // Calculate the total number of items left to satisfy the stream
    let mut total_items_to_fetch = end_index
        .checked_sub(start_index)
        .and_then(|e| e.checked_add(1)) // = end_index - start_index + 1
        .ok_or_else(|| Error::IntegerOverflow("Total items to fetch has overflown!".into()))?;

    // Iterate until we've requested all transactions or hit the maximum number of requests
    let mut data_client_requests = vec![];
    let mut num_requests_made = 0;
    let mut next_index_to_request = start_index;
    while total_items_to_fetch > 0 && num_requests_made < max_number_of_requests {
        // Calculate the number of items to fetch in this request
        let num_items_to_fetch = cmp::min(total_items_to_fetch, optimal_chunk_size);

        // Calculate the start and end indices for the request
        let request_start_index = next_index_to_request;
        let request_end_index = request_start_index
            .checked_add(num_items_to_fetch)
            .and_then(|e| e.checked_sub(1)) // = request_start_index + num_items_to_fetch - 1
            .ok_or_else(|| Error::IntegerOverflow("End index to fetch has overflown!".into()))?;

        // Create the data client requests
        let data_client_request = create_data_client_request(
            request_start_index,
            request_end_index,
            &stream_progress_tracker,
        );
        data_client_requests.push(data_client_request);

        // Update the local loop state
        next_index_to_request = request_end_index
            .checked_add(1)
            .ok_or_else(|| Error::IntegerOverflow("Next index to request has overflown!".into()))?;
        total_items_to_fetch = total_items_to_fetch
            .checked_sub(num_items_to_fetch)
            .ok_or_else(|| Error::IntegerOverflow("Total items to fetch has overflown!".into()))?;
        num_requests_made = num_requests_made.checked_add(1).ok_or_else(|| {
            Error::IntegerOverflow("Number of payload requests has overflown!".into())
        })?;
    }

    Ok(data_client_requests)
}

/// Creates a data client request for the given stream tracker using the
/// specified start and end indices.
fn create_data_client_request(
    start_index: u64,
    end_index: u64,
    stream_progress_tracker: &StreamProgressTracker,
) -> DataClientRequest {
    match stream_progress_tracker {
        StreamProgressTracker::EpochEndingStreamTracker(_) => {
            DataClientRequest::EpochEndingLedgerInfos(EpochEndingLedgerInfosRequest {
                start_epoch: start_index,
                end_epoch: end_index,
            })
        }
        StreamProgressTracker::TransactionStreamTracker(stream_tracker) => {
            DataClientRequest::TransactionsWithProof(TransactionsWithProofRequest {
                start_version: start_index,
                end_version: end_index,
                max_proof_version: stream_tracker.request.max_proof_version,
                include_events: stream_tracker.request.include_events,
            })
        }
    }
}

/// Returns the most common highest epoch advertised in the network.
/// Note: we use this to reduce the likelihood of malicious nodes
/// interfering with syncing progress by advertising non-existent epochs.
fn most_common_highest_epoch(advertised_data: &AdvertisedData) -> Option<Epoch> {
    // Count the frequencies of the highest epochs
    let highest_epoch_frequencies = advertised_data
        .epoch_ending_ledger_infos
        .iter()
        .map(|epoch_range| epoch_range.highest)
        .clone()
        .counts();

    // Return the most common epoch
    highest_epoch_frequencies
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(epoch, _)| epoch)
}
