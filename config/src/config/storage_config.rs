// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::utils;
use serde::{Deserialize, Serialize};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
};

/// Port selected RocksDB options for tuning underlying rocksdb instance of LibraDB.
/// see https://github.com/facebook/rocksdb/blob/master/include/rocksdb/options.h
/// for detailed explanations.
#[derive(Copy, Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct RocksdbConfig {
    pub max_open_files: i32,
    pub max_total_wal_size: u64,
}

impl Default for RocksdbConfig {
    fn default() -> Self {
        Self {
            // Set max_open_files to 10k instead of -1 to avoid keep-growing memory in corridance
            // with the number of files.
            max_open_files: 10_000,
            // For now we set the max total WAL size to be 1G. This config can be useful when column
            // families are updated at non-uniform frequencies.
            max_total_wal_size: 1u64 << 30,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct StorageConfig {
    pub address: SocketAddr,
    pub backup_service_address: SocketAddr,
    pub dir: PathBuf,
    pub grpc_max_receive_len: Option<i32>,
    /// None disables pruning. The windows is in number of versions, consider system tps
    /// (transaction per second) when calculating proper window.
    pub prune_window: Option<u64>,
    #[serde(skip)]
    data_dir: PathBuf,
    /// Read, Write, Connect timeout for network operations in milliseconds
    pub timeout_ms: u64,
    /// Rocksdb-specific configurations
    pub rocksdb_config: RocksdbConfig,
}

impl Default for StorageConfig {
    fn default() -> StorageConfig {
        StorageConfig {
            address: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 6666),
            backup_service_address: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 6186),
            dir: PathBuf::from("db"),
            grpc_max_receive_len: Some(100_000_000),
            // ~50GB state tree history (about 1 day at 100 tps)
            prune_window: Some(10_000_000),
            data_dir: PathBuf::from("/opt/libra/data"),
            // Default read/write/connection timeout, in milliseconds
            timeout_ms: 30_000,
            rocksdb_config: RocksdbConfig::default(),
        }
    }
}

impl StorageConfig {
    pub fn dir(&self) -> PathBuf {
        if self.dir.is_relative() {
            self.data_dir.join(&self.dir)
        } else {
            self.dir.clone()
        }
    }

    pub fn set_data_dir(&mut self, data_dir: PathBuf) {
        self.data_dir = data_dir;
    }

    pub fn randomize_ports(&mut self) {
        self.address.set_port(utils::get_available_port());
        self.backup_service_address
            .set_port(utils::get_available_port());
    }
}
