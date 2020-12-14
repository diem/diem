// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! The purpose of the JsonRpcClient presented here is to provide a lightweight and secure
//! JSON RPC client to talk to the JSON RPC service offered by Diem Full Nodes. This is useful
//! for various security-critical components (e.g., the secure key manager), as it allows
//! interaction with the Diem blockchain in a minimal and secure manner.
//!
//! Note: While a JSON RPC client implementation already exists in the Diem codebase, this
//! provides a simpler and (hopefully) more secure implementation with fewer dependencies.
#![forbid(unsafe_code)]

use diem_types::{
    account_address::AccountAddress, account_state::AccountState,
    account_state_blob::AccountStateBlob, transaction::SignedTransaction,
};
use hex::FromHexError;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{convert::TryFrom, env, io};
use thiserror::Error;
use ureq::Response;

/// Various constants for the JSON RPC client implementation
const JSON_RPC_VERSION: &str = "2.0";
const REQUEST_TIMEOUT: u64 = 10_000;

#[derive(Debug, Error, PartialEq)]
pub enum Error {
    #[error("JSON RPC call returned a custom internal error: {0}")]
    InternalRPCError(String),
    #[error("Data does not exist. Missing data: {0}")]
    MissingData(String),
    #[error("JSON RPC call failed with response: {0}")]
    RPCFailure(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Unknown error: {0}")]
    UnknownError(String),
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::UnknownError(format!("{}", error))
    }
}

impl From<bcs::Error> for Error {
    fn from(error: bcs::Error) -> Self {
        Self::SerializationError(format!("{}", error))
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Self::SerializationError(format!("{}", error))
    }
}

impl From<FromHexError> for Error {
    fn from(error: FromHexError) -> Self {
        Self::SerializationError(format!("{}", error))
    }
}

/// Provides a lightweight JsonRpcClient implementation.
#[derive(Clone)]
pub struct JsonRpcClient {
    host: String,
}

impl JsonRpcClient {
    pub fn new(host: String) -> Self {
        Self { host }
    }

    /// Submits a signed transaction to the Diem blockchain. This is done by sending a submit()
    /// request to the JSON RPC server using the given transaction.
    pub fn submit_transaction(&self, signed_transaction: SignedTransaction) -> Result<(), Error> {
        let method = "submit".into();
        let params = vec![Value::String(hex::encode(bcs::to_bytes(
            &signed_transaction,
        )?))];
        let response = self.execute_request(method, params);

        process_submit_transaction_response(response)
    }

    /// Returns the associated AccountState for a specific account at a given version height.
    /// This is done by sending a get_account_state_with_proof() request to the JSON RPC server
    /// for the specified account, optional version height and optional ledger_version height
    /// (for the proof).
    pub fn get_account_state(
        &self,
        account: AccountAddress,
        version: Option<u64>,
    ) -> Result<AccountState, Error> {
        let method = "get_account_state_with_proof".into();
        let params = vec![
            Value::String(account.to_string()),
            json!(version),
            json!(version),
        ];
        let response = self.execute_request(method, params);

        process_account_state_response(response)
    }

    /// Retrieves the status of a transaction on a given account.
    pub fn get_transaction_status(
        &self,
        account: AccountAddress,
        sequence_number: u64,
    ) -> Result<Option<TransactionView>, Error> {
        let method = "get_account_transaction".into();
        let params = vec![
            Value::String(account.to_string()),
            json!(sequence_number),
            json!(false),
        ];
        let response = self.execute_request(method, params);

        process_transaction_status_response(response)
    }

    // Executes the specified request method using the given parameters by contacting the JSON RPC
    // server. If the 'http_proxy' or 'https_proxy' environment variable is set, enable the proxy.
    fn execute_request(&self, method: String, params: Vec<Value>) -> Response {
        let mut request = ureq::post(&self.host)
            .timeout_connect(REQUEST_TIMEOUT)
            .build();

        let scheme = request
            .get_scheme()
            .expect("Unable to get the scheme from the host");
        match scheme.as_str() {
            "http" => {
                if let Ok(proxy) = env::var("http_proxy") {
                    request.set_proxy(
                        ureq::Proxy::new(proxy)
                            .expect("Unable to parse http_proxy environment variable"),
                    );
                };
            }
            "https" => {
                if let Ok(proxy) = env::var("https_proxy") {
                    request.set_proxy(
                        ureq::Proxy::new(proxy)
                            .expect("Unable to parse https_proxy environment variable"),
                    );
                };
            }
            _ => {}
        }

        request.send_json(
            json!({"jsonrpc": JSON_RPC_VERSION, "method": method, "params": params, "id": 0}),
        )
    }
}

/// Processes the response from a submit_transaction() JSON RPC request.
pub fn process_submit_transaction_response(response: Response) -> Result<(), Error> {
    match response.status() {
        200 => {
            let response = response.into_string()?;
            if let Ok(failure_response) = serde_json::from_str::<JSONRpcFailureResponse>(&response)
            {
                Err(Error::InternalRPCError(format!("{:?}", failure_response)))
            } else {
                let _submit_response =
                    serde_json::from_str::<SubmitTransactionResponse>(&response)?;
                Ok(())
            }
        }
        _ => Err(Error::RPCFailure(response.into_string()?)),
    }
}

/// Processes the response from a get_account_state_with_proof() JSON RPC request.
pub fn process_account_state_response(response: Response) -> Result<AccountState, Error> {
    match response.status() {
        200 => {
            let response = &response.into_string()?;
            if let Ok(failure_response) = serde_json::from_str::<JSONRpcFailureResponse>(&response)
            {
                Err(Error::InternalRPCError(format!("{:?}", failure_response)))
            } else if let Some(blob_bytes) =
                serde_json::from_str::<AccountStateWithProofResponse>(&response)?
                    .result
                    .blob
            {
                let account_state_blob =
                    AccountStateBlob::from(bcs::from_bytes::<Vec<u8>>(&*blob_bytes.into_bytes()?)?);
                if let Ok(account_state) = AccountState::try_from(&account_state_blob) {
                    Ok(account_state)
                } else {
                    Err(Error::SerializationError(format!(
                        "Unable to convert account_state_blob to AccountState: {:?}",
                        account_state_blob
                    )))
                }
            } else {
                Err(Error::MissingData("AccountState".into()))
            }
        }
        _ => Err(Error::RPCFailure(response.into_string()?)),
    }
}

/// Processes the response from a get_transaction_status() JSON RPC request.
pub fn process_transaction_status_response(
    response: Response,
) -> Result<Option<TransactionView>, Error> {
    match response.status() {
        200 => {
            let response = response.into_string()?;
            if let Ok(response) = serde_json::from_str::<JSONRpcFailureResponse>(&response) {
                return Err(Error::InternalRPCError(format!("{:?}", response)));
            }

            let view = serde_json::from_str::<TransactionViewResponse>(&response)?;
            Ok(view.result)
        }
        _ => Err(Error::RPCFailure(response.into_string()?)),
    }
}

/// Below is a sample response from a successful submit() JSON RPC call:
/// "{
///   "id": 0,
///   "jsonrpc": "2.0",
///   "result": null
/// }"
///
///
#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct SubmitTransactionResponse {
    id: u64,
    jsonrpc: String,
    result: Option<Value>,
}

/// Below is a sample response from a successful get_account_state_with_proof_call() JSON RPC call.
/// "{
///   "id": 0,
///   "jsonrpc": "2.0",
///   "result": {
///     "blob": "0100...",
///     "proof": {
///       "ledger_info_to_transaction_info_proof": "00..",
///       "transaction_info": "200000000000000000...<truncated>...ffffffffffffffff",
///       "transaction_info_to_account_proof": "0000..."
///     },
///     "version": 0
///   }
/// }"
#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct AccountStateWithProofResponse {
    id: u64,
    jsonrpc: String,
    result: AccountStateResponse,
}

/// In practice this represents an AccountStateWithProof, however, we only decode the relevant
/// fields here.
#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct AccountStateResponse {
    version: u64,
    blob: Option<Bytes>,
}

/// Below is a sample response from a successful get_account_state_with_proof_call() JSON RPC call.
/// "{
///   "id": 0,
///   "jsonrpc": "2.0",
///   "result": ...
/// }"
#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct TransactionViewResponse {
    id: u64,
    jsonrpc: String,
    result: Option<TransactionView>,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct TransactionView {
    pub vm_status: VMStatusView,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum VMStatusView {
    Executed,
    OutOfGas,
    MoveAbort {
        location: String,
        abort_code: u64,
    },
    ExecutionFailure {
        location: String,
        function_index: u16,
        code_offset: u16,
    },
    VerificationError,
    DeserializationError,
    PublishingFailure,
}

impl std::fmt::Display for VMStatusView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VMStatusView::Executed => write!(f, "Executed"),
            VMStatusView::OutOfGas => write!(f, "Out of Gas"),
            VMStatusView::MoveAbort {
                location,
                abort_code,
            } => write!(f, "Move Abort: {} at {}", abort_code, location),
            VMStatusView::ExecutionFailure {
                location,
                function_index,
                code_offset,
            } => write!(
                f,
                "Execution failure: {} {} {}",
                location, function_index, code_offset
            ),
            VMStatusView::VerificationError => write!(f, "Verification Error"),
            VMStatusView::DeserializationError => write!(f, "Deserialization Error"),
            VMStatusView::PublishingFailure => write!(f, "Publishing Failure"),
        }
    }
}

/// Below is a sample response from a failed JSON RPC call:
/// "{
///   "error": {
///     "code": -32000,
///     "data": null,
///     "message": "Server error: send failed because receiver is gone"
///   },
///   "id": 0,
///   "jsonrpc: "2.0"
/// }"
#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct JSONRpcFailureResponse {
    id: u64,
    jsonrpc: String,
    error: JsonRpcError,
}

/// A custom error returned by the JSON RPC server for API calls that fail internally (e.g.,
/// an internal error during execution on the server side).
#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct JsonRpcError {
    code: i16,
    message: String,
    data: Option<Value>,
}

/// Represents a vector of bytes using hex encoding.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Bytes(pub String);

impl Bytes {
    pub fn into_bytes(self) -> Result<Vec<u8>, Error> {
        Ok(hex::decode(self.0)?)
    }
}

impl From<&[u8]> for Bytes {
    fn from(bytes: &[u8]) -> Self {
        Self(hex::encode(bytes))
    }
}

impl From<&Vec<u8>> for Bytes {
    fn from(bytes: &Vec<u8>) -> Self {
        Self(hex::encode(bytes))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;
    use diem_config::utils;
    use diem_crypto::HashValue;
    use diem_json_rpc::test_bootstrap;
    use diem_types::{
        account_address::AccountAddress,
        account_state_blob::{AccountStateBlob, AccountStateWithProof},
        block_info::BlockInfo,
        contract_event::ContractEvent,
        epoch_change::EpochChangeProof,
        event::EventKey,
        ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
        mempool_status::{MempoolStatus, MempoolStatusCode},
        proof::{AccumulatorConsistencyProof, SparseMerkleProof},
        transaction::{TransactionListWithProof, TransactionWithProof, Version},
    };
    use diemdb::errors::DiemDbError::NotFound;
    use futures::{channel::mpsc::channel, StreamExt};
    use std::{collections::BTreeMap, sync::Arc};
    use storage_interface::{DbReader, Order, StartupInfo, TreeState};
    use tokio::runtime::Runtime;
    use vm_validator::{
        mocks::mock_vm_validator::MockVMValidator, vm_validator::TransactionValidation,
    };

    #[test]
    fn test_submit_transaction() {
        let mock_db = create_empty_mock_db();
        let (client, _server) = create_client_and_server(mock_db, true);
        let signed_transaction = test_helpers::generate_signed_transaction();

        // Ensure transaction submitted and validated successfully
        let result = client.submit_transaction(signed_transaction);
        assert!(result.is_ok());
    }

    #[test]
    fn test_submit_transaction_failure() {
        let mock_db = create_empty_mock_db();
        // When creating the JSON RPC server, specify 'mock_validator=false' to prevent a vm validator
        // being passed to the server. This will cause any transaction submission requests to the JSON
        // RPC server to fail, thus causing the server to return an error.
        let (client, _server) = create_client_and_server(mock_db, false);
        let signed_transaction = test_helpers::generate_signed_transaction();

        // Ensure transaction submitted successfully
        let result = client.submit_transaction(signed_transaction);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_account_state() {
        // Create test account state data
        let account = AccountAddress::random();
        let account_state = test_helpers::create_test_account_state();
        let version_height = 0;
        let account_state_with_proof =
            test_helpers::create_test_state_with_proof(&account_state, version_height);

        // Create an account to account_state_with_proof mapping
        let mut map = BTreeMap::new();
        map.insert(account, account_state_with_proof);

        // Populate the test database with the test data and create the client/server
        let mock_db = MockDiemDB::new(map);
        let (client, _server) = create_client_and_server(mock_db, true);

        // Ensure the client returns the correct AccountState
        let result = client.get_account_state(account, Some(version_height));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), account_state);
    }

    #[test]
    fn test_get_account_state_version_not_specified() {
        // Create test account state data
        let account = AccountAddress::random();
        let account_state = test_helpers::create_test_account_state();
        let account_state_with_proof =
            test_helpers::create_test_state_with_proof(&account_state, 0);

        // Create an account to account_state_with_proof mapping
        let mut map = BTreeMap::new();
        map.insert(account, account_state_with_proof);

        // Populate the test database with the test data and create the client/server
        let mock_db = MockDiemDB::new(map);
        let (client, _server) = create_client_and_server(mock_db, true);

        // Ensure the client returns the latest AccountState (even though no version was specified)
        let result = client.get_account_state(account, None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), account_state);
    }

    #[test]
    fn test_get_account_state_missing() {
        let mock_db = create_empty_mock_db();
        let (client, _server) = create_client_and_server(mock_db, true);

        // Ensure the client returns an error for a missing AccountState
        let account = AccountAddress::random();
        let result = client.get_account_state(account, Some(0));
        assert!(result.is_err());
    }

    #[test]
    fn test_get_account_state_missing_blob() {
        // Create test account state data
        let account = AccountAddress::random();
        let version_height = 0;
        let account_state_proof = test_helpers::create_test_state_proof();
        let account_state_with_proof =
            AccountStateWithProof::new(version_height, None, account_state_proof);

        // Create an account to account_state_with_proof mapping
        let mut map = BTreeMap::new();
        map.insert(account, account_state_with_proof);

        // Populate the test database with the test data and create the client/server
        let mock_db = MockDiemDB::new(map);
        let (client, _server) = create_client_and_server(mock_db, true);

        // Ensure the client returns an error for the missing AccountState
        let result = client.get_account_state(account, Some(version_height));
        assert!(result.is_err());
    }

    /// Generates and returns a (client, server) pair, where the client is a lightweight JSON client
    /// and the server is a JSON server that serves the JSON RPC requests. The server communicates
    /// with the given database to handle each JSON RPC request. If mock_validator is set to true,
    /// the server is also given a mock vm validator to validate any submitted transactions.
    fn create_client_and_server(db: MockDiemDB, mock_validator: bool) -> (JsonRpcClient, Runtime) {
        let address = "0.0.0.0";
        let port = utils::get_available_port();
        let host = format!("{}:{}", address, port);
        let (mp_sender, mut mp_events) = channel(1024);
        let server = test_bootstrap(host.parse().unwrap(), Arc::new(db), mp_sender);

        let url = format!("http://{}", host);
        let client = JsonRpcClient::new(url);

        if mock_validator {
            // Provide a VMValidator to the runtime.
            server.spawn(async move {
                while let Some((txn, cb)) = mp_events.next().await {
                    let vm_status = MockVMValidator.validate_transaction(txn).unwrap().status();
                    let result = if vm_status.is_some() {
                        (MempoolStatus::new(MempoolStatusCode::VmError), vm_status)
                    } else {
                        (MempoolStatus::new(MempoolStatusCode::Accepted), None)
                    };
                    cb.send(Ok(result)).unwrap();
                }
            });
        }

        (client, server)
    }

    /// Returns an empty mock database for testing.
    fn create_empty_mock_db() -> MockDiemDB {
        MockDiemDB::new(BTreeMap::new())
    }

    // This offers a simple mock of DiemDB for testing.
    #[derive(Clone)]
    pub struct MockDiemDB {
        account_states_with_proof: BTreeMap<AccountAddress, AccountStateWithProof>,
    }

    /// A mock diem database for test purposes.
    impl MockDiemDB {
        pub fn new(
            account_states_with_proof: BTreeMap<AccountAddress, AccountStateWithProof>,
        ) -> Self {
            Self {
                account_states_with_proof,
            }
        }
    }

    /// We only require implementing a subset of these API calls for testing purposes. To keep
    /// our code as minimal as possible, the unimplemented API calls simply return an error.
    impl DbReader for MockDiemDB {
        fn get_transactions(
            &self,
            _start_version: u64,
            _limit: u64,
            _ledger_version: u64,
            _fetch_events: bool,
        ) -> Result<TransactionListWithProof> {
            unimplemented!()
        }

        fn get_events(
            &self,
            _event_key: &EventKey,
            _start: u64,
            _order: Order,
            _limit: u64,
        ) -> Result<Vec<(u64, ContractEvent)>> {
            unimplemented!()
        }

        fn get_latest_account_state(
            &self,
            _address: AccountAddress,
        ) -> Result<Option<AccountStateBlob>> {
            unimplemented!()
        }

        fn get_latest_ledger_info(&self) -> Result<LedgerInfoWithSignatures> {
            Ok(LedgerInfoWithSignatures::new(
                LedgerInfo::new(
                    BlockInfo::new(0, 0, HashValue::zero(), HashValue::zero(), 0, 0, None),
                    HashValue::zero(),
                ),
                BTreeMap::new(),
            ))
        }

        fn get_startup_info(&self) -> Result<Option<StartupInfo>> {
            unimplemented!()
        }

        fn get_txn_by_account(
            &self,
            _address: AccountAddress,
            _seq_num: u64,
            _ledger_version: u64,
            _fetch_events: bool,
        ) -> Result<Option<TransactionWithProof>> {
            unimplemented!()
        }

        fn get_state_proof_with_ledger_info(
            &self,
            _known_version: u64,
            _ledger_info: LedgerInfoWithSignatures,
        ) -> Result<(EpochChangeProof, AccumulatorConsistencyProof)> {
            unimplemented!()
        }

        fn get_state_proof(
            &self,
            _known_version: u64,
        ) -> Result<(
            LedgerInfoWithSignatures,
            EpochChangeProof,
            AccumulatorConsistencyProof,
        )> {
            unimplemented!()
        }

        /// Return the associated AccountStateWithProof for the given account address. If the
        /// AccountStateWithProof doesn't exist, an error is returned.
        fn get_account_state_with_proof(
            &self,
            address: AccountAddress,
            _version: Version,
            _ledger_version: Version,
        ) -> Result<AccountStateWithProof> {
            if let Some(account_state_proof) = self.account_states_with_proof.get(&address) {
                Ok(account_state_proof.clone())
            } else {
                Err(NotFound("AccountStateWithProof".into()).into())
            }
        }

        fn get_account_state_with_proof_by_version(
            &self,
            _address: AccountAddress,
            _version: u64,
        ) -> Result<(Option<AccountStateBlob>, SparseMerkleProof)> {
            unimplemented!()
        }

        fn get_latest_state_root(&self) -> Result<(u64, HashValue)> {
            unimplemented!()
        }

        fn get_latest_tree_state(&self) -> Result<TreeState> {
            unimplemented!()
        }

        fn get_epoch_ending_ledger_infos(&self, _: u64, _: u64) -> Result<EpochChangeProof> {
            unimplemented!()
        }

        fn get_epoch_ending_ledger_info(&self, _: u64) -> Result<LedgerInfoWithSignatures> {
            unimplemented!()
        }

        fn get_block_timestamp(&self, _: u64) -> Result<u64> {
            unimplemented!()
        }
    }
}

#[cfg(any(test, feature = "fuzzing"))]
/// This module provides all the functionality necessary to test the secure JSON RPC client using
/// fuzzing.
pub mod fuzzing {
    use crate::{
        AccountStateResponse, AccountStateWithProofResponse, Bytes, SubmitTransactionResponse,
        TransactionView, TransactionViewResponse, VMStatusView,
    };
    use diem_proptest_helpers::Index;
    use diem_types::proptest_types::{arb_json_value, AccountInfoUniverse, AccountStateBlobGen};
    use proptest::prelude::*;
    use ureq::Response;

    // Note: these tests ensure that the various fuzzers are maintained (i.e., not broken
    // at some time in the future and only discovered when a fuzz test fails).
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10))]

        #[test]
        fn test_get_account_state_proptest(input in arb_account_state_response()) {
            let _ = crate::process_account_state_response(input);
        }

        #[test]
        fn test_submit_transaction_proptest(input in arb_submit_transaction_response()) {
            let _ = crate::process_submit_transaction_response(input);
        }

        #[test]
        fn test_transaction_status_proptest(input in arb_transaction_status_response()) {
            let _ = crate::process_transaction_status_response(input);
        }
    }

    // This generates an arbitrary response for the get_account_state_with_proof() JSON RPC API
    // call.
    prop_compose! {
        pub fn arb_account_state_response(
        )(
            status in any::<u16>(),
            status_text in any::<String>(),
            id in any::<u64>(),
            jsonrpc in any::<String>(),
            version_height in any::<u64>(),
            universe in any_with::<AccountInfoUniverse>(1).no_shrink(),
            index in any::<Index>(),
            account_state_blob_gen in any::<AccountStateBlobGen>(),
        ) -> Response {
            let account_state_blob = account_state_blob_gen.materialize(index, &universe);
            let encoded_blob = Bytes::from(&bcs::to_bytes(&account_state_blob).unwrap());

            let response_body = AccountStateWithProofResponse {
                id,
                jsonrpc,
                result: AccountStateResponse {
                    version: version_height,
                    blob: Some(encoded_blob),
                },
            };
            let response_body =
                serde_json::to_string::<AccountStateWithProofResponse>(&response_body).unwrap();
            Response::new(status, &status_text, &response_body)
        }
    }

    // This generates an arbitrary response for the submit_transaction() JSON RPC API call.
    prop_compose! {
        pub fn arb_submit_transaction_response(
        )(
            status in any::<u16>(),
            status_text in any::<String>(),
            id in any::<u64>(),
            jsonrpc in any::<String>(),
            result in arb_json_value(),
        ) -> Response {
            let response_body = SubmitTransactionResponse {
                id,
                jsonrpc,
                result: Some(result),
            };
            let response_body =
                serde_json::to_string::<SubmitTransactionResponse>(&response_body).unwrap();
            Response::new(status, &status_text, &response_body)
        }
    }

    // This generates an arbitrary response for the get_account_transaction() JSON RPC API call.
    prop_compose! {
        pub fn arb_transaction_status_response(
        )(
            status in any::<u16>(),
            status_text in any::<String>(),
            id in any::<u64>(),
            jsonrpc in any::<String>(),
            vm_status in arb_vm_status_view(),
        ) -> Response {
            let transaction_view = TransactionView {
                vm_status
            };
            let response_body = TransactionViewResponse {
                id,
                jsonrpc,
                result: Some(transaction_view),
            };
            let response_body =
                serde_json::to_string::<TransactionViewResponse>(&response_body).unwrap();
            Response::new(status, &status_text, &response_body)
        }
    }

    // This function generates an arbitrary VMStatusView.
    fn arb_vm_status_view() -> impl Strategy<Value = VMStatusView> {
        prop_oneof![
            Just(VMStatusView::Executed),
            Just(VMStatusView::OutOfGas),
            (any::<String>(), any::<u64>()).prop_map(|(location, abort_code)| {
                VMStatusView::MoveAbort {
                    location,
                    abort_code,
                }
            }),
            (any::<String>(), any::<u16>(), any::<u16>()).prop_map(
                |(location, function_index, code_offset)| {
                    VMStatusView::ExecutionFailure {
                        location,
                        function_index,
                        code_offset,
                    }
                }
            ),
            Just(VMStatusView::VerificationError),
            Just(VMStatusView::DeserializationError),
            Just(VMStatusView::PublishingFailure),
        ]
    }
}

#[cfg(test)]
mod test_helpers {
    use diem_crypto::{ed25519::Ed25519PrivateKey, HashValue, PrivateKey, Uniform};
    use diem_types::{
        account_address::AccountAddress,
        account_config::{AccountResource, BalanceResource},
        account_state::AccountState,
        account_state_blob::{AccountStateBlob, AccountStateWithProof},
        event::EventHandle,
        proof::{AccountStateProof, AccumulatorProof, SparseMerkleProof, TransactionInfoWithProof},
        test_helpers::transaction_test_helpers::get_test_signed_txn,
        transaction::{SignedTransaction, TransactionInfo},
        vm_status::KeptVMStatus,
    };
    use std::convert::TryFrom;

    /// Generates an AccountStateWithProof using the given AccountState and version height for
    /// testing.
    pub(crate) fn create_test_state_with_proof(
        account_state: &AccountState,
        version_height: u64,
    ) -> AccountStateWithProof {
        AccountStateWithProof::new(
            version_height,
            Some(AccountStateBlob::try_from(account_state).unwrap()),
            create_test_state_proof(),
        )
    }

    /// Generates an AccountState for testing.
    pub(crate) fn create_test_account_state() -> AccountState {
        let account_resource = create_test_account_resource();
        let balance_resource = create_test_balance_resource();
        AccountState::try_from((&account_resource, &balance_resource)).unwrap()
    }

    /// Generates an AccountStateProof for testing.
    pub(crate) fn create_test_state_proof() -> AccountStateProof {
        let transaction_info = TransactionInfo::new(
            HashValue::zero(),
            HashValue::zero(),
            HashValue::zero(),
            0,
            KeptVMStatus::MiscellaneousError,
        );

        AccountStateProof::new(
            TransactionInfoWithProof::new(AccumulatorProof::new(vec![]), transaction_info),
            SparseMerkleProof::new(None, vec![]),
        )
    }

    /// Generates and returns a (randomized) SignedTransaction for testing.
    pub(crate) fn generate_signed_transaction() -> SignedTransaction {
        let sender = AccountAddress::random();
        let private_key = Ed25519PrivateKey::generate_for_testing();
        get_test_signed_txn(sender, 0, &private_key, private_key.public_key(), None)
    }

    /// Generates an AccountResource for testing.
    fn create_test_account_resource() -> AccountResource {
        AccountResource::new(
            10,
            vec![],
            None,
            None,
            EventHandle::random_handle(100),
            EventHandle::random_handle(100),
        )
    }

    /// Generates a BalanceResource for testing.
    fn create_test_balance_resource() -> BalanceResource {
        BalanceResource::new(100)
    }
}
