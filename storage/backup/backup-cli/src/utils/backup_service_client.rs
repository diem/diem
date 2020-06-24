// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use futures::TryStreamExt;
use libra_crypto::HashValue;
use libra_types::transaction::Version;
use structopt::StructOpt;
use tokio::prelude::*;
use tokio_util::compat::FuturesAsyncReadCompatExt;

#[derive(StructOpt)]
pub struct BackupServiceClientOpt {
    #[structopt(
        long = "backup-service-port",
        help = "Backup service port. The service must listen on localhost."
    )]
    pub port: u16,
}

pub struct BackupServiceClient {
    port: u16,
    client: reqwest::Client,
}

impl BackupServiceClient {
    pub fn new_with_opt(opt: BackupServiceClientOpt) -> Self {
        Self::new(opt.port)
    }

    pub fn new(port: u16) -> Self {
        Self {
            port,
            client: reqwest::Client::builder()
                .no_proxy()
                .build()
                .expect("Http client should build."),
        }
    }

    async fn get(&self, path: &str) -> Result<impl AsyncRead> {
        Ok(self
            .client
            .get(&format!("http://localhost:{}/{}", self.port, path))
            .send()
            .await?
            .error_for_status()?
            .bytes_stream()
            .map_err(|e| futures::io::Error::new(futures::io::ErrorKind::Other, e))
            .into_async_read()
            .compat())
    }

    pub async fn get_latest_state_root(&self) -> Result<(Version, HashValue)> {
        let mut buf = Vec::new();
        self.get("latest_state_root")
            .await?
            .read_to_end(&mut buf)
            .await?;
        Ok(lcs::from_bytes(&buf)?)
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
}
