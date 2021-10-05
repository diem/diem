// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{access_path::AccessPath, on_chain_config::ConfigID, transaction::Version};
use anyhow::Result;

// TODO combine with ConfigStorage
pub trait MoveStorage {
    /// Returns a vector of Move resources as serialized byte array
    /// Order of resources returned matches the order of `access_path`
    fn batch_fetch_resources(&self, access_paths: Vec<AccessPath>) -> Result<Vec<Vec<u8>>>;

    /// Returns a vector of Move resources as serialized byte array from a
    /// specified version of the database
    /// Order of resources returned matches the order of `access_path`
    fn batch_fetch_resources_by_version(
        &self,
        access_paths: Vec<AccessPath>,
        version: Version,
    ) -> Result<Vec<Vec<u8>>>;

    /// Returns an on-chain resource as serialized byte array from a
    /// specified version of the database.
    fn fetch_config_by_version(&self, config_id: ConfigID, version: Version) -> Result<Vec<u8>>;

    /// Get the version on the latest transaction info.
    fn fetch_synced_version(&self) -> Result<Version>;
}
