// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::utils;
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, path::PathBuf};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct StorageConfig {
    pub address: SocketAddr,
    /// Use a different address name for non-GRPC storage serivce.
    /// Will be renamed as `address` once GRPC service is deprecated.
    pub simple_address: SocketAddr,
    pub backup_service_port: u16,
    pub dir: PathBuf,
    pub grpc_max_receive_len: Option<i32>,
    /// None disables pruning. The windows is in number of versions, consider system tps
    /// (transaction per second) when calculating proper window.
    pub prune_window: Option<u64>,
    #[serde(skip)]
    data_dir: PathBuf,
}

impl Default for StorageConfig {
    fn default() -> StorageConfig {
        StorageConfig {
            address: "127.0.0.1:6184".parse().unwrap(),
            simple_address: "127.0.0.1:6666".parse().unwrap(),
            backup_service_port: 7777,
            dir: PathBuf::from("libradb/db"),
            grpc_max_receive_len: Some(100_000_000),
            prune_window: None,
            data_dir: PathBuf::from("/opt/libra/data/common"),
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
        self.simple_address.set_port(utils::get_available_port());
        self.backup_service_port = utils::get_available_port();
    }
}
