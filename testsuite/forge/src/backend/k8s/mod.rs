// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{Factory, Swarm};
use anyhow::{format_err, Error};
use std::{env, fs::File, io::Read, path::PathBuf};
use tokio::runtime::Runtime;

mod node;
mod swarm;
pub use node::K8sNode;
pub use swarm::*;

use diem_sdk::crypto::ed25519::ED25519_PRIVATE_KEY_LENGTH;
use diem_secure_storage::{CryptoStorage, KVStorage, VaultStorage};

pub struct K8sFactory {
    root_key: [u8; ED25519_PRIVATE_KEY_LENGTH],
    treasury_compliance_key: [u8; ED25519_PRIVATE_KEY_LENGTH],
}

impl K8sFactory {
    pub fn new() -> std::result::Result<K8sFactory, Error> {
        let vault_addr = env::var("VAULT_ADDR")
            .map_err(|_| format_err!("Expected environment variable VAULT_ADDR"))?;
        let vault_cacert = env::var("VAULT_CACERT")
            .map_err(|_| format_err!("Expected environment variable VAULT_CACERT"))?;
        let vault_token = env::var("VAULT_TOKEN")
            .map_err(|_| format_err!("Expected environment variable VAULT_TOKEN"))?;

        let vault_cacert_path = PathBuf::from(vault_cacert.clone());

        let mut vault_cacert_file = File::open(vault_cacert_path)
            .map_err(|_| format_err!("Failed to open VAULT_CACERT file at {}", &vault_cacert))?;
        let mut vault_cacert_contents = String::new();
        vault_cacert_file
            .read_to_string(&mut vault_cacert_contents)
            .map_err(|_| format_err!("Failed to read VAULT_CACERT file at {}", &vault_cacert))?;

        let vault = VaultStorage::new(
            vault_addr,
            vault_token,
            Some(vault_cacert_contents),
            None,
            false,
            None,
            None,
        );
        vault.available()?;
        let root_key = vault
            .export_private_key("diem__diem_root")
            .unwrap()
            .to_bytes();
        let treasury_compliance_key = vault
            .export_private_key("diem__treasury_compliance")
            .unwrap()
            .to_bytes();

        Ok(Self {
            root_key,
            treasury_compliance_key,
        })
    }
}

impl Factory for K8sFactory {
    fn launch_swarm(&self, _node_num: usize) -> Box<dyn Swarm> {
        let rt = Runtime::new().unwrap();
        let swarm = rt
            .block_on(K8sSwarm::new(&self.root_key, &self.treasury_compliance_key))
            .unwrap();
        Box::new(swarm)
    }
}
