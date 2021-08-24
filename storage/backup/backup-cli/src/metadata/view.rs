// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::metadata::{
    EpochEndingBackupMeta, Metadata, StateSnapshotBackupMeta, TransactionBackupMeta,
};
use anyhow::{anyhow, ensure, Result};
use diem_types::transaction::Version;
use itertools::Itertools;
use std::{fmt, str::FromStr};

pub struct MetadataView {
    epoch_ending_backups: Vec<EpochEndingBackupMeta>,
    state_snapshot_backups: Vec<StateSnapshotBackupMeta>,
    transaction_backups: Vec<TransactionBackupMeta>,
}

impl MetadataView {
    pub fn get_storage_state(&self) -> BackupStorageState {
        let latest_epoch_ending_epoch =
            self.epoch_ending_backups.iter().map(|e| e.last_epoch).max();
        let latest_state_snapshot_version =
            self.state_snapshot_backups.iter().map(|s| s.version).max();
        let latest_transaction_version = self
            .transaction_backups
            .iter()
            .map(|t| t.last_version)
            .max();

        BackupStorageState {
            latest_epoch_ending_epoch,
            latest_state_snapshot_version,
            latest_transaction_version,
        }
    }

    pub fn select_state_snapshot(
        &self,
        target_version: Version,
    ) -> Result<Option<StateSnapshotBackupMeta>> {
        Ok(self
            .state_snapshot_backups
            .iter()
            .sorted()
            .rev()
            .find(|m| m.version <= target_version)
            .map(Clone::clone))
    }

    pub fn select_transaction_backups(
        &self,
        start_version: Version,
        target_version: Version,
    ) -> Result<Vec<TransactionBackupMeta>> {
        // This can be more flexible, but for now we assume and check backups are continuous in
        // range (which is always true when we backup from a single backup coordinator)
        let mut next_ver = 0;
        let mut res = Vec::new();
        for backup in self.transaction_backups.iter().sorted() {
            if backup.first_version > target_version {
                break;
            }
            ensure!(
                backup.first_version == next_ver,
                "Transactioon backup ranges not continuous, expecting version {}, got {}.",
                next_ver,
                backup.first_version,
            );

            if backup.last_version >= start_version {
                res.push(backup.clone());
            }

            next_ver = backup.last_version + 1;
        }

        Ok(res)
    }

    pub fn select_epoch_ending_backups(
        &self,
        target_version: Version,
    ) -> Result<Vec<EpochEndingBackupMeta>> {
        // This can be more flexible, but for now we assume and check backups are continuous in
        // range (which is always true when we backup from a single backup coordinator)
        let mut next_epoch = 0;
        let mut res = Vec::new();
        for backup in self.epoch_ending_backups.iter().sorted() {
            if backup.first_version > target_version {
                break;
            }

            ensure!(
                backup.first_epoch == next_epoch,
                "Epoch ending backup ranges not continuous, expecting epoch {}, got {}.",
                next_epoch,
                backup.first_epoch,
            );
            res.push(backup.clone());

            next_epoch = backup.last_epoch + 1;
        }

        Ok(res)
    }
}

impl From<Vec<Metadata>> for MetadataView {
    fn from(metadata_vec: Vec<Metadata>) -> Self {
        let mut epoch_ending_backups = Vec::new();
        let mut state_snapshot_backups = Vec::new();
        let mut transaction_backups = Vec::new();

        for meta in metadata_vec {
            match meta {
                Metadata::EpochEndingBackup(e) => epoch_ending_backups.push(e),
                Metadata::StateSnapshotBackup(s) => state_snapshot_backups.push(s),
                Metadata::TransactionBackup(t) => transaction_backups.push(t),
            }
        }

        Self {
            epoch_ending_backups,
            state_snapshot_backups,
            transaction_backups,
        }
    }
}

pub struct BackupStorageState {
    pub latest_epoch_ending_epoch: Option<u64>,
    pub latest_state_snapshot_version: Option<Version>,
    pub latest_transaction_version: Option<Version>,
}

impl fmt::Display for BackupStorageState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "latest_epoch_ending_epoch: {}, latest_state_snapshot_version: {}, latest_transaction_version: {}",
            self.latest_epoch_ending_epoch.as_ref().map_or("none".to_string(), u64::to_string),
            self.latest_state_snapshot_version.as_ref().map_or("none".to_string(), Version::to_string),
            self.latest_transaction_version.as_ref().map_or("none".to_string(), Version::to_string),
        )
    }
}

trait ParseOptionU64 {
    fn parse_option_u64(&self) -> Result<Option<u64>>;
}

impl ParseOptionU64 for Option<regex::Match<'_>> {
    fn parse_option_u64(&self) -> Result<Option<u64>> {
        let m = self.ok_or_else(|| anyhow!("No match."))?;
        if m.as_str() == "none" {
            Ok(None)
        } else {
            Ok(Some(m.as_str().parse()?))
        }
    }
}

impl FromStr for BackupStorageState {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let captures = regex::Regex::new(
            r"latest_epoch_ending_epoch: (none|\d+), latest_state_snapshot_version: (none|\d+), latest_transaction_version: (none|\d+)",
        )?.captures(s).ok_or_else(|| anyhow!("Not in BackupStorageState display format: {}", s))?;

        Ok(Self {
            latest_epoch_ending_epoch: captures.get(1).parse_option_u64()?,
            latest_state_snapshot_version: captures.get(2).parse_option_u64()?,
            latest_transaction_version: captures.get(3).parse_option_u64()?,
        })
    }
}
