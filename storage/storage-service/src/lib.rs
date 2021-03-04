// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! This crate implements the storage service.
//!
//! The user of storage service is supposed to use it via client lib provided in
//! [`storage-client`](../storage-client/index.html) instead of via

use anyhow::Result;
use diem_config::config::NodeConfig;
use diem_logger::prelude::*;
use diem_secure_net::NetworkServer;
use diem_types::{account_state_blob::AccountStateBlob, proof::SparseMerkleProof};
use diemdb::DiemDB;
use std::{
    sync::Arc,
    thread::{self, JoinHandle},
};
use storage_interface::{DbReader, DbWriter, Error, StartupInfo};

/// Starts storage service with a given DiemDB
pub fn start_storage_service_with_db(config: &NodeConfig, diem_db: Arc<DiemDB>) -> JoinHandle<()> {
    let storage_service = StorageService { db: diem_db };
    storage_service.run(config)
}

#[derive(Clone)]
pub struct StorageService {
    db: Arc<DiemDB>,
}

impl StorageService {
    fn handle_message(&self, input_message: Vec<u8>) -> Result<Vec<u8>, Error> {
        let input = bcs::from_bytes(&input_message)?;
        let output = match input {
            storage_interface::StorageRequest::GetAccountStateWithProofByVersionRequest(req) => {
                bcs::to_bytes(&self.get_account_state_with_proof_by_version(&req))
            }
            storage_interface::StorageRequest::GetStartupInfoRequest => {
                bcs::to_bytes(&self.get_startup_info())
            }
            storage_interface::StorageRequest::SaveTransactionsRequest(req) => {
                bcs::to_bytes(&self.save_transactions(&req))
            }
        };
        Ok(output?)
    }

    fn get_account_state_with_proof_by_version(
        &self,
        req: &storage_interface::GetAccountStateWithProofByVersionRequest,
    ) -> Result<
        (
            Option<AccountStateBlob>,
            SparseMerkleProof<AccountStateBlob>,
        ),
        Error,
    > {
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
        let mut network_server =
            NetworkServer::new("storage", config.storage.address, config.storage.timeout_ms);
        let ret = thread::spawn(move || loop {
            if let Err(e) = self.process_one_message(&mut network_server) {
                warn!(
                    error = ?e,
                    "Failed to process message.",
                );
            }
        });
        info!("StorageService spawned.");
        ret
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
