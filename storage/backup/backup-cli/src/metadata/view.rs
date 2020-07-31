// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::metadata::{
    EpochEndingBackupMeta, Metadata, StateSnapshotBackupMeta, TransactionBackupMeta,
};
use anyhow::{ensure, Result};
use itertools::Itertools;
use libra_types::transaction::Version;
use serde::export::Formatter;
use std::fmt::Display;

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

            res.push(backup.clone());

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

impl Display for BackupStorageState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "latest_epoch_ending_epoch: {}, latest_state_snapshot_version: {}, latest_transaction_version: {}",
            self.latest_epoch_ending_epoch.as_ref().map_or("none".to_string(), u64::to_string),
            self.latest_state_snapshot_version.as_ref().map_or("none".to_string(), Version::to_string),
            self.latest_transaction_version.as_ref().map_or("none".to_string(), Version::to_string),
        )
    }
}
