// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::AccountData;
use anyhow::{bail, ensure, format_err, Error, Result};
use libra_json_rpc::{
    errors::JsonRpcError,
    views::{
        AccountStateWithProofView, AccountView, BytesView, EventView, StateProofView,
        TransactionView,
    },
};
use libra_logger::prelude::*;
use libra_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_config::{ACCOUNT_RECEIVED_EVENT_PATH, ACCOUNT_SENT_EVENT_PATH},
    account_state_blob::AccountStateBlob,
    ledger_info::LedgerInfoWithSignatures,
    transaction::{SignedTransaction, Version},
    trusted_state::{TrustedState, TrustedStateChange},
    validator_change::ValidatorChangeProof,
    vm_error::StatusCode,
    waypoint::Waypoint,
};
use rand::Rng;
use reqwest::blocking::Client;
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
    addr: String,
    client: Client,
}

impl JsonRpcClient {
    pub fn new(host: &str, port: u16) -> Self {
        let addr = format!("http://{}:{}", host, port);
        let client = Client::new();

        Self { client, addr }
    }

    /// Sends JSON request `request`, performs basic checks on the payload, and returns Ok(`result`),
    /// where `result` is the payload under the key "result" in the JSON RPC response
    /// If there is an error payload in the JSON RPC response, throw an Err with message describing the error
    /// payload
    pub fn send_libra_request(
        &mut self,
        method: String,
        params: Vec<serde_json::Value>,
    ) -> Result<serde_json::Value> {
        let id: u64 = rand::thread_rng().gen();
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": id,
        });

        let response = self
            .send_with_retry(request)?
            .error_for_status()
            .map_err(|e| format_err!("Server returned error: {:?}", e))?;

        // check payload
        let data: serde_json::Value = response.json()?;
        let (response_id, result) = Self::process_single_response(data)?;

        // check ID
        ensure!(
            response_id == id,
            "JSON RPC response ID {} does not match request ID {}",
            response_id,
            id
        );

        Ok(result)
    }

    /// processes a single JSON RPC response
    fn process_single_response(response: serde_json::Value) -> Result<(u64, serde_json::Value)> {
        // check JSON RPC protocol
        let json_rpc_protocol = response.get("jsonrpc");
        ensure!(
            json_rpc_protocol == Some(&serde_json::Value::String("2.0".to_string())),
            "JSON RPC response with incorrect protocol: {:?}",
            json_rpc_protocol
        );

        if let Some(error) = response.get("error") {
            let json_error: JsonRpcError = serde_json::from_value(error.clone())?;
            return Err(Error::new(json_error));
        }

        if let Some(result) = response.get("result") {
            let response_id: u64 = serde_json::from_value(response.get("id").unwrap().clone())?;
            Ok((response_id, result.clone()))
        } else {
            bail!("Received JSON RPC response with no result payload");
        }
    }

    /// Sends a batch JSON RPC request.
    /// Returns a vector of responses s.t. response order matches the request order
    /// Throws error if any of the responses are not valid JSON RPC responses or any of the valid JSON RPC responses
    /// contain an `error` payload
    pub fn send_libra_batch_request(
        &mut self,
        batch_requests: Vec<(String, serde_json::Value)>,
    ) -> Result<Vec<serde_json::Value>> {
        if batch_requests.is_empty() {
            return Ok(vec![]);
        }

        let mut req_id: u64 = 0;
        // convert requests to JSON
        let request = batch_requests
            .into_iter()
            .map(|(method, params)| {
                req_id += 1;
                let request = serde_json::json!({
                    "jsonrpc": "2.0",
                    "method": method,
                    "params": params,
                    "id": req_id,
                });
                request
            })
            .collect();

        //retry send
        let response = self
            .send_with_retry(serde_json::Value::Array(request))?
            .error_for_status()
            .map_err(|e| format_err!("Server returned error: {:?}", e))?;

        // basic process payload
        let mut processed_responses = vec![];
        match response.json()? {
            serde_json::Value::Array(responses) => {
                for response in responses {
                    let (id, result) = Self::process_single_response(response)?;
                    // check ID
                    ensure!(
                        0 < id && id <= req_id,
                        "Unexpected JSON RPC response ID {:?}",
                        id
                    );

                    processed_responses.push((id, result));
                }
            }
            _ => bail!("invalid JSON RPC server response format"),
        }

        // sort response by ID
        processed_responses.sort_by_key(|(idx, _result)| *idx);
        Ok(processed_responses
            .into_iter()
            .map(|(_idx, result)| result)
            .collect())
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
            .post(&self.addr)
            .json(request)
            .timeout(Duration::from_millis(JSON_RPC_TIMEOUT_MS))
            .send()
            .map_err(Into::into)
    }
}

impl LibraClient {
    /// Construct a new Client instance.
    // TODO(philiphayes/dmitrip): Waypoint should not be optional
    pub fn new(host: &str, port: u16, waypoint: Option<Waypoint>) -> Result<Self> {
        // If waypoint is present, use it for initial verification, otherwise the initial
        // verification is essentially empty.
        let initial_trusted_state = match waypoint {
            Some(waypoint) => TrustedState::from_waypoint(waypoint),
            None => TrustedState::new_trust_any_genesis_WARNING_UNSAFE(),
        };
        let client = JsonRpcClient::new(host, port);
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
        let payload = hex::encode(lcs::to_bytes(&transaction).unwrap());
        let params = vec![serde_json::json!(payload)];

        match self.client.send_libra_request("submit".to_string(), params) {
            Ok(result) => {
                ensure!(
                    result == serde_json::Value::Null,
                    "Received unexpected result payload from txn submission: {:?}",
                    result
                );
                if let Some(sender_account) = sender_account_opt {
                    // Bump up sequence_number if transaction is accepted.
                    sender_account.sequence_number += 1;
                }
                Ok(())
            }
            Err(e) => {
                if let Some(error) = e.downcast_ref::<JsonRpcError>() {
                    // check VM status
                    if let Some(vm_error) = error.get_vm_error() {
                        if vm_error.major_status == StatusCode::SEQUENCE_NUMBER_TOO_OLD {
                            if let Some(sender_account) = sender_account_opt {
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

    // Retrieves and
    // - If `with_state_change`, will also retrieve state proof from node and update trusted_state accordingly
    pub fn get_account_state(
        &mut self,
        account: AccountAddress,
        with_state_proof: bool,
    ) -> Result<(Option<AccountView>, Version)> {
        let method = "get_account_state".to_string();
        let params = vec![serde_json::json!(account)];

        let response = if with_state_proof {
            self.get_with_state_proof(method, serde_json::json!(params))
        } else {
            self.client.send_libra_request(method, params)
        };

        match response {
            Ok(result) => {
                let account_view = match result {
                    serde_json::Value::Null => None,
                    _ => {
                        let account_view: AccountView = serde_json::from_value(result)?;
                        Some(account_view)
                    }
                };
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
        let params = vec![
            serde_json::json!(event_key),
            serde_json::json!(start),
            serde_json::json!(limit),
        ];

        match self
            .client
            .send_libra_request("get_events".to_string(), params)
        {
            Ok(result) => {
                let events: Vec<EventView> = serde_json::from_value(result)?;
                Ok(events)
            }
            Err(e) => bail!("Failed to get events with error: {:?}", e),
        }
    }

    pub fn get_state_proof(&mut self) -> Result<LedgerInfoWithSignatures> {
        let client_version = self.trusted_state.latest_version();
        match self.client.send_libra_request(
            "get_state_proof".to_string(),
            vec![serde_json::json!(client_version)],
        ) {
            Ok(response) => {
                let proof: StateProofView = serde_json::from_value(response)?;
                let li: LedgerInfoWithSignatures =
                    lcs::from_bytes(&proof.ledger_info_with_signatures.clone().into_bytes()?)?;
                self.verify_state_proof(proof)?;
                Ok(li)
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
                self.trusted_state = new_state;
            }
        }
        Ok(())
    }

    fn get_with_state_proof(
        &mut self,
        method: String,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let client_version = self.trusted_state.latest_version();

        let batch_requests = vec![
            (method, params),
            (
                "get_state_proof".to_string(),
                serde_json::json!(vec![client_version]),
            ),
        ];

        // batch send
        match self.client.send_libra_batch_request(batch_requests) {
            Err(e) => bail!("Failed to get with state proof, error: {:?}", e),
            Ok(responses) => {
                // verify and update trusted state
                let proof: StateProofView = serde_json::from_value(responses[1].clone())?;
                self.verify_state_proof(proof)?;
                // verify sub-response - responsibility of each call to subresponse
                Ok(responses[0].clone())
            }
        }
    }

    /// LedgerInfo corresponding to the latest epoch change.
    pub(crate) fn latest_epoch_change_li(&self) -> Option<&LedgerInfoWithSignatures> {
        self.latest_epoch_change_li.as_ref()
    }

    /// Get the latest account state blob from validator.
    pub(crate) fn get_account_blob(
        &mut self,
        address: AccountAddress,
    ) -> Result<Option<AccountStateBlob>> {
        let version = serde_json::json!(self.trusted_state.latest_version());
        let params = vec![serde_json::json!(address), version.clone(), version];

        match self
            .client
            .send_libra_request("get_account_state_with_proof".to_string(), params)
        {
            Ok(result) => {
                let account_state_with_proof: AccountStateWithProofView =
                    serde_json::from_value(result)?;
                let account_blob = if let Some(blob) = account_state_with_proof.blob {
                    let account_blob: AccountStateBlob = lcs::from_bytes(&blob.into_bytes()?)?;
                    Some(account_blob)
                } else {
                    None
                };
                Ok(account_blob)
            }
            Err(e) => bail!("Failed to get account state blob with error: {:?}", e),
        }
    }

    /// Get transaction from validator by account and sequence number.
    pub fn get_txn_by_acc_seq(
        &mut self,
        account: AccountAddress,
        sequence_number: u64,
        fetch_events: bool,
    ) -> Result<Option<TransactionView>> {
        let params = vec![
            serde_json::json!(account),
            serde_json::json!(sequence_number),
            serde_json::json!(fetch_events),
        ];
        match self.get_with_state_proof(
            "get_account_transaction".to_string(),
            serde_json::json!(params),
        ) {
            Ok(result) => {
                let txn = if result == serde_json::Value::Null {
                    None
                } else {
                    let txn_view: TransactionView = serde_json::from_value(result)?;
                    Some(txn_view)
                };
                Ok(txn)
            }
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
        let params = vec![
            serde_json::json!(start_version),
            serde_json::json!(limit),
            serde_json::json!(fetch_events),
        ];

        match self.get_with_state_proof("get_transactions".to_string(), serde_json::json!(params)) {
            Ok(result) => {
                let txns: Vec<TransactionView> = serde_json::from_value(result)?;
                Ok(txns)
            }
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
