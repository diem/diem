// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! This crate implements the storage service.
//!
//! The user of storage service is supposed to use it via client lib provided in
//! [`storage-client`](../storage-client/index.html) instead of via

use anyhow::Result;
use libra_config::config::NodeConfig;
use libra_logger::prelude::*;
use libra_secure_net::NetworkServer;
use libra_types::{account_state_blob::AccountStateBlob, proof::SparseMerkleProof};
use libradb::LibraDB;
use std::{
    sync::Arc,
    thread::{self, JoinHandle},
};
use storage_interface::{DbReader, DbWriter, Error, StartupInfo};

/// Starts storage service with a given LibraDB
pub fn start_storage_service_with_db(
    config: &NodeConfig,
    libra_db: Arc<LibraDB>,
) -> JoinHandle<()> {
    let storage_service = StorageService { db: libra_db };
    storage_service.run(config)
}

#[derive(Clone)]
pub struct StorageService {
    db: Arc<LibraDB>,
}

impl StorageService {
    fn handle_message(&self, input_message: Vec<u8>) -> Result<Vec<u8>, Error> {
        let input = lcs::from_bytes(&input_message)?;
        let output = match input {
            storage_interface::StorageRequest::GetAccountStateWithProofByVersionRequest(req) => {
                lcs::to_bytes(&self.get_account_state_with_proof_by_version(&req))
            }
            storage_interface::StorageRequest::GetStartupInfoRequest => {
                lcs::to_bytes(&self.get_startup_info())
            }
            storage_interface::StorageRequest::SaveTransactionsRequest(req) => {
                lcs::to_bytes(&self.save_transactions(&req))
            }
        };
        Ok(output?)
    }

    fn get_account_state_with_proof_by_version(
        &self,
        req: &storage_interface::GetAccountStateWithProofByVersionRequest,
    ) -> Result<(Option<AccountStateBlob>, SparseMerkleProof), Error> {
        Ok(self
            .db
            .get_account_state_with_proof_by_version(req.address, req.version)?)
    }

    fn get_startup_info(&self) -> Result<Option<StartupInfo>, Error> {
        Ok(self.db.get_startup_info()?)
    }

    fn save_transactions(
        &self,
        req: &storage_interface::SaveTransactionsRequest,
    ) -> Result<(), Error> {
        Ok(self.db.save_transactions(
            &req.txns_to_commit,
            req.first_version,
            req.ledger_info_with_signatures.as_ref(),
        )?)
    }

    fn run(self, config: &NodeConfig) -> JoinHandle<()> {
        let mut network_server = NetworkServer::new(config.storage.address);
        thread::spawn(move || loop {
            if let Err(e) = self.process_one_message(&mut network_server) {
                warn!("Failed to process message: {}", e);
            }
        })
    }

    fn process_one_message(&self, network_server: &mut NetworkServer) -> Result<(), Error> {
        let request = network_server.read()?;
        let response = self.handle_message(request)?;
        network_server.write(&response)?;
        Ok(())
    }
}

#[cfg(test)]
mod storage_service_test;
