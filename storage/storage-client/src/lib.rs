// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use anyhow::Result;
use libra_crypto::HashValue;
use libra_secure_net::NetworkClient;
use libra_types::{
    account_address::AccountAddress,
    account_state_blob::{AccountStateBlob, AccountStateWithProof},
    contract_event::ContractEvent,
    epoch_change::EpochChangeProof,
    event::EventKey,
    ledger_info::LedgerInfoWithSignatures,
    proof::{AccumulatorConsistencyProof, SparseMerkleProof},
    transaction::{TransactionListWithProof, TransactionToCommit, TransactionWithProof, Version},
};
use serde::de::DeserializeOwned;
use std::{net::SocketAddr, sync::Mutex};
use storage_interface::{
    DbReader, DbWriter, Error, GetAccountStateWithProofByVersionRequest, SaveTransactionsRequest,
    StartupInfo, StorageRequest, TreeState,
};

pub struct StorageClient {
    network_client: Mutex<NetworkClient>,
}

impl StorageClient {
    pub fn new(server_address: &SocketAddr) -> Self {
        Self {
            network_client: Mutex::new(NetworkClient::new(*server_address)),
        }
    }

    fn request<T: DeserializeOwned>(&self, input: StorageRequest) -> std::result::Result<T, Error> {
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
    ) -> std::result::Result<(Option<AccountStateBlob>, SparseMerkleProof), Error> {
        self.request(StorageRequest::GetAccountStateWithProofByVersionRequest(
            Box::new(GetAccountStateWithProofByVersionRequest::new(
                address, version,
            )),
        ))
    }

    pub fn get_startup_info(&self) -> std::result::Result<Option<StartupInfo>, Error> {
        self.request(StorageRequest::GetStartupInfoRequest)
    }

    pub fn save_transactions(
        &self,
        txns_to_commit: Vec<TransactionToCommit>,
        first_version: Version,
        ledger_info_with_sigs: Option<LedgerInfoWithSignatures>,
    ) -> std::result::Result<(), Error> {
        self.request(StorageRequest::SaveTransactionsRequest(Box::new(
            SaveTransactionsRequest::new(txns_to_commit, first_version, ledger_info_with_sigs),
        )))
    }
}

impl DbReader for StorageClient {
    fn get_account_state_with_proof_by_version(
        &self,
        address: AccountAddress,
        version: u64,
    ) -> Result<(Option<AccountStateBlob>, SparseMerkleProof)> {
        Ok(Self::get_account_state_with_proof_by_version(
            self, address, version,
        )?)
    }

    fn get_startup_info(&self) -> Result<Option<StartupInfo>> {
        Ok(Self::get_startup_info(self)?)
    }

    fn get_latest_account_state(
        &self,
        _address: AccountAddress,
    ) -> Result<Option<AccountStateBlob>> {
        unimplemented!()
    }

    fn get_latest_ledger_info(&self) -> Result<LedgerInfoWithSignatures> {
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
        _key: &EventKey,
        _start: u64,
        _ascending: bool,
        _limit: u64,
    ) -> Result<Vec<(u64, ContractEvent)>> {
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

    fn get_state_proof_with_ledger_info(
        &self,
        _known_version: u64,
        _ledger_info: LedgerInfoWithSignatures,
    ) -> Result<(EpochChangeProof, AccumulatorConsistencyProof)> {
        unimplemented!()
    }

    fn get_account_state_with_proof(
        &self,
        _address: AccountAddress,
        _version: Version,
        _ledger_version: Version,
    ) -> Result<AccountStateWithProof> {
        unimplemented!()
    }

    fn get_latest_state_root(&self) -> Result<(u64, HashValue)> {
        unimplemented!()
    }

    fn get_latest_tree_state(&self) -> Result<TreeState> {
        unimplemented!()
    }

    fn get_epoch_change_ledger_infos(
        &self,
        _start_epoch: u64,
        _end_epoch: u64,
    ) -> Result<EpochChangeProof> {
        unimplemented!()
    }

    fn get_ledger_info(&self, _: u64) -> Result<LedgerInfoWithSignatures> {
        unimplemented!()
    }
}

impl DbWriter for StorageClient {
    fn save_transactions(
        &self,
        txns_to_commit: &[TransactionToCommit],
        first_version: Version,
        ledger_info_with_sigs: Option<&LedgerInfoWithSignatures>,
    ) -> Result<()> {
        Ok(Self::save_transactions(
            self,
            txns_to_commit.to_vec(),
            first_version,
            ledger_info_with_sigs.cloned(),
        )?)
    }
}
