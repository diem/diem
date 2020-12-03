// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod cache;
pub mod view;

use crate::storage::{FileHandle, ShellSafeName, TextLine};
use anyhow::Result;
use diem_types::transaction::Version;
use serde::{Deserialize, Serialize};
use std::convert::TryInto;

#[derive(Deserialize, Serialize)]
#[allow(clippy::enum_variant_names)] // to introduce: BackupperId, etc
pub(crate) enum Metadata {
    EpochEndingBackup(EpochEndingBackupMeta),
    StateSnapshotBackup(StateSnapshotBackupMeta),
    TransactionBackup(TransactionBackupMeta),
}

impl Metadata {
    pub fn new_epoch_ending_backup(
        first_epoch: u64,
        last_epoch: u64,
        first_version: Version,
        last_version: Version,
        manifest: FileHandle,
    ) -> Self {
        Self::EpochEndingBackup(EpochEndingBackupMeta {
            first_epoch,
            last_epoch,
            first_version,
            last_version,
            manifest,
        })
    }

    pub fn new_state_snapshot_backup(version: Version, manifest: FileHandle) -> Self {
        Self::StateSnapshotBackup(StateSnapshotBackupMeta { version, manifest })
    }

    pub fn new_transaction_backup(
        first_version: Version,
        last_version: Version,
        manifest: FileHandle,
    ) -> Self {
        Self::TransactionBackup(TransactionBackupMeta {
            first_version,
            last_version,
            manifest,
        })
    }

    pub fn name(&self) -> ShellSafeName {
        match self {
            Self::EpochEndingBackup(e) => {
                format!("epoch_ending_{}-{}.meta", e.first_epoch, e.last_epoch)
            }
            Self::StateSnapshotBackup(s) => format!("state_snapshot_ver_{}.meta", s.version),
            Self::TransactionBackup(t) => {
                format!("transaction_{}-{}.meta", t.first_version, t.last_version,)
            }
        }
        .try_into()
        .unwrap()
    }

    pub fn to_text_line(&self) -> Result<TextLine> {
        TextLine::new(&serde_json::to_string(self)?)
    }
}

#[derive(Clone, Deserialize, Serialize, Eq, PartialEq, Ord, PartialOrd)]
pub struct EpochEndingBackupMeta {
    pub first_epoch: u64,
    pub last_epoch: u64,
    pub first_version: Version,
    pub last_version: Version,
    pub manifest: FileHandle,
}

#[derive(Clone, Deserialize, Serialize, Eq, PartialEq, Ord, PartialOrd)]
pub struct StateSnapshotBackupMeta {
    pub version: Version,
    pub manifest: FileHandle,
}

#[derive(Clone, Deserialize, Serialize, Eq, PartialEq, Ord, PartialOrd)]
pub struct TransactionBackupMeta {
    pub first_version: Version,
    pub last_version: Version,
    pub manifest: FileHandle,
}
