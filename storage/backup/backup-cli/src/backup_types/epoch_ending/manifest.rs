// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::storage::FileHandle;
use anyhow::{ensure, Result};
use diem_types::waypoint::Waypoint;
use serde::{Deserialize, Serialize};

/// A chunk of an epoch ending backup manifest, representing the
/// [`first_epoch`, `last_epoch`] range (right side inclusive).
#[derive(Deserialize, Serialize)]
pub struct EpochEndingChunk {
    pub first_epoch: u64,
    pub last_epoch: u64,
    pub ledger_infos: FileHandle,
}

/// Epoch ending backup manifest, representing epoch ending information in the
/// [`first_epoch`, `last_epoch`] range (right side inclusive).
#[derive(Deserialize, Serialize)]
pub struct EpochEndingBackup {
    pub first_epoch: u64,
    pub last_epoch: u64,
    pub waypoints: Vec<Waypoint>,
    pub chunks: Vec<EpochEndingChunk>,
}

impl EpochEndingBackup {
    pub fn verify(&self) -> Result<()> {
        // check number of waypoints
        ensure!(
            self.first_epoch <= self.last_epoch
                && self.last_epoch - self.first_epoch + 1 == self.waypoints.len() as u64,
            "Malformed manifest. first epoch: {}, last epoch {}, num waypoints {}",
            self.first_epoch,
            self.last_epoch,
            self.waypoints.len(),
        );

        // check chunk ranges
        ensure!(!self.chunks.is_empty(), "No chunks.");
        let mut next_epoch = self.first_epoch;
        for chunk in &self.chunks {
            ensure!(
                chunk.first_epoch == next_epoch,
                "Chunk ranges not continuous. Expected first epoch: {}, actual: {}.",
                next_epoch,
                chunk.first_epoch,
            );
            ensure!(
                chunk.last_epoch >= chunk.first_epoch,
                "Chunk range invalid. [{}, {}]",
                chunk.first_epoch,
                chunk.last_epoch,
            );
            next_epoch = chunk.last_epoch + 1;
        }

        // check last epoch in chunk matches manifest
        ensure!(
            next_epoch - 1 == self.last_epoch, // okay to -1 because chunks is not empty.
            "Last epoch in chunks: {}, in manifest: {}",
            next_epoch - 1,
            self.last_epoch,
        );

        Ok(())
    }
}
