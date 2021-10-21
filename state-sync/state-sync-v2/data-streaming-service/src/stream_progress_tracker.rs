// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    data_notification::{
        AccountsWithProofRequest, DataClientRequest,
        DataClientRequest::{
            AccountsWithProof, EpochEndingLedgerInfos, NumberOfAccounts,
            TransactionOutputsWithProof, TransactionsWithProof,
        },
        DataNotification, DataPayload, EpochEndingLedgerInfosRequest, NumberOfAccountsRequest,
        TransactionOutputsWithProofRequest, TransactionsWithProofRequest,
    },
    error::Error,
    streaming_client::{
        ContinuouslyStreamTransactionsRequest, Epoch, GetAllAccountsRequest,
        GetAllEpochEndingLedgerInfosRequest, GetAllTransactionOutputsRequest,
        GetAllTransactionsRequest, StreamRequest,
    },
};
use diem_data_client::{
    AdvertisedData, DataClientPayload, DataClientPayload::NumberOfAccountStates,
    DataClientResponse, GlobalDataSummary,
};
use diem_types::{ledger_info::LedgerInfoWithSignatures, transaction::Version};
use enum_dispatch::enum_dispatch;
use itertools::Itertools;
use std::{
    cmp,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

/// The interface offered by each stream tracker.
#[enum_dispatch]
pub trait DataStreamTracker {
    /// Creates a batch of data client requests (up to `max_number_of_requests`)
    /// that can be sent to the diem data client to progress the stream.
    fn create_data_client_requests(
        &mut self,
        max_number_of_requests: u64,
        global_data_summary: &GlobalDataSummary,
    ) -> Result<Vec<DataClientRequest>, Error>;

    /// Returns true iff all remaining data required to satisfy the stream is
    /// available in the given advertised data.
    fn is_remaining_data_available(&self, advertised_data: &AdvertisedData) -> bool;

    /// Transforms a given data client response (for the previously sent
    /// request) into a data notification to be sent along the data stream.
    /// Note: this call may return `None`, in which case, no notification needs
    /// to be sent to the client.
    fn transform_client_response_into_notification(
        &mut self,
        client_request: &DataClientRequest,
        client_response: &DataClientResponse,
        notification_id_generator: Arc<AtomicU64>,
    ) -> Result<Option<DataNotification>, Error>;

    /// Updates the last sent request for the stream ( i.e., the last client
    /// request that was created and sent to the network). This keeps
    /// track of what data has already been requested.
    fn update_request_tracking(&mut self, client_request: &DataClientRequest) -> Result<(), Error>;
}

/// A single progress tracker that allows each data stream type to track and
/// update progress through the `DataStreamTracker` interface.
#[enum_dispatch(DataStreamTracker)]
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum StreamProgressTracker {
    AccountsStreamTracker,
    ContinuousTransactionStreamTracker,
    EpochEndingStreamTracker,
    TransactionOutputStreamTracker,
    TransactionStreamTracker,
}

impl StreamProgressTracker {
    pub fn new(
        stream_request: &StreamRequest,
        advertised_data: &AdvertisedData,
    ) -> Result<Self, Error> {
        // Identify the type of stream tracker we need based on the stream request
        match stream_request {
            StreamRequest::ContinuouslyStreamTransactions(request) => {
                Ok(ContinuousTransactionStreamTracker::new(request)?.into())
            }
            StreamRequest::GetAllAccounts(request) => {
                Ok(AccountsStreamTracker::new(request)?.into())
            }
            StreamRequest::GetAllEpochEndingLedgerInfos(request) => {
                Ok(EpochEndingStreamTracker::new(request, advertised_data)?.into())
            }
            StreamRequest::GetAllTransactionOutputs(request) => {
                Ok(TransactionOutputStreamTracker::new(request)?.into())
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
pub struct AccountsStreamTracker {
    // The original accounts request made by the client
    pub request: GetAllAccountsRequest,

    // True iff a request has been created to fetch the number of accounts
    pub account_num_requested: bool,

    // The total number of accounts to fetch at this version
    pub number_of_accounts: Option<u64>,

    // The next account index that we're waiting to send to the client along the
    // stream. All accounts before this index have already been sent.
    pub next_stream_index: u64,

    // The next account index that we're waiting to request from the network.
    // All accounts before this index have already been requested.
    pub next_request_index: u64,
}

impl AccountsStreamTracker {
    fn new(request: &GetAllAccountsRequest) -> Result<Self, Error> {
        Ok(AccountsStreamTracker {
            request: request.clone(),
            account_num_requested: false,
            number_of_accounts: None,
            next_stream_index: 0,
            next_request_index: 0,
        })
    }
}

impl DataStreamTracker for AccountsStreamTracker {
    fn create_data_client_requests(
        &mut self,
        max_number_of_requests: u64,
        global_data_summary: &GlobalDataSummary,
    ) -> Result<Vec<DataClientRequest>, Error> {
        if let Some(number_of_accounts) = self.number_of_accounts {
            create_data_client_requests(
                self.next_request_index,
                number_of_accounts,
                max_number_of_requests,
                global_data_summary
                    .optimal_chunk_sizes
                    .account_states_chunk_size,
                self.clone().into(),
            )
        } else if self.account_num_requested {
            Ok(vec![]) // Wait for the number of accounts to be returned
        } else {
            let client_request = DataClientRequest::NumberOfAccounts(NumberOfAccountsRequest {
                version: self.request.version,
            });
            Ok(vec![client_request])
        }
    }

    fn is_remaining_data_available(&self, advertised_data: &AdvertisedData) -> bool {
        AdvertisedData::contains_range(
            self.request.version,
            self.request.version,
            &advertised_data.account_states,
        )
    }

    fn transform_client_response_into_notification(
        &mut self,
        client_request: &DataClientRequest,
        client_response: &DataClientResponse,
        notification_id_generator: Arc<AtomicU64>,
    ) -> Result<Option<DataNotification>, Error> {
        match client_request {
            AccountsWithProof(request) => {
                verify_client_request_indices(
                    self.next_stream_index,
                    request.start_index,
                    request.end_index,
                );

                // Update the local stream notification tracker
                self.next_stream_index = request.end_index.checked_add(1).ok_or_else(|| {
                    Error::IntegerOverflow("Next stream index has overflown!".into())
                })?;

                // Create a new data notification
                let data_notification = create_data_notification(
                    notification_id_generator,
                    client_response,
                    self.clone().into(),
                );
                return Ok(Some(data_notification));
            }
            NumberOfAccounts(_) => {
                if let NumberOfAccountStates(number_of_accounts) = client_response.response_payload
                {
                    // We got a response. Save the number of accounts.
                    self.number_of_accounts = Some(number_of_accounts);
                }
            }
            client_request => {
                invalid_client_request(client_request, self.clone().into());
            }
        }
        Ok(None)
    }

    fn update_request_tracking(&mut self, client_request: &DataClientRequest) -> Result<(), Error> {
        match client_request {
            AccountsWithProof(request) => {
                verify_client_request_indices(
                    self.next_request_index,
                    request.start_index,
                    request.end_index,
                );
                self.next_request_index = request.end_index.checked_add(1).ok_or_else(|| {
                    Error::IntegerOverflow("Next request index has overflown!".into())
                })?;
            }
            NumberOfAccounts(_) => {
                self.account_num_requested = true;
            }
            client_request => {
                invalid_client_request(client_request, self.clone().into());
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct ContinuousTransactionStreamTracker {
    // The original stream request made by the client
    pub request: ContinuouslyStreamTransactionsRequest,

    // The target ledger info that we're currently syncing to
    pub target_ledger_info: Option<LedgerInfoWithSignatures>,

    // True iff a request has been created to fetch an epoch ending ledger info
    pub end_of_epoch_requested: bool,

    // The next transaction version and epoch that we're waiting to send to the
    // client along the stream. All transactions before this version have been sent.
    pub next_stream_version_and_epoch: (Version, Epoch),

    // The next transaction version and epoch that we're waiting to request from
    // the network. All transactions before this version have been requested.
    pub next_request_version_and_epoch: (Version, Epoch),
}

impl ContinuousTransactionStreamTracker {
    fn new(request: &ContinuouslyStreamTransactionsRequest) -> Result<Self, Error> {
        Ok(ContinuousTransactionStreamTracker {
            request: request.clone(),
            target_ledger_info: None,
            end_of_epoch_requested: false,
            next_stream_version_and_epoch: (request.start_version, request.start_epoch),
            next_request_version_and_epoch: (request.start_version, request.start_epoch),
        })
    }

    fn select_new_target_ledger_info(
        &self,
        advertised_data: &AdvertisedData,
    ) -> Result<LedgerInfoWithSignatures, Error> {
        if let Some(highest_synced_ledger_info) = highest_synced_ledger_info(advertised_data) {
            let (next_request_version, _) = self.next_request_version_and_epoch;
            if next_request_version > highest_synced_ledger_info.ledger_info().version() {
                Err(Error::NoDataToFetch(
                    "We're already at the highest synced ledger info version!".into(),
                ))
            } else {
                Ok(highest_synced_ledger_info)
            }
        } else {
            Err(Error::DataIsUnavailable(
                "Unable to find the highest synced ledger info!".into(),
            ))
        }
    }

    fn get_target_ledger_info(&self) -> &LedgerInfoWithSignatures {
        self.target_ledger_info
            .as_ref()
            .expect("No target ledger info found!")
    }
}

impl DataStreamTracker for ContinuousTransactionStreamTracker {
    fn create_data_client_requests(
        &mut self,
        max_number_of_requests: u64,
        global_data_summary: &GlobalDataSummary,
    ) -> Result<Vec<DataClientRequest>, Error> {
        let (next_request_version, next_request_epoch) = self.next_request_version_and_epoch;

        // Check if we have a syncing target and set one if not
        if self.target_ledger_info.is_none() {
            if self.end_of_epoch_requested {
                return Ok(vec![]); // We are waiting for the epoch ending ledger info
            } else {
                // Select a new ledger info from the advertised data
                let new_target_ledger_info =
                    self.select_new_target_ledger_info(&global_data_summary.advertised_data)?;
                if new_target_ledger_info.ledger_info().epoch() > next_request_epoch {
                    // There was an epoch change. Request an epoch ending ledger info.
                    return Ok(vec![DataClientRequest::EpochEndingLedgerInfos(
                        EpochEndingLedgerInfosRequest {
                            start_epoch: next_request_epoch,
                            end_epoch: next_request_epoch,
                        },
                    )]);
                } else {
                    // Set the ledger info as the target
                    self.target_ledger_info = Some(new_target_ledger_info);
                }
            }
        }

        // We have a target ledger info. Create pending requests for that target.
        let target_ledger_info_version = self.get_target_ledger_info().ledger_info().version();
        if next_request_version <= target_ledger_info_version {
            create_data_client_requests(
                next_request_version,
                target_ledger_info_version,
                max_number_of_requests,
                global_data_summary
                    .optimal_chunk_sizes
                    .transaction_chunk_size,
                self.clone().into(),
            )
        } else {
            Ok(vec![]) // Wait until all notifications for the target have been sent.
        }
    }

    fn is_remaining_data_available(&self, advertised_data: &AdvertisedData) -> bool {
        let (next_request_version, _) = self.next_request_version_and_epoch;

        // Verify we can satisfy the next transaction version
        AdvertisedData::contains_range(
            next_request_version,
            next_request_version,
            &advertised_data.transactions,
        )
    }

    fn transform_client_response_into_notification(
        &mut self,
        client_request: &DataClientRequest,
        client_response: &DataClientResponse,
        notification_id_generator: Arc<AtomicU64>,
    ) -> Result<Option<DataNotification>, Error> {
        match client_request {
            TransactionsWithProof(request) => {
                let (next_stream_version, mut next_stream_epoch) =
                    self.next_stream_version_and_epoch;
                verify_client_request_indices(
                    next_stream_version,
                    request.start_version,
                    request.end_version,
                );

                // Create a new data notification
                let data_notification = create_data_notification(
                    notification_id_generator,
                    client_response,
                    self.clone().into(),
                );

                // Update the next stream version and epoch
                if request.end_version == self.get_target_ledger_info().ledger_info().version() {
                    if self.get_target_ledger_info().ledger_info().ends_epoch() {
                        next_stream_epoch = next_stream_epoch.checked_add(1).ok_or_else(|| {
                            Error::IntegerOverflow("Next stream epoch has overflown!".into())
                        })?;
                    }
                    self.target_ledger_info = None; // We've sent all notifications up to the target
                }
                let next_stream_version = request.end_version.checked_add(1).ok_or_else(|| {
                    Error::IntegerOverflow("Next stream version has overflown!".into())
                })?;
                self.next_stream_version_and_epoch = (next_stream_version, next_stream_epoch);

                return Ok(Some(data_notification));
            }
            EpochEndingLedgerInfos(_) => {
                if let DataClientPayload::EpochEndingLedgerInfos(epoch_ending_ledger_infos) =
                    &client_response.response_payload
                {
                    match &epoch_ending_ledger_infos[..] {
                        [target_ledger_info] => {
                            self.target_ledger_info = Some(target_ledger_info.clone());
                        }
                        _ => {
                            panic!(
                                "Invalid epoch ending ledger info payload: {:?}",
                                epoch_ending_ledger_infos
                            );
                            // TODO(joshlind): notify the data client of the bad response
                        }
                    }
                } else {
                    panic!(
                        "Invalid client response payload: {:?}!",
                        &client_response.response_payload
                    );
                    // TODO(joshlind): notify the data client of the bad response
                }
                self.end_of_epoch_requested = false;
            }
            client_request => {
                invalid_client_request(client_request, self.clone().into());
            }
        }
        Ok(None)
    }

    fn update_request_tracking(&mut self, client_request: &DataClientRequest) -> Result<(), Error> {
        match client_request {
            DataClientRequest::EpochEndingLedgerInfos(_) => {
                self.end_of_epoch_requested = true;
            }
            DataClientRequest::TransactionsWithProof(request) => {
                let (next_request_version, mut next_request_epoch) =
                    self.next_request_version_and_epoch;
                verify_client_request_indices(
                    next_request_version,
                    request.start_version,
                    request.end_version,
                );

                // Update the next request version and epoch
                if request.end_version == self.get_target_ledger_info().ledger_info().version()
                    && self.get_target_ledger_info().ledger_info().ends_epoch()
                {
                    // We've hit an epoch change
                    next_request_epoch = next_request_epoch.checked_add(1).ok_or_else(|| {
                        Error::IntegerOverflow("Next request epoch has overflown!".into())
                    })?;
                }
                let next_request_version = request.end_version.checked_add(1).ok_or_else(|| {
                    Error::IntegerOverflow("Next request version has overflown!".into())
                })?;
                self.next_request_version_and_epoch = (next_request_version, next_request_epoch);
            }
            client_request => {
                invalid_client_request(client_request, self.clone().into());
            }
        }
        Ok(())
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
        &mut self,
        max_number_of_requests: u64,
        global_data_summary: &GlobalDataSummary,
    ) -> Result<Vec<DataClientRequest>, Error> {
        create_data_client_requests(
            self.next_request_epoch,
            self.end_epoch,
            max_number_of_requests,
            global_data_summary.optimal_chunk_sizes.epoch_chunk_size,
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

    fn transform_client_response_into_notification(
        &mut self,
        client_request: &DataClientRequest,
        client_response: &DataClientResponse,
        notification_id_generator: Arc<AtomicU64>,
    ) -> Result<Option<DataNotification>, Error> {
        match client_request {
            EpochEndingLedgerInfos(request) => {
                verify_client_request_indices(
                    self.next_stream_epoch,
                    request.start_epoch,
                    request.end_epoch,
                );

                // Update the local stream notification tracker
                self.next_stream_epoch = request.end_epoch.checked_add(1).ok_or_else(|| {
                    Error::IntegerOverflow("Next stream epoch has overflown!".into())
                })?;

                // Create a new data notification
                let data_notification = create_data_notification(
                    notification_id_generator,
                    client_response,
                    self.clone().into(),
                );
                return Ok(Some(data_notification));
            }
            client_request => {
                invalid_client_request(client_request, self.clone().into());
            }
        }
        Ok(None)
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
pub struct TransactionOutputStreamTracker {
    // The original transaction output request made by the client
    pub request: GetAllTransactionOutputsRequest,

    // The next transaction output version that we're waiting to send to the
    // client along the stream. All outputs before this have been sent.
    pub next_stream_version: Version,

    // The next transaction output version that we're waiting to request from
    // the network. All outputs before this have already been requested.
    pub next_request_version: Epoch,
}

impl TransactionOutputStreamTracker {
    fn new(request: &GetAllTransactionOutputsRequest) -> Result<Self, Error> {
        Ok(TransactionOutputStreamTracker {
            request: request.clone(),
            next_stream_version: request.start_version,
            next_request_version: request.start_version,
        })
    }
}

impl DataStreamTracker for TransactionOutputStreamTracker {
    fn create_data_client_requests(
        &mut self,
        max_number_of_requests: u64,
        global_data_summary: &GlobalDataSummary,
    ) -> Result<Vec<DataClientRequest>, Error> {
        create_data_client_requests(
            self.next_request_version,
            self.request.end_version,
            max_number_of_requests,
            global_data_summary
                .optimal_chunk_sizes
                .transaction_output_chunk_size,
            self.clone().into(),
        )
    }

    fn is_remaining_data_available(&self, advertised_data: &AdvertisedData) -> bool {
        let start_version = self.next_stream_version;
        let end_version = self.request.end_version;
        AdvertisedData::contains_range(
            start_version,
            end_version,
            &advertised_data.transaction_outputs,
        )
    }

    fn transform_client_response_into_notification(
        &mut self,
        client_request: &DataClientRequest,
        client_response: &DataClientResponse,
        notification_id_generator: Arc<AtomicU64>,
    ) -> Result<Option<DataNotification>, Error> {
        match client_request {
            TransactionOutputsWithProof(request) => {
                verify_client_request_indices(
                    self.next_stream_version,
                    request.start_version,
                    request.end_version,
                );

                // Update the local stream notification tracker
                self.next_stream_version = request.end_version.checked_add(1).ok_or_else(|| {
                    Error::IntegerOverflow("Next stream version has overflown!".into())
                })?;

                // Create a new data notification
                let data_notification = create_data_notification(
                    notification_id_generator,
                    client_response,
                    self.clone().into(),
                );
                return Ok(Some(data_notification));
            }
            client_request => {
                invalid_client_request(client_request, self.clone().into());
            }
        }
        Ok(None)
    }

    fn update_request_tracking(&mut self, client_request: &DataClientRequest) -> Result<(), Error> {
        match client_request {
            TransactionOutputsWithProof(request) => {
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
        &mut self,
        max_number_of_requests: u64,
        global_data_summary: &GlobalDataSummary,
    ) -> Result<Vec<DataClientRequest>, Error> {
        create_data_client_requests(
            self.next_request_version,
            self.request.end_version,
            max_number_of_requests,
            global_data_summary
                .optimal_chunk_sizes
                .transaction_chunk_size,
            self.clone().into(),
        )
    }

    fn is_remaining_data_available(&self, advertised_data: &AdvertisedData) -> bool {
        let start_version = self.next_stream_version;
        let end_version = self.request.end_version;
        AdvertisedData::contains_range(start_version, end_version, &advertised_data.transactions)
    }

    fn transform_client_response_into_notification(
        &mut self,
        client_request: &DataClientRequest,
        client_response: &DataClientResponse,
        notification_id_generator: Arc<AtomicU64>,
    ) -> Result<Option<DataNotification>, Error> {
        match client_request {
            TransactionsWithProof(request) => {
                verify_client_request_indices(
                    self.next_stream_version,
                    request.start_version,
                    request.end_version,
                );

                // Update the local stream notification tracker
                self.next_stream_version = request.end_version.checked_add(1).ok_or_else(|| {
                    Error::IntegerOverflow("Next stream version has overflown!".into())
                })?;

                // Create a new data notification
                let data_notification = create_data_notification(
                    notification_id_generator,
                    client_response,
                    self.clone().into(),
                );
                return Ok(Some(data_notification));
            }
            client_request => {
                invalid_client_request(client_request, self.clone().into());
            }
        }
        Ok(None)
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
    if start_index > end_index {
        // TODO(joshlind): log this occurrence! We need to handle stream completion.
        return Ok(vec![]);
    }

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
        StreamProgressTracker::AccountsStreamTracker(stream_tracker) => {
            DataClientRequest::AccountsWithProof(AccountsWithProofRequest {
                version: stream_tracker.request.version,
                start_index,
                end_index,
            })
        }
        StreamProgressTracker::ContinuousTransactionStreamTracker(stream_tracker) => {
            DataClientRequest::TransactionsWithProof(TransactionsWithProofRequest {
                start_version: start_index,
                end_version: end_index,
                max_proof_version: stream_tracker
                    .get_target_ledger_info()
                    .ledger_info()
                    .version(),
                include_events: stream_tracker.request.include_events,
            })
        }
        StreamProgressTracker::EpochEndingStreamTracker(_) => {
            DataClientRequest::EpochEndingLedgerInfos(EpochEndingLedgerInfosRequest {
                start_epoch: start_index,
                end_epoch: end_index,
            })
        }
        StreamProgressTracker::TransactionOutputStreamTracker(stream_tracker) => {
            DataClientRequest::TransactionOutputsWithProof(TransactionOutputsWithProofRequest {
                start_version: start_index,
                end_version: end_index,
                max_proof_version: stream_tracker.request.max_proof_version,
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

/// Returns the highest synced ledger info advertised in the network
fn highest_synced_ledger_info(
    advertised_data: &AdvertisedData,
) -> Option<LedgerInfoWithSignatures> {
    let highest_synced_position = advertised_data
        .synced_ledger_infos
        .iter()
        .map(|ledger_info_with_sigs| ledger_info_with_sigs.ledger_info().version())
        .position_max();

    if let Some(highest_synced_position) = highest_synced_position {
        advertised_data
            .synced_ledger_infos
            .get(highest_synced_position)
            .cloned()
    } else {
        None
    }
}

/// Creates a new data notification for the given client response.
fn create_data_notification(
    notification_id_generator: Arc<AtomicU64>,
    client_response: &DataClientResponse,
    stream_progress_tracker: StreamProgressTracker,
) -> DataNotification {
    let notification_id = notification_id_generator.fetch_add(1, Ordering::Relaxed);

    let data_payload = match &client_response.response_payload {
        DataClientPayload::AccountStatesWithProof(accounts_chunk) => {
            DataPayload::AccountStatesWithProof(accounts_chunk.clone())
        }
        DataClientPayload::EpochEndingLedgerInfos(ledger_infos) => {
            DataPayload::EpochEndingLedgerInfos(ledger_infos.clone())
        }
        DataClientPayload::TransactionsWithProof(transactions_chunk) => {
            match stream_progress_tracker {
                StreamProgressTracker::ContinuousTransactionStreamTracker(stream_tracker) => {
                    DataPayload::ContinuousTransactionsWithProof(
                        stream_tracker.get_target_ledger_info().clone(),
                        transactions_chunk.clone(),
                    )
                }
                StreamProgressTracker::TransactionStreamTracker(_) => {
                    DataPayload::TransactionsWithProof(transactions_chunk.clone())
                }
                _ => {
                    panic!(
                        "The client response is type mismatched: {:?}",
                        client_response
                    );
                }
            }
        }
        DataClientPayload::TransactionOutputsWithProof(transactions_output_chunk) => {
            DataPayload::TransactionOutputsWithProof(transactions_output_chunk.clone())
        }
        _ => {
            panic!(
                "The client response is type mismatched: {:?}",
                client_response
            );
        }
    };

    DataNotification {
        notification_id,
        data_payload,
    }
}
