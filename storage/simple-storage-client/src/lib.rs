// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use libra_secure_net::NetworkClient;
use libra_types::{
    account_address::AccountAddress,
    account_state_blob::AccountStateBlob,
    ledger_info::LedgerInfoWithSignatures,
    proof::SparseMerkleProof,
    transaction::{TransactionToCommit, Version},
};
use serde::de::DeserializeOwned;
use std::{net::SocketAddr, sync::Mutex};
use storage_interface::{
    Error, GetAccountStateWithProofByVersionRequest, SaveTransactionsRequest, StartupInfo,
    StorageRequest,
};

pub struct SimpleStorageClient {
    network_client: Mutex<NetworkClient>,
}

impl SimpleStorageClient {
    pub fn new(server_address: &SocketAddr) -> Self {
        Self {
            network_client: Mutex::new(NetworkClient::new(*server_address)),
        }
    }

    fn request<T: DeserializeOwned>(&self, input: StorageRequest) -> Result<T, Error> {
        let input_message = lcs::to_bytes(&input)?;
        let mut client = self.network_client.lock().unwrap();
        client.write(&input_message)?;
        let result = client.read()?;
        lcs::from_bytes(&result)?
    }

    pub fn get_account_state_with_proof_by_version(
        &self,
        address: AccountAddress,
        version: Version,
    ) -> Result<(Option<AccountStateBlob>, SparseMerkleProof), Error> {
        self.request(StorageRequest::GetAccountStateWithProofByVersionRequest(
            Box::new(GetAccountStateWithProofByVersionRequest::new(
                address, version,
            )),
        ))
    }

    pub fn get_startup_info(&self) -> Result<Option<StartupInfo>, Error> {
        self.request(StorageRequest::GetStartupInfoRequest)
    }

    pub fn save_transactions(
        &self,
        txns_to_commit: Vec<TransactionToCommit>,
        first_version: Version,
        ledger_info_with_sigs: Option<LedgerInfoWithSignatures>,
    ) -> Result<(), Error> {
        self.request(StorageRequest::SaveTransactionsRequest(Box::new(
            SaveTransactionsRequest::new(txns_to_commit, first_version, ledger_info_with_sigs),
        )))
    }
}
