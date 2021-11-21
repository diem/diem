// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    diemnet::state::{ErrorType, PeerStates},
    DiemDataClient, Error, GlobalDataSummary, Response, ResponseCallback, ResponseContext,
    ResponseError, ResponseId, Result,
};
use async_trait::async_trait;
use diem_config::{config::StorageServiceConfig, network_id::PeerNetworkId};
use diem_id_generator::{IdGenerator, U64IdGenerator};
use diem_infallible::RwLock;
use diem_logger::*;
use diem_time_service::{TimeService, TimeServiceTrait};
use diem_types::{
    account_state_blob::AccountStatesChunkWithProof,
    epoch_change::EpochChangeProof,
    ledger_info::LedgerInfoWithSignatures,
    transaction::{
        default_protocol::{TransactionListWithProof, TransactionOutputListWithProof},
        Version,
    },
};
use futures::StreamExt;
use network::{
    application::interface::NetworkInterface,
    protocols::{rpc::error::RpcError, wire::handshake::v1::ProtocolId},
};
use rand::seq::SliceRandom;
use std::{convert::TryFrom, fmt, sync::Arc, time::Duration};
use storage_service_client::StorageServiceClient;
use storage_service_types::{
    AccountStatesChunkWithProofRequest, Epoch, EpochEndingLedgerInfoRequest, StorageServerSummary,
    StorageServiceRequest, StorageServiceResponse, TransactionOutputsWithProofRequest,
    TransactionsWithProofRequest,
};

mod state;
#[cfg(test)]
mod tests;

// TODO(philiphayes): does this belong in a different crate? I feel like we're
// accumulating a lot of tiny crates though...

// TODO(philiphayes): configuration / pass as argument?
const DEFAULT_TIMEOUT: Duration = Duration::from_millis(10_000);
pub const DATA_SUMMARY_POLL_INTERVAL: Duration = Duration::from_millis(1_000);

/// A [`DiemDataClient`] that fulfills requests from remote peers' Storage Service
/// over DiemNet.
///
/// The `DiemNetDataClient`:
///
/// 1. Sends requests to connected DiemNet peers.
/// 2. Does basic type conversions and error handling on the responses.
/// 3. Routes requests to peers that advertise availability for that data.
/// 4. Maintains peer scores based on each peer's observed quality of service
///    and upper client reports of invalid or malicious data.
/// 5. Selects high quality peers to send each request to.
/// 6. Exposes a condensed data summary of our peers' data advertisements.
///
/// The client currently assumes 1-request => 1-response. Streaming responses
/// are handled at an upper layer.
///
/// The client is expected to be cloneable and usable from many concurrent tasks
/// and/or threads.
#[derive(Clone, Debug)]
pub struct DiemNetDataClient {
    /// The underlying DiemNet storage service client.
    network_client: StorageServiceClient,
    /// All of the data-client specific data we have on each network peer.
    peer_states: Arc<RwLock<PeerStates>>,
    /// A cached, aggregate data summary of all unbanned peers' data summaries.
    global_summary_cache: Arc<RwLock<GlobalDataSummary>>,
    /// Used for generating the next request/response id.
    response_id_generator: Arc<U64IdGenerator>,
}

impl DiemNetDataClient {
    pub fn new(
        config: StorageServiceConfig,
        time_service: TimeService,
        network_client: StorageServiceClient,
    ) -> (Self, DataSummaryPoller) {
        let client = Self {
            network_client,
            peer_states: Arc::new(RwLock::new(PeerStates::new(config))),
            global_summary_cache: Arc::new(RwLock::new(GlobalDataSummary::empty())),
            response_id_generator: Arc::new(U64IdGenerator::new()),
        };
        let poller =
            DataSummaryPoller::new(time_service, client.clone(), DATA_SUMMARY_POLL_INTERVAL);
        (client, poller)
    }

    fn next_response_id(&self) -> u64 {
        self.response_id_generator.next()
    }

    /// Update a peer's data summary.
    fn update_summary(&self, peer: PeerNetworkId, summary: StorageServerSummary) {
        self.peer_states.write().update_summary(peer, summary)
    }

    /// Recompute and update the global data summary cache.
    fn update_global_summary_cache(&self) {
        let aggregate = self.peer_states.read().aggregate_summary();
        *self.global_summary_cache.write() = aggregate;
    }

    /// Choose a connected peer that can service the given request. Returns an
    /// error if no such peer can be found.
    fn choose_peer(&self, request: &StorageServiceRequest) -> Result<PeerNetworkId, Error> {
        let all_connected = {
            let network_peer_metadata = self.network_client.peer_metadata_storage();
            network_peer_metadata
                .networks()
                .flat_map(|network_id| {
                    network_peer_metadata
                        .read_filtered(network_id, |(_, peer_metadata)| {
                            peer_metadata.is_connected()
                                && peer_metadata.supports_protocol(ProtocolId::StorageServiceRpc)
                        })
                        .into_keys()
                })
                .collect::<Vec<_>>()
        };

        if all_connected.is_empty() {
            return Err(Error::DataIsUnavailable(
                "no connected diemnet peers".to_owned(),
            ));
        }

        let internal_peer_states = self.peer_states.read();
        let all_serviceable = all_connected
            .into_iter()
            .filter(|peer| internal_peer_states.can_service_request(peer, request))
            .collect::<Vec<_>>();

        all_serviceable
            .choose(&mut rand::thread_rng())
            .copied()
            .ok_or_else(|| {
                Error::DataIsUnavailable(
                    "no connected peers are advertising that they can serve this data range"
                        .to_owned(),
                )
            })
    }

    async fn send_request_and_decode<T, E>(
        &self,
        request: StorageServiceRequest,
    ) -> Result<Response<T>>
    where
        T: TryFrom<StorageServiceResponse, Error = E>,
        E: Into<Error>,
    {
        let peer = self.choose_peer(&request)?;
        self.send_request_to_peer_and_decode(peer, request).await
    }

    async fn send_request_to_peer_and_decode<T, E>(
        &self,
        peer: PeerNetworkId,
        request: StorageServiceRequest,
    ) -> Result<Response<T>>
    where
        T: TryFrom<StorageServiceResponse, Error = E>,
        E: Into<Error>,
    {
        let response = self.send_request_to_peer(peer, request).await?;

        let (context, payload) = response.into_parts();

        // try to convert the storage service enum into the exact variant we're expecting.
        match T::try_from(payload) {
            Ok(new_payload) => Ok(Response::new(context, new_payload)),
            // if the variant doesn't match what we're expecting, report the issue.
            Err(err) => {
                context
                    .response_callback
                    .notify_bad_response(ResponseError::InvalidPayloadDataType);
                Err(err.into())
            }
        }
    }

    async fn send_request_to_peer(
        &self,
        peer: PeerNetworkId,
        request: StorageServiceRequest,
    ) -> Result<Response<StorageServiceResponse>, Error> {
        let id = self.next_response_id();
        let result = self
            .network_client
            .send_request(peer, request.clone(), DEFAULT_TIMEOUT)
            .await;

        match result {
            Ok(response) => {
                // For now, record all responses that at least pass the data
                // client layer successfully. An alternative might also have the
                // consumer notify both success and failure via the callback.
                // On the one hand, scoring dynamics are simpler when each request
                // is successful or failed but not both; on the other hand, this
                // feels simpler for the consumer.
                self.peer_states.write().update_score_success(peer);

                // package up all of the context needed to fully report an error
                // with this RPC.
                let response_callback = DiemNetResponseCallback {
                    data_client: self.clone(),
                    id,
                    peer,
                    request,
                };
                let context = ResponseContext {
                    id,
                    response_callback: Box::new(response_callback),
                };
                Ok(Response::new(context, response))
            }
            Err(err) => {
                // convert network error and storage service error types into
                // data client errors. also categorize the error type for scoring
                // purposes.
                let client_err = match err {
                    storage_service_client::Error::RpcError(err) => match err {
                        RpcError::NotConnected(_) => Error::DataIsUnavailable(err.to_string()),
                        RpcError::TimedOut => Error::TimeoutWaitingForResponse(err.to_string()),
                        _ => Error::UnexpectedErrorEncountered(err.to_string()),
                    },
                    storage_service_client::Error::StorageServiceError(err) => {
                        Error::UnexpectedErrorEncountered(err.to_string())
                    }
                };

                self.notify_bad_response(id, peer, &request, ErrorType::NotUseful);
                Err(client_err)
            }
        }
    }

    fn notify_bad_response(
        &self,
        _id: ResponseId,
        peer: PeerNetworkId,
        _request: &StorageServiceRequest,
        error_type: ErrorType,
    ) {
        self.peer_states
            .write()
            .update_score_error(peer, error_type);
    }
}

#[async_trait]
impl DiemDataClient for DiemNetDataClient {
    fn get_global_data_summary(&self) -> GlobalDataSummary {
        self.global_summary_cache.read().clone()
    }

    async fn get_account_states_with_proof(
        &self,
        version: u64,
        start_account_index: u64,
        end_account_index: u64,
    ) -> Result<Response<AccountStatesChunkWithProof>> {
        let request = StorageServiceRequest::GetAccountStatesChunkWithProof(
            AccountStatesChunkWithProofRequest {
                version,
                start_account_index,
                end_account_index,
            },
        );
        self.send_request_and_decode(request).await
    }

    async fn get_epoch_ending_ledger_infos(
        &self,
        start_epoch: Epoch,
        expected_end_epoch: Epoch,
    ) -> Result<Response<Vec<LedgerInfoWithSignatures>>> {
        let request =
            StorageServiceRequest::GetEpochEndingLedgerInfos(EpochEndingLedgerInfoRequest {
                start_epoch,
                expected_end_epoch,
            });
        let response: Response<EpochChangeProof> = self.send_request_and_decode(request).await?;
        Ok(response.map(|epoch_change| epoch_change.ledger_info_with_sigs))
    }

    async fn get_number_of_account_states(&self, version: Version) -> Result<Response<u64>> {
        let request = StorageServiceRequest::GetNumberOfAccountsAtVersion(version);
        self.send_request_and_decode(request).await
    }

    async fn get_transaction_outputs_with_proof(
        &self,
        proof_version: Version,
        start_version: Version,
        end_version: Version,
    ) -> Result<Response<TransactionOutputListWithProof>> {
        let request = StorageServiceRequest::GetTransactionOutputsWithProof(
            TransactionOutputsWithProofRequest {
                proof_version,
                start_version,
                end_version,
            },
        );
        self.send_request_and_decode(request).await
    }

    async fn get_transactions_with_proof(
        &self,
        proof_version: Version,
        start_version: Version,
        end_version: Version,
        include_events: bool,
    ) -> Result<Response<TransactionListWithProof>> {
        let request =
            StorageServiceRequest::GetTransactionsWithProof(TransactionsWithProofRequest {
                proof_version,
                start_version,
                end_version,
                include_events,
            });
        self.send_request_and_decode(request).await
    }
}

/// The DiemNet-specific request context needed to update a peer's scoring.
struct DiemNetResponseCallback {
    data_client: DiemNetDataClient,
    id: ResponseId,
    peer: PeerNetworkId,
    request: StorageServiceRequest,
}

impl ResponseCallback for DiemNetResponseCallback {
    fn notify_bad_response(&self, error: ResponseError) {
        let error_type = ErrorType::from(error);
        self.data_client
            .notify_bad_response(self.id, self.peer, &self.request, error_type);
    }
}

impl fmt::Debug for DiemNetResponseCallback {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DiemNetResponseCallback")
            .field("data_client", &"..")
            .field("id", &self.id)
            .field("peer", &self.peer)
            .field("request", &self.request)
            .finish()
    }
}

// TODO(philiphayes):
// + ownership b/w poller and data client a bit murky
// + how to stop poller loop? ideally all data client refs get dropped and it
//   just works.
// + would need to make data client contain weak refs somehow when in poller...
// + or maybe poller needs to not depend on data client?
// + an explicit close method seems unsafe / easy to forget...
// + ofc, in prod we will never cancel lol
pub struct DataSummaryPoller {
    time_service: TimeService,
    data_client: DiemNetDataClient,
    poll_interval: Duration,
}

impl DataSummaryPoller {
    fn new(
        time_service: TimeService,
        data_client: DiemNetDataClient,
        poll_interval: Duration,
    ) -> Self {
        Self {
            time_service,
            data_client,
            poll_interval,
        }
    }

    pub async fn start(self) {
        trace!("Starting the data summary poller!");

        let ticker = self.time_service.interval(self.poll_interval);
        futures::pin_mut!(ticker);

        // TODO(philiphayes): rather than polling one at a time, maybe do
        // round-robin with a few concurrent polls.
        loop {
            // wait for next round to poll
            ticker.next().await;

            // just sample a random peer for now. do something smarter here in
            // the future.
            let peer = match self
                .data_client
                .choose_peer(&StorageServiceRequest::GetStorageServerSummary)
            {
                Ok(peer) => peer,
                Err(error) => {
                    debug!("Unable to poll the next peer! Error: {:?}", error);
                    continue;
                }
            };

            debug!("Polling peer ({:?}) to update the data summary!", peer);

            let result: Result<StorageServerSummary> = self
                .data_client
                .send_request_to_peer_and_decode(
                    peer,
                    StorageServiceRequest::GetStorageServerSummary,
                )
                .await
                .map(Response::into_payload);

            let storage_summary = match result {
                Ok(storage_summary) => storage_summary,
                Err(error) => {
                    debug!(
                        "Error encountered when polling peer ({:?})! Error: {:?}",
                        peer, error
                    );
                    continue;
                }
            };

            debug!(
                "Received response from peer ({:?}) when polled: {:?}",
                peer, storage_summary
            );

            self.data_client.update_summary(peer, storage_summary);
            self.data_client.update_global_summary_cache();
        }
    }
}
