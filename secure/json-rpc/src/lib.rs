// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! The purpose of the JsonRpcClient presented here is to provide a lightweight and secure
//! JSON RPC client to talk to the JSON RPC service offered by Libra Full Nodes. This is useful
//! for various security-critical components (e.g., the secure key manager), as it allows
//! interaction with the Libra blockchain in a minimal and secure manner.
//!
//! Note: While a JSON RPC client implementation already exists in the Libra codebase, this
//! provides a simpler and (hopefully) more secure implementation with fewer dependencies.
#![forbid(unsafe_code)]

use hex::FromHexError;
use libra_types::{
    account_address::AccountAddress, account_state::AccountState,
    account_state_blob::AccountStateBlob, transaction::SignedTransaction,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{convert::TryFrom, io};
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

impl From<lcs::Error> for Error {
    fn from(error: lcs::Error) -> Self {
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
pub struct JsonRpcClient {
    host: String,
}

impl JsonRpcClient {
    pub fn new(host: String) -> Self {
        Self { host }
    }

    /// Submits a signed transaction to the Libra blockchain via the JSON RPC API.
    pub fn submit_signed_transaction(
        &self,
        signed_transaction: SignedTransaction,
    ) -> Result<(), Error> {
        let response = self.submit_transaction(signed_transaction)?;
        match response.status() {
            200 => {
                let response = &response.into_string()?;
                if let Ok(failure_response) =
                    serde_json::from_str::<JSONRpcFailureResponse>(response)
                {
                    Err(Error::InternalRPCError(format!("{:?}", failure_response)))
                } else {
                    let _submit_response =
                        serde_json::from_str::<SubmitTransactionResponse>(response)?;
                    Ok(())
                }
            }
            _ => Err(Error::RPCFailure(response.into_string()?)),
        }
    }

    /// Returns the associated AccountState for a specific account at a given version height
    /// using the JSON RCP API.
    pub fn get_account_state(
        &self,
        account: AccountAddress,
        version: u64,
    ) -> Result<AccountState, Error> {
        let response = self.get_account_state_with_proof(account, version, version)?;
        match response.status() {
            200 => {
                let response = &response.into_string()?;
                if let Ok(failure_response) =
                    serde_json::from_str::<JSONRpcFailureResponse>(response)
                {
                    Err(Error::InternalRPCError(format!("{:?}", failure_response)))
                } else if let Some(blob_bytes) =
                    serde_json::from_str::<AccountStateWithProofResponse>(response)?
                        .result
                        .blob
                {
                    let account_state_blob = AccountStateBlob::from(lcs::from_bytes::<Vec<u8>>(
                        &*blob_bytes.into_bytes()?,
                    )?);
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

    // Executes the specified request method using the given parameters by contacting the JSON RPC
    // server.
    fn execute_request(&self, method: String, params: Vec<Value>) -> Response {
        ureq::post(&self.host)
            .timeout_connect(REQUEST_TIMEOUT)
            .send_json(
                json!({"jsonrpc": JSON_RPC_VERSION, "method": method, "params": params, "id": 0}),
            )
    }

    // Sends a submit() request to the JSON RPC server using the given transaction.
    fn submit_transaction(&self, signed_transaction: SignedTransaction) -> Result<Response, Error> {
        let method = "submit".to_string();
        let params = vec![Value::String(hex::encode(lcs::to_bytes(
            &signed_transaction,
        )?))];
        Ok(self.execute_request(method, params))
    }

    // Sends a get_account_state_with_proof() request to the JSON RPC server for the specified
    // account, version height and ledger_version height (for the proof).
    fn get_account_state_with_proof(
        &self,
        account: AccountAddress,
        version: u64,
        ledger_version: u64,
    ) -> Result<Response, Error> {
        let method = "get_account_state_with_proof".to_string();
        let params = vec![
            Value::String(account.to_string()),
            Value::Number(version.into()),
            Value::Number(ledger_version.into()),
        ];
        Ok(self.execute_request(method, params))
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
    use futures::{channel::mpsc::channel, StreamExt};
    use libra_config::utils;
    use libra_crypto::{ed25519, HashValue, PrivateKey, Uniform};
    use libra_json_rpc::bootstrap;
    use libra_types::{
        account_address::AccountAddress,
        account_config::{AccountResource, BalanceResource},
        account_state::AccountState,
        account_state_blob::{AccountStateBlob, AccountStateWithProof},
        block_info::BlockInfo,
        contract_event::ContractEvent,
        event::{EventHandle, EventKey},
        ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
        mempool_status::{MempoolStatus, MempoolStatusCode},
        proof::{
            AccountStateProof, AccumulatorConsistencyProof, AccumulatorProof, SparseMerkleProof,
        },
        test_helpers::transaction_test_helpers::get_test_signed_txn,
        transaction::{
            SignedTransaction, TransactionInfo, TransactionListWithProof, TransactionWithProof,
            Version,
        },
        validator_change::ValidatorChangeProof,
        vm_error::StatusCode,
    };
    use libradb::errors::LibraDbError::NotFound;
    use std::{collections::BTreeMap, convert::TryFrom, sync::Arc};
    use storage_interface::DbReader;
    use storage_proto::StartupInfo;
    use tokio::runtime::Runtime;
    use vm_validator::{
        mocks::mock_vm_validator::MockVMValidator, vm_validator::TransactionValidation,
    };

    #[test]
    fn test_submit_transaction() {
        let mock_db = create_empty_mock_db();
        let (client, _server) = create_client_and_server(mock_db, true);
        let signed_transaction = generate_signed_transaction();

        // Ensure transaction submitted and validated successfully
        let result = client.submit_signed_transaction(signed_transaction);
        assert!(result.is_ok());
    }

    #[test]
    fn test_submit_transaction_failure() {
        let mock_db = create_empty_mock_db();
        // When creating the JSON RPC server, specify 'mock_validator=false' to prevent a vm validator
        // being passed to the server. This will cause any transaction submission requests to the JSON
        // RPC server to fail, thus causing the server to return an error.
        let (client, _server) = create_client_and_server(mock_db, false);
        let signed_transaction = generate_signed_transaction();

        // Ensure transaction submitted successfully
        let result = client.submit_signed_transaction(signed_transaction);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_account_state() {
        // Create test account state data
        let account = AccountAddress::random();
        let account_state = create_test_account_state();
        let version_height = 0;
        let account_state_with_proof = create_test_state_with_proof(&account_state, version_height);

        // Create an account to account_state_with_proof mapping
        let mut map = BTreeMap::new();
        map.insert(account, account_state_with_proof);

        // Populate the test database with the test data and create the client/server
        let mock_db = MockLibraDB::new(map);
        let (client, _server) = create_client_and_server(mock_db, true);

        // Ensure the client returns the correct AccountState
        let result = client.get_account_state(account, version_height);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), account_state);
    }

    #[test]
    fn test_get_account_state_missing() {
        let mock_db = create_empty_mock_db();
        let (client, _server) = create_client_and_server(mock_db, true);

        // Ensure the client returns an error for a missing AccountState
        let account = AccountAddress::random();
        let result = client.get_account_state(account, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_account_state_missing_blob() {
        // Create test account state data
        let account = AccountAddress::random();
        let version_height = 0;
        let account_state_proof = create_test_state_proof();
        let account_state_with_proof =
            AccountStateWithProof::new(version_height, None, account_state_proof);

        // Create an account to account_state_with_proof mapping
        let mut map = BTreeMap::new();
        map.insert(account, account_state_with_proof);

        // Populate the test database with the test data and create the client/server
        let mock_db = MockLibraDB::new(map);
        let (client, _server) = create_client_and_server(mock_db, true);

        // Ensure the client returns an error for the missing AccountState
        let result = client.get_account_state(account, version_height);
        assert!(result.is_err());
    }

    /// Generates and returns a (client, server) pair, where the client is a lightweight JSON client
    /// and the server is a JSON server that serves the JSON RPC requests. The server communicates
    /// with the given database to handle each JSON RPC request. If mock_validator is set to true,
    /// the server is also given a mock vm validator to validate any submitted transactions.
    fn create_client_and_server(db: MockLibraDB, mock_validator: bool) -> (JsonRpcClient, Runtime) {
        let address = "0.0.0.0";
        let port = utils::get_available_port();
        let host = format!("{}:{}", address, port);
        let (mp_sender, mut mp_events) = channel(1024);
        let server = bootstrap(host.parse().unwrap(), Arc::new(db), mp_sender);

        let url = format!("http://{}", host);
        let client = JsonRpcClient::new(url);

        if mock_validator {
            // Provide a VMValidator to the runtime.
            server.spawn(async move {
                while let Some((txn, cb)) = mp_events.next().await {
                    let vm_status = MockVMValidator
                        .validate_transaction(txn)
                        .await
                        .unwrap()
                        .status();
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

    /// Generates an AccountStateProof for testing.
    fn create_test_state_proof() -> AccountStateProof {
        let transaction_info = TransactionInfo::new(
            HashValue::zero(),
            HashValue::zero(),
            HashValue::zero(),
            0,
            StatusCode::UNKNOWN_STATUS,
        );

        AccountStateProof::new(
            AccumulatorProof::new(vec![]),
            transaction_info,
            SparseMerkleProof::new(None, vec![]),
        )
    }

    /// Generates an AccountStateWithProof using the given AccountState and version height for
    /// testing.
    fn create_test_state_with_proof(
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
    fn create_test_account_state() -> AccountState {
        let account_resource = create_test_account_resource();
        let balance_resource = create_test_balance_resource();
        AccountState::try_from((&account_resource, &balance_resource)).unwrap()
    }

    /// Generates an AccountResource for testing.
    fn create_test_account_resource() -> AccountResource {
        AccountResource::new(
            10,
            vec![],
            false,
            false,
            EventHandle::random_handle(100),
            EventHandle::random_handle(100),
            0,
        )
    }

    /// Generates a BalanceResource for testing.
    fn create_test_balance_resource() -> BalanceResource {
        BalanceResource::new(100)
    }

    /// Generates and returns a (randomized) SignedTransaction for testing.
    fn generate_signed_transaction() -> SignedTransaction {
        let sender = AccountAddress::random();
        let private_key = ed25519::SigningKey::generate_for_testing();
        get_test_signed_txn(sender, 0, &private_key, private_key.public_key(), None)
    }

    /// Returns an empty mock database for testing.
    fn create_empty_mock_db() -> MockLibraDB {
        MockLibraDB::new(BTreeMap::new())
    }

    // This offers a simple mock of LibraDB for testing.
    #[derive(Clone)]
    pub struct MockLibraDB {
        account_states_with_proof: BTreeMap<AccountAddress, AccountStateWithProof>,
    }

    /// A mock libra database for test purposes.
    impl MockLibraDB {
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
    impl DbReader for MockLibraDB {
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
            _ascending: bool,
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
        ) -> Result<(ValidatorChangeProof, AccumulatorConsistencyProof)> {
            unimplemented!()
        }

        fn get_state_proof(
            &self,
            _known_version: u64,
        ) -> Result<(
            LedgerInfoWithSignatures,
            ValidatorChangeProof,
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
                Err(NotFound("AccountStateWithProof".to_string()).into())
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
    }
}
