// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    data_notification::{
        DataClientRequest, DataClientRequest::EpochEndingLedgerInfos,
        EpochEndingLedgerInfosRequest, SentDataNotification,
    },
    error::Error,
    streaming_client::{Epoch, GetAllEpochEndingLedgerInfosRequest, StreamRequest},
};
use diem_data_client::AdvertisedData;
use itertools::Itertools;
use std::cmp;

/// An enum holding different types of data streams and progress tracking
/// indicators for tracking and serving that data along the stream.
#[derive(Debug)]
pub enum StreamProgressTracker {
    EpochEndingStreamTracker(EpochEndingStreamTracker),
}

impl StreamProgressTracker {
    pub fn new(
        stream_request: &StreamRequest,
        advertised_data: &AdvertisedData,
    ) -> Result<StreamProgressTracker, Error> {
        match stream_request {
            StreamRequest::GetAllEpochEndingLedgerInfos(request) => {
                StreamProgressTracker::new_epoch_ending_stream_tracker(request, advertised_data)
            }
            _ => Err(Error::UnsupportedRequestEncountered(format!(
                "Stream request not currently supported: {:?}",
                stream_request
            ))),
        }
    }

    fn new_epoch_ending_stream_tracker(
        request: &GetAllEpochEndingLedgerInfosRequest,
        advertised_data: &AdvertisedData,
    ) -> Result<StreamProgressTracker, Error> {
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

        Ok(StreamProgressTracker::EpochEndingStreamTracker(
            EpochEndingStreamTracker {
                request: request.clone(),
                end_epoch,
                next_stream_epoch: request.start_epoch,
                next_request_epoch: request.start_epoch,
            },
        ))
    }

    /// Updates the progress of the sent notifications using the given notification
    pub fn update_notification_progress(
        &mut self,
        sent_data_notification: &SentDataNotification,
    ) -> Result<(), Error> {
        let StreamProgressTracker::EpochEndingStreamTracker(stream_tracker) = self;

        match &sent_data_notification.client_request {
            EpochEndingLedgerInfos(request) => {
                let expected_next_epoch = stream_tracker.next_stream_epoch;
                if request.start_epoch != expected_next_epoch
                    || request.end_epoch < expected_next_epoch
                {
                    panic!(
                        "Updating an epoch ending tracker with an old notification! Given {:?} but expected epoch: {:?}",
                        request, expected_next_epoch
                    );
                }
                stream_tracker.next_stream_epoch =
                    request.end_epoch.checked_add(1).ok_or_else(|| {
                        Error::IntegerOverflow("Next stream epoch has overflown!".into())
                    })?;
            }
            _ => {
                panic!(
                    "Invalid client request {:?} found for the data stream tracker {:?}",
                    sent_data_notification.client_request, stream_tracker
                );
            }
        }

        Ok(())
    }

    /// Updates the progress of the requested data using the given data request
    pub fn update_request_progress(
        &mut self,
        client_request: &DataClientRequest,
    ) -> Result<(), Error> {
        let StreamProgressTracker::EpochEndingStreamTracker(stream_tracker) = self;

        match client_request {
            EpochEndingLedgerInfos(request) => {
                let expected_next_epoch = stream_tracker.next_request_epoch;
                if request.start_epoch != expected_next_epoch
                    || request.end_epoch < expected_next_epoch
                {
                    panic!(
                        "Updating an epoch ending tracker with an old request! Given {:?} but expected epoch: {:?}",
                        request, expected_next_epoch
                    );
                }
                stream_tracker.next_request_epoch =
                    request.end_epoch.checked_add(1).ok_or_else(|| {
                        Error::IntegerOverflow("Next stream epoch has overflown!".into())
                    })?;
            }
            _ => {
                panic!(
                    "Invalid client request {:?} found for the data stream tracker {:?}",
                    client_request, stream_tracker
                );
            }
        }

        Ok(())
    }

    /// Verifies that the data required by the stream can be satisfied using the
    /// currently advertised data in the network. If not, returns an error.
    pub fn ensure_data_is_available(&self, advertised_data: &AdvertisedData) -> Result<(), Error> {
        match self {
            StreamProgressTracker::EpochEndingStreamTracker(stream_tracker) => {
                if stream_tracker.epoch_ending_ledger_infos_available(advertised_data) {
                    return Ok(());
                }
            }
        }

        Err(Error::DataIsUnavailable(format!(
            "Unable to satisfy requested data stream: {:?}, with advertised data: {:?}",
            self, advertised_data
        )))
    }
}

#[derive(Debug)]
pub struct EpochEndingStreamTracker {
    // The original epoch ending ledger infos request made by the client
    pub request: GetAllEpochEndingLedgerInfosRequest,

    // The last epoch ending ledger info that this stream will send to the client
    pub end_epoch: Epoch,

    // The next epoch that we're waiting to send to the client along the
    // stream. All epochs before this have already been sent.
    pub next_stream_epoch: Epoch,

    // The next epoch that we're waiting to request from the network. All epochs
    // before this have already been requested, but not necessarily sent to the
    // client via the stream (e.g., the requests may still be in-flight).
    pub next_request_epoch: Epoch,
}

impl EpochEndingStreamTracker {
    /// Returns true iff all epoch ending ledger infos required by the stream
    /// are available in the advertised data.
    pub fn epoch_ending_ledger_infos_available(&self, advertised_data: &AdvertisedData) -> bool {
        let start_epoch = self.request.start_epoch;
        let end_epoch = self.end_epoch;

        // Verify all epoch ending ledger infos can be found in the advertised data
        for epoch in start_epoch..=end_epoch {
            let mut epoch_exists = false;
            for epoch_range in &advertised_data.epoch_ending_ledger_infos {
                if !epoch_exists && epoch_range.contains(epoch) {
                    epoch_exists = true;
                }
            }

            if !epoch_exists {
                return false;
            }
        }
        true
    }

    /// Creates epoch ending payload requests for the Diem data client using the
    /// given stream tracker. At most `max_number_of_requests` will be created.
    pub fn create_epoch_ending_client_requests(
        &mut self,
        max_number_of_requests: u64,
        optimal_epoch_chunk_size: u64,
    ) -> Result<Vec<DataClientRequest>, Error> {
        // Calculate the total number of epochs left to satisfy the stream
        let start_epoch = self.next_request_epoch;
        let end_epoch = self.end_epoch;
        let mut total_epochs_to_fetch = end_epoch
            .checked_sub(start_epoch)
            .and_then(|e| e.checked_add(1)) // = end_epoch - start_epoch + 1
            .ok_or_else(|| Error::IntegerOverflow("Total epochs to fetch has overflown!".into()))?;

        // Iterate until we've requested all epochs or hit the maximum number of requests
        let mut data_client_requests = vec![];
        let mut num_requests_made = 0;
        let mut next_epoch_to_request = self.next_request_epoch;
        while total_epochs_to_fetch > 0 && num_requests_made < max_number_of_requests {
            // Calculate the number of epochs to fetch in this request
            let num_epochs_to_fetch = cmp::min(total_epochs_to_fetch, optimal_epoch_chunk_size);

            // Calculate the start and end epochs for the request
            let request_start_epoch = next_epoch_to_request;
            let request_end_epoch = request_start_epoch
                .checked_add(num_epochs_to_fetch)
                .and_then(|e| e.checked_sub(1)) // = request_start_epoch + num_epochs_to_fetch - 1
                .ok_or_else(|| {
                    Error::IntegerOverflow("End epoch to fetch has overflown!".into())
                })?;

            // Create the data client requests
            let data_client_request =
                DataClientRequest::EpochEndingLedgerInfos(EpochEndingLedgerInfosRequest {
                    start_epoch: request_start_epoch,
                    end_epoch: request_end_epoch,
                });
            data_client_requests.push(data_client_request);

            // Update the local loop state
            next_epoch_to_request = request_end_epoch.checked_add(1).ok_or_else(|| {
                Error::IntegerOverflow("Next epoch to request has overflown!".into())
            })?;
            total_epochs_to_fetch = total_epochs_to_fetch
                .checked_sub(num_epochs_to_fetch)
                .ok_or_else(|| {
                    Error::IntegerOverflow("Total epochs to fetch has overflown!".into())
                })?;
            num_requests_made = num_requests_made.checked_add(1).ok_or_else(|| {
                Error::IntegerOverflow("Number of payload requests has overflown!".into())
            })?;
        }

        Ok(data_client_requests)
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
