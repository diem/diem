// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::AccountData;
use anyhow::{bail, ensure, format_err, Result};
use libra_json_rpc::{
    errors::JsonRpcError,
    get_response_from_batch, process_batch_response,
    views::{
        AccountView, BlockMetadata, BytesView, EventView, ResponseAsView, StateProofView,
        TransactionView,
    },
    JsonRpcBatch, JsonRpcResponse,
};
use libra_logger::prelude::*;
use libra_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_config::{ACCOUNT_RECEIVED_EVENT_PATH, ACCOUNT_SENT_EVENT_PATH},
    ledger_info::LedgerInfoWithSignatures,
    transaction::{SignedTransaction, Version},
    trusted_state::{TrustedState, TrustedStateChange},
    validator_change::ValidatorChangeProof,
    vm_error::StatusCode,
    waypoint::Waypoint,
};
use reqwest::{
    blocking::{Client, ClientBuilder},
    Url,
};
use std::time::Duration;

const JSON_RPC_TIMEOUT_MS: u64 = 5_000;
const MAX_JSON_RPC_RETRY_COUNT: u64 = 2;

/// A client connection to an AdmissionControl (AC) service. `LibraClient` also
/// handles verifying the server's responses, retrying on non-fatal failures, and
/// ratcheting our latest verified state, which includes the latest verified
/// version and latest verified epoch change ledger info.
///
/// ### Note
///
/// `LibraClient` will reject out-of-date responses. For example, this can happen if
///
/// 1. We make a request to the remote AC service.
/// 2. The remote service crashes and it forgets the most recent state or an
///    out-of-date replica takes its place.
/// 3. We make another request to the remote AC service. In this case, the remote
///    AC will be behind us and we will reject their response as stale.
pub struct LibraClient {
    client: JsonRpcClient,
    /// The latest verified chain state.
    trusted_state: TrustedState,
    /// The most recent epoch change ledger info. This is `None` if we only know
    /// about our local [`Waypoint`] and have not yet ratcheted to the remote's
    /// latest state.
    latest_epoch_change_li: Option<LedgerInfoWithSignatures>,
}

pub struct JsonRpcClient {
    url: Url,
    client: Client,
}

impl JsonRpcClient {
    pub fn new(url: Url) -> Result<Self> {
        Ok(Self {
            client: ClientBuilder::new().use_rustls_tls().build()?,
            url,
        })
    }

    /// Sends a JSON RPC batched request.
    /// Returns a vector of responses s.t. response order matches the request order
    pub fn execute(&mut self, batch: JsonRpcBatch) -> Result<Vec<Result<JsonRpcResponse>>> {
        if batch.requests.is_empty() {
            return Ok(vec![]);
        }
        let request = batch.json_request();

        //retry send
        let response = self
            .send_with_retry(request)?
            .error_for_status()
            .map_err(|e| format_err!("Server returned error: {:?}", e))?;

        let response = process_batch_response(batch.clone(), response.json()?)?;
        ensure!(
            batch.requests.len() == response.len(),
            "received unexpected number of responses in batch"
        );
        Ok(response)
    }

    // send with retry
    pub fn send_with_retry(
        &mut self,
        request: serde_json::Value,
    ) -> Result<reqwest::blocking::Response> {
        let mut response = self.send(&request);
        let mut try_cnt = 0;

        // retry if send fails
        while try_cnt < MAX_JSON_RPC_RETRY_COUNT && response.is_err() {
            response = self.send(&request);
            try_cnt += 1;
        }
        response
    }

    fn send(&mut self, request: &serde_json::Value) -> Result<reqwest::blocking::Response> {
        self.client
            .post(self.url.clone())
            .json(request)
            .timeout(Duration::from_millis(JSON_RPC_TIMEOUT_MS))
            .send()
            .map_err(Into::into)
    }
}

impl LibraClient {
    /// Construct a new Client instance.
    // TODO(philiphayes/dmitrip): Waypoint should not be optional
    pub fn new(url: Url, waypoint: Option<Waypoint>) -> Result<Self> {
        // If waypoint is present, use it for initial verification, otherwise the initial
        // verification is essentially empty.
        let initial_trusted_state = match waypoint {
            Some(waypoint) => TrustedState::from_waypoint(waypoint),
            None => TrustedState::new_trust_any_genesis_WARNING_UNSAFE(),
        };
        let client = JsonRpcClient::new(url)?;
        Ok(LibraClient {
            client,
            trusted_state: initial_trusted_state,
            latest_epoch_change_li: None,
        })
    }

    /// Submits a transaction and bumps the sequence number for the sender, pass in `None` for
    /// sender_account if sender's address is not managed by the client.
    pub fn submit_transaction(
        &mut self,
        sender_account_opt: Option<&mut AccountData>,
        transaction: SignedTransaction,
    ) -> Result<()> {
        // form request
        let mut batch = JsonRpcBatch::new();
        batch.add_submit_request(transaction)?;

        let responses = self.client.execute(batch)?;
        match get_response_from_batch(0, &responses)? {
            Ok(response) => {
                if response == &JsonRpcResponse::SubmissionResponse {
                    if let Some(sender_account) = sender_account_opt {
                        // Bump up sequence_number if transaction is accepted.
                        sender_account.sequence_number += 1;
                    }
                    Ok(())
                } else {
                    bail!("Received non-submit response payload: {:?}", response)
                }
            }
            Err(e) => {
                if let Some(error) = e.downcast_ref::<JsonRpcError>() {
                    // check VM status
                    if let Some(vm_error) = error.get_vm_error() {
                        if vm_error.major_status == StatusCode::SEQUENCE_NUMBER_TOO_OLD {
                            if let Some(sender_account) = sender_account_opt {
                                // update sender's sequence number if too old
                                sender_account.sequence_number =
                                    self.get_sequence_number(sender_account.address)?;
                            }
                        }
                    }
                }
                bail!("Transaction submission failed with error: {:?}", e)
            }
        }
    }

    /// Retrieves account state
    /// - If `with_state_proof`, will also retrieve state proof from node and update trusted_state accordingly
    pub fn get_account_state(
        &mut self,
        account: AccountAddress,
        with_state_proof: bool,
    ) -> Result<(Option<AccountView>, Version)> {
        let client_version = self.trusted_state.latest_version();
        let mut batch = JsonRpcBatch::new();
        batch.add_get_account_state_request(account);
        if with_state_proof {
            batch.add_get_state_proof_request(client_version);
        }
        let responses = self.client.execute(batch)?;

        if with_state_proof {
            let state_proof = get_response_from_batch(1, &responses)?.as_ref();
            self.process_state_proof_response(state_proof)?;
        }

        match get_response_from_batch(0, &responses)? {
            Ok(result) => {
                let account_view = AccountView::optional_from_response(result.clone())?;
                Ok((account_view, self.trusted_state.latest_version()))
            }
            Err(e) => bail!(
                "Failed to get account state for account address {} with error: {:?}",
                account,
                e
            ),
        }
    }

    pub fn get_events(
        &mut self,
        event_key: String,
        start: u64,
        limit: u64,
    ) -> Result<Vec<EventView>> {
        let mut batch = JsonRpcBatch::new();
        batch.add_get_events_request(event_key, start, limit);
        let responses = self.client.execute(batch)?;

        match get_response_from_batch(0, &responses)? {
            Ok(resp) => Ok(EventView::vec_from_response(resp.clone())?),
            Err(e) => bail!("Failed to get events with error: {:?}", e),
        }
    }

    /// Gets the block metadata
    pub fn get_metadata(&mut self) -> Result<BlockMetadata> {
        let mut batch = JsonRpcBatch::new();
        batch.add_get_metadata_request();
        let responses = self.client.execute(batch)?;

        match get_response_from_batch(0, &responses)? {
            Ok(resp) => Ok(BlockMetadata::from_response(resp.clone())?),
            Err(e) => bail!("Failed to get block metadata with error: {:?}", e),
        }
    }

    /// Retrieves and checks the state proof
    pub fn get_state_proof(&mut self) -> Result<()> {
        let mut batch = JsonRpcBatch::new();
        batch.add_get_state_proof_request(self.trusted_state.latest_version());
        let responses = self.client.execute(batch)?;

        let state_proof = get_response_from_batch(0, &responses)?.as_ref();
        self.process_state_proof_response(state_proof)
    }

    fn process_state_proof_response(
        &mut self,
        response: Result<&JsonRpcResponse, &anyhow::Error>,
    ) -> Result<()> {
        match response {
            Ok(resp) => {
                let state_proof = StateProofView::from_response(resp.clone())?;
                self.verify_state_proof(state_proof)
            }
            Err(e) => bail!("Failed to get state proof with error: {:?}", e),
        }
    }

    fn verify_state_proof(&mut self, state_proof: StateProofView) -> Result<()> {
        let client_version = self.trusted_state.latest_version();
        let li: LedgerInfoWithSignatures =
            lcs::from_bytes(&state_proof.ledger_info_with_signatures.into_bytes()?)?;
        let validator_change_proof: ValidatorChangeProof =
            lcs::from_bytes(&state_proof.validator_change_proof.into_bytes()?)?;

        // check ledger info version
        ensure!(
            li.ledger_info().version() >= client_version,
            "Got stale ledger_info with version {}, known version: {}",
            li.ledger_info().version(),
            client_version,
        );

        // trusted_state_change
        match self
            .trusted_state
            .verify_and_ratchet(&li, &validator_change_proof)?
        {
            TrustedStateChange::Epoch {
                new_state,
                latest_epoch_change_li,
                latest_validator_set,
                ..
            } => {
                info!(
                    "Verified epoch change to epoch: {}, validator set: [{}]",
                    latest_epoch_change_li.ledger_info().epoch(),
                    latest_validator_set
                );
                // Update client state
                self.trusted_state = new_state;
                self.latest_epoch_change_li = Some(latest_epoch_change_li.clone());
            }
            TrustedStateChange::Version { new_state, .. } => {
                if self.trusted_state.latest_version() < new_state.latest_version() {
                    info!("Verified version change to: {}", new_state.latest_version());
                }
                self.trusted_state = new_state;
            }
        }
        Ok(())
    }

    /// LedgerInfo corresponding to the latest epoch change.
    pub(crate) fn latest_epoch_change_li(&self) -> Option<&LedgerInfoWithSignatures> {
        self.latest_epoch_change_li.as_ref()
    }

    /// Get transaction from validator by account and sequence number.
    pub fn get_txn_by_acc_seq(
        &mut self,
        account: AccountAddress,
        sequence_number: u64,
        fetch_events: bool,
    ) -> Result<Option<TransactionView>> {
        let mut batch = JsonRpcBatch::new();
        batch.add_get_account_transaction_request(account, sequence_number, fetch_events);
        batch.add_get_state_proof_request(self.trusted_state.latest_version());

        let responses = self.client.execute(batch)?;
        let state_proof_view = get_response_from_batch(1, &responses)?.as_ref();
        self.process_state_proof_response(state_proof_view)?;

        match get_response_from_batch(0, &responses)? {
            Ok(response) => Ok(TransactionView::optional_from_response(response.clone())?),
            Err(e) => bail!("Failed to get account txn with error: {:?}", e),
        }
    }

    /// Get transactions in range (start_version..start_version + limit - 1) from validator.
    pub fn get_txn_by_range(
        &mut self,
        start_version: u64,
        limit: u64,
        fetch_events: bool,
    ) -> Result<Vec<TransactionView>> {
        let mut batch = JsonRpcBatch::new();
        batch.add_get_transactions_request(start_version, limit, fetch_events);
        batch.add_get_state_proof_request(self.trusted_state.latest_version());

        let responses = self.client.execute(batch)?;
        let state_proof = get_response_from_batch(1, &responses)?.as_ref();
        self.process_state_proof_response(state_proof)?;

        match get_response_from_batch(0, &responses)? {
            Ok(result) => Ok(TransactionView::vec_from_response(result.clone())?),
            Err(e) => bail!("Failed to get transactions with error: {:?}", e),
        }
    }

    fn get_sequence_number(&mut self, account: AccountAddress) -> Result<u64> {
        match self.get_account_state(account, true)?.0 {
            None => bail!("No account found for address {:?}", account),
            Some(account_view) => Ok(account_view.sequence_number),
        }
    }

    pub fn get_events_by_access_path(
        &mut self,
        access_path: AccessPath,
        start_event_seq_num: u64,
        limit: u64,
    ) -> Result<(Vec<EventView>, AccountView)> {
        // get event key from access_path
        match self.get_account_state(access_path.address, false)?.0 {
            None => bail!("No account found for address {:?}", access_path.address),
            Some(account_view) => {
                let path = access_path.path;
                let event_key = if path == ACCOUNT_SENT_EVENT_PATH.to_vec() {
                    let BytesView(sent_events_key) = &account_view.sent_events_key;
                    sent_events_key
                } else if path == ACCOUNT_RECEIVED_EVENT_PATH.to_vec() {
                    let BytesView(received_events_key) = &account_view.received_events_key;
                    received_events_key
                } else {
                    bail!("Unexpected event path found in access path");
                };

                // get_events
                let events = self.get_events(event_key.to_string(), start_event_seq_num, limit)?;
                Ok((events, account_view))
            }
        }
    }
}
