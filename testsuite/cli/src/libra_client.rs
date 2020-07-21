// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::AccountData;
use anyhow::{bail, ensure, Result};
use libra_json_rpc_client::{
    errors::JsonRpcError,
    get_response_from_batch,
    views::{
        AccountStateWithProofView, AccountView, BlockMetadata, BytesView, CurrencyInfoView,
        EventView, StateProofView, TransactionView,
    },
    JsonRpcBatch, JsonRpcClient, JsonRpcResponse, ResponseAsView,
};
use libra_logger::prelude::*;
use libra_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_config::{ACCOUNT_RECEIVED_EVENT_PATH, ACCOUNT_SENT_EVENT_PATH},
    account_state_blob::AccountStateBlob,
    epoch_change::EpochChangeProof,
    ledger_info::LedgerInfoWithSignatures,
    transaction::{SignedTransaction, Version},
    trusted_state::{TrustedState, TrustedStateChange},
    vm_status::StatusCode,
    waypoint::Waypoint,
};
use reqwest::Url;

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

impl LibraClient {
    /// Construct a new Client instance.
    pub fn new(url: Url, waypoint: Waypoint) -> Result<Self> {
        let initial_trusted_state = TrustedState::from(waypoint);
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
                    if let Some(status_code) = error.as_status_code() {
                        if status_code == StatusCode::SEQUENCE_NUMBER_TOO_OLD {
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

    /// Retrieves account information
    /// - If `with_state_proof`, will also retrieve state proof from node and update trusted_state accordingly
    pub fn get_account(
        &mut self,
        account: AccountAddress,
        with_state_proof: bool,
    ) -> Result<(Option<AccountView>, Version)> {
        let client_version = self.trusted_state.latest_version();
        let mut batch = JsonRpcBatch::new();
        batch.add_get_account_request(account);
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
                "Failed to get account for account address {} with error: {:?}",
                account,
                e
            ),
        }
    }

    pub fn get_account_state_blob(
        &mut self,
        account: AccountAddress,
    ) -> Result<(Option<AccountStateBlob>, Version)> {
        let mut batch = JsonRpcBatch::new();
        batch.add_get_account_state_with_proof_request(account, None, None);
        let responses = self.client.execute(batch)?;
        match get_response_from_batch(0, &responses)? {
            Ok(result) => {
                let account_state_with_proof =
                    AccountStateWithProofView::from_response(result.clone())?;
                if let Some(bytes) = account_state_with_proof.blob {
                    Ok((
                        Some(lcs::from_bytes(&bytes.into_bytes()?)?),
                        account_state_with_proof.version,
                    ))
                } else {
                    Ok((None, account_state_with_proof.version))
                }
            }
            Err(e) => bail!(
                "Failed to get account state blob for account address {} with error: {:?}",
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
        batch.add_get_metadata_request(None);
        let responses = self.client.execute(batch)?;

        match get_response_from_batch(0, &responses)? {
            Ok(resp) => Ok(BlockMetadata::from_response(resp.clone())?),
            Err(e) => bail!("Failed to get block metadata with error: {:?}", e),
        }
    }

    /// Gets the currency info stored on-chain
    pub fn get_currency_info(&mut self) -> Result<Vec<CurrencyInfoView>> {
        let mut batch = JsonRpcBatch::new();
        batch.add_get_currencies_info();
        let responses = self.client.execute(batch)?;
        match get_response_from_batch(0, &responses)? {
            Ok(resp) => Ok(CurrencyInfoView::vec_from_response(resp.clone())?),
            Err(e) => bail!("Failed to get currencies info with error: {:?}", e),
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
        let epoch_change_proof: EpochChangeProof =
            lcs::from_bytes(&state_proof.epoch_change_proof.into_bytes()?)?;

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
            .verify_and_ratchet(&li, &epoch_change_proof)?
        {
            TrustedStateChange::Epoch {
                new_state,
                latest_epoch_change_li,
            } => {
                info!(
                    "Verified epoch changed to {}",
                    latest_epoch_change_li
                        .ledger_info()
                        .next_epoch_state()
                        .expect("no validator set in epoch change ledger info"),
                );
                // Update client state
                self.trusted_state = new_state;
                self.latest_epoch_change_li = Some(latest_epoch_change_li.clone());
            }
            TrustedStateChange::Version { new_state } => {
                if self.trusted_state.latest_version() < new_state.latest_version() {
                    info!("Verified version change to: {}", new_state.latest_version());
                }
                self.trusted_state = new_state;
            }
            TrustedStateChange::NoChange => (),
        }
        Ok(())
    }

    /// LedgerInfo corresponding to the latest epoch change.
    pub(crate) fn latest_epoch_change_li(&self) -> Option<&LedgerInfoWithSignatures> {
        self.latest_epoch_change_li.as_ref()
    }

    /// Latest trusted state
    pub(crate) fn trusted_state(&self) -> TrustedState {
        self.trusted_state.clone()
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
        match self.get_account(account, true)?.0 {
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
        match self.get_account(access_path.address, false)?.0 {
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
