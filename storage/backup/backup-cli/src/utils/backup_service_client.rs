// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::utils::error_notes::ErrorNotes;
use anyhow::Result;
use diem_crypto::HashValue;
use diem_types::transaction::Version;
use diemdb::backup::backup_handler::DbState;
use futures::TryStreamExt;
use structopt::StructOpt;
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio_util::compat::FuturesAsyncReadCompatExt;

#[derive(StructOpt)]
pub struct BackupServiceClientOpt {
    #[structopt(
        long = "backup-service-address",
        default_value = "http://localhost:6186",
        help = "Backup service address."
    )]
    pub address: String,
}

pub struct BackupServiceClient {
    address: String,
    client: reqwest::Client,
}

impl BackupServiceClient {
    pub fn new_with_opt(opt: BackupServiceClientOpt) -> Self {
        Self::new(opt.address)
    }

    pub fn new(address: String) -> Self {
        Self {
            address,
            client: reqwest::Client::builder()
                .no_proxy()
                .build()
                .expect("Http client should build."),
        }
    }

    async fn get(&self, path: &str) -> Result<impl AsyncRead> {
        let url = format!("{}/{}", self.address, path);
        Ok(self
            .client
            .get(&url)
            .send()
            .await
            .err_notes(&url)?
            .error_for_status()
            .err_notes(&url)?
            .bytes_stream()
            .map_err(|e| futures::io::Error::new(futures::io::ErrorKind::Other, e))
            .into_async_read()
            .compat())
    }

    pub async fn get_db_state(&self) -> Result<Option<DbState>> {
        let mut buf = Vec::new();
        self.get("db_state").await?.read_to_end(&mut buf).await?;
        Ok(bcs::from_bytes(&buf)?)
    }

    pub async fn get_account_range_proof(
        &self,
        key: HashValue,
        version: Version,
    ) -> Result<impl AsyncRead> {
        self.get(&format!("state_range_proof/{}/{:x}", version, key))
            .await
    }

    pub async fn get_state_snapshot(&self, version: Version) -> Result<impl AsyncRead> {
        self.get(&format!("state_snapshot/{}", version)).await
    }

    pub async fn get_state_root_proof(&self, version: Version) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        self.get(&format!("state_root_proof/{}", version))
            .await?
            .read_to_end(&mut buf)
            .await?;
        Ok(buf)
    }

    pub async fn get_epoch_ending_ledger_infos(
        &self,
        start_epoch: u64,
        end_epoch: u64,
    ) -> Result<impl AsyncRead> {
        self.get(&format!(
            "epoch_ending_ledger_infos/{}/{}",
            start_epoch, end_epoch
        ))
        .await
    }

    pub async fn get_transactions(
        &self,
        start_version: Version,
        num_transactions: usize,
    ) -> Result<impl AsyncRead> {
        self.get(&format!(
            "transactions/{}/{}",
            start_version, num_transactions
        ))
        .await
    }

    pub async fn get_transaction_range_proof(
        &self,
        first_version: Version,
        last_version: Version,
    ) -> Result<impl AsyncRead> {
        self.get(&format!(
            "transaction_range_proof/{}/{}",
            first_version, last_version,
        ))
        .await
    }
}
