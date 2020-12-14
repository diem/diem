// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, ensure, Result};
use diem_json_rpc_client::async_client::{
    types as jsonrpc, Client, Retry, WaitForTransactionError,
};
use diem_logger::prelude::info;
use diem_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_config::{ACCOUNT_RECEIVED_EVENT_PATH, ACCOUNT_SENT_EVENT_PATH},
    account_state_blob::AccountStateBlob,
    epoch_change::EpochChangeProof,
    ledger_info::LedgerInfoWithSignatures,
    transaction::{SignedTransaction, Version},
    trusted_state::{TrustedState, TrustedStateChange},
    waypoint::Waypoint,
};
use reqwest::Url;
use std::time::Duration;

/// A client connection to an AdmissionControl (AC) service. `DiemClient` also
/// handles verifying the server's responses, retrying on non-fatal failures, and
/// ratcheting our latest verified state, which includes the latest verified
/// version and latest verified epoch change ledger info.
///
/// ### Note
///
/// `DiemClient` will reject out-of-date responses. For example, this can happen if
///
/// 1. We make a request to the remote AC service.
/// 2. The remote service crashes and it forgets the most recent state or an
///    out-of-date replica takes its place.
/// 3. We make another request to the remote AC service. In this case, the remote
///    AC will be behind us and we will reject their response as stale.
pub struct DiemClient {
    client: Client<Retry>,
    /// The latest verified chain state.
    trusted_state: TrustedState,
    /// The most recent epoch change ledger info. This is `None` if we only know
    /// about our local [`Waypoint`] and have not yet ratcheted to the remote's
    /// latest state.
    latest_epoch_change_li: Option<LedgerInfoWithSignatures>,
    runtime: diem_infallible::Mutex<tokio::runtime::Runtime>,
}

impl DiemClient {
    /// Construct a new Client instance.
    pub fn new(url: Url, waypoint: Waypoint) -> Result<Self> {
        let initial_trusted_state = TrustedState::from(waypoint);
        let client = Client::from_url(url, Retry::default())?;

        Ok(DiemClient {
            runtime: diem_infallible::Mutex::new(
                tokio::runtime::Builder::new()
                    .thread_name("cli-client")
                    .threaded_scheduler()
                    .enable_all()
                    .build()
                    .expect("failed to create runtime"),
            ),
            client,
            trusted_state: initial_trusted_state,
            latest_epoch_change_li: None,
        })
    }

    /// Submits a transaction and bumps the sequence number for the sender, pass in `None` for
    /// sender_account if sender's address is not managed by the client.
    pub fn submit_transaction(&self, transaction: &SignedTransaction) -> Result<()> {
        self.runtime
            .lock()
            .block_on(self.client.submit(transaction))
            .map_err(anyhow::Error::new)
            .map(|r| r.result)
    }

    /// Retrieves account information
    /// - If `with_state_proof`, will also retrieve state proof from node and update trusted_state accordingly
    pub fn get_account(&self, account: &AccountAddress) -> Result<Option<jsonrpc::Account>> {
        self.runtime
            .lock()
            .block_on(self.client.get_account(account))
            .map_err(anyhow::Error::new)
            .map(|r| r.result)
    }

    pub fn get_account_state_blob(
        &self,
        account: &AccountAddress,
    ) -> Result<(Option<AccountStateBlob>, Version)> {
        let ret = self
            .runtime
            .lock()
            .block_on(
                self.client
                    .get_account_state_with_proof(account, None, None),
            )
            .map(|r| r.result)
            .map_err(anyhow::Error::new)?;
        if !ret.blob.is_empty() {
            Ok((Some(bcs::from_bytes(&hex::decode(ret.blob)?)?), ret.version))
        } else {
            Ok((None, ret.version))
        }
    }

    pub fn get_events(
        &self,
        event_key: &str,
        start: u64,
        limit: u64,
    ) -> Result<Vec<jsonrpc::Event>> {
        self.runtime
            .lock()
            .block_on(self.client.get_events(event_key, start, limit))
            .map(|r| r.result)
            .map_err(anyhow::Error::new)
    }

    pub fn wait_for_transaction(
        &self,
        txn: &SignedTransaction,
        timeout: Duration,
    ) -> Result<jsonrpc::Transaction, WaitForTransactionError> {
        self.runtime
            .lock()
            .block_on(
                self.client
                    .wait_for_signed_transaction(txn, Some(timeout), None),
            )
            .map(|r| r.result)
    }

    /// Gets the block metadata
    pub fn get_metadata(&self) -> Result<jsonrpc::Metadata> {
        self.runtime
            .lock()
            .block_on(self.client.get_metadata())
            .map(|r| r.result)
            .map_err(anyhow::Error::new)
    }

    /// Gets the currency info stored on-chain
    pub fn get_currency_info(&self) -> Result<Vec<jsonrpc::CurrencyInfo>> {
        self.runtime
            .lock()
            .block_on(self.client.get_currencies())
            .map(|r| r.result)
            .map_err(anyhow::Error::new)
    }

    /// Retrieves and checks the state proof
    pub fn update_and_verify_state_proof(&mut self) -> Result<()> {
        let state_proof = self
            .runtime
            .lock()
            .block_on(
                self.client
                    .get_state_proof(self.trusted_state().latest_version()),
            )
            .map(|r| r.result)
            .map_err(anyhow::Error::new)?;
        self.verify_state_proof(state_proof)
    }

    fn verify_state_proof(&mut self, state_proof: jsonrpc::StateProof) -> Result<()> {
        let state = self.trusted_state();

        let li: LedgerInfoWithSignatures =
            bcs::from_bytes(&hex::decode(state_proof.ledger_info_with_signatures)?)?;
        let epoch_change_proof: EpochChangeProof =
            bcs::from_bytes(&hex::decode(state_proof.epoch_change_proof)?)?;

        // check ledger info version
        ensure!(
            li.ledger_info().version() >= state.latest_version(),
            "Got stale ledger_info with version {}, known version: {}",
            li.ledger_info().version(),
            state.latest_version(),
        );

        // trusted_state_change
        match state.verify_and_ratchet(&li, &epoch_change_proof)? {
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
                self.update_trusted_state(new_state);
                self.update_latest_epoch_change_li(latest_epoch_change_li.clone());
            }
            TrustedStateChange::Version { new_state } => {
                if state.latest_version() < new_state.latest_version() {
                    info!("Verified version change to: {}", new_state.latest_version());
                }
                self.update_trusted_state(new_state);
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

    fn update_latest_epoch_change_li(&mut self, ledger: LedgerInfoWithSignatures) {
        self.latest_epoch_change_li = Some(ledger);
    }

    fn update_trusted_state(&mut self, state: TrustedState) {
        self.trusted_state = state
    }

    /// Get transaction from validator by account and sequence number.
    pub fn get_txn_by_acc_seq(
        &self,
        account: &AccountAddress,
        sequence_number: u64,
        fetch_events: bool,
    ) -> Result<Option<jsonrpc::Transaction>> {
        self.runtime
            .lock()
            .block_on(
                self.client
                    .get_account_transaction(&account, sequence_number, fetch_events),
            )
            .map(|r| r.result)
            .map_err(anyhow::Error::new)
    }

    /// Get transactions in range (start_version..start_version + limit - 1) from validator.
    pub fn get_txn_by_range(
        &self,
        start_version: u64,
        limit: u64,
        fetch_events: bool,
    ) -> Result<Vec<jsonrpc::Transaction>> {
        self.runtime
            .lock()
            .block_on(
                self.client
                    .get_transactions(start_version, limit, fetch_events),
            )
            .map(|r| r.result)
            .map_err(anyhow::Error::new)
    }

    pub fn get_events_by_access_path(
        &self,
        access_path: AccessPath,
        start_event_seq_num: u64,
        limit: u64,
    ) -> Result<(Vec<jsonrpc::Event>, jsonrpc::Account)> {
        // get event key from access_path
        match self.get_account(&access_path.address)? {
            None => bail!("No account found for address {:?}", access_path.address),
            Some(account_view) => {
                let path = access_path.path;
                let event_key = if path == ACCOUNT_SENT_EVENT_PATH.to_vec() {
                    &account_view.sent_events_key
                } else if path == ACCOUNT_RECEIVED_EVENT_PATH.to_vec() {
                    &account_view.received_events_key
                } else {
                    bail!("Unexpected event path found in access path");
                };

                // get_events
                let events = self.get_events(event_key, start_event_seq_num, limit)?;
                Ok((events, account_view))
            }
        }
    }
}
