// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{Factory, Result, Swarm, Version};
use anyhow::format_err;
use rand::rngs::StdRng;
use std::{env, fs::File, io::Read, num::NonZeroUsize, path::PathBuf};
use tokio::runtime::Runtime;

mod cluster_helper;
mod node;
mod swarm;

pub use cluster_helper::*;
pub use node::K8sNode;
pub use swarm::*;

use diem_sdk::crypto::ed25519::ED25519_PRIVATE_KEY_LENGTH;
use diem_secure_storage::{CryptoStorage, KVStorage, VaultStorage};

const DEFAULT_TESTNET_IMAGE_TAG: &str = "devnet";

pub struct K8sFactory {
    root_key: [u8; ED25519_PRIVATE_KEY_LENGTH],
    treasury_compliance_key: [u8; ED25519_PRIVATE_KEY_LENGTH],
    helm_repo: String,
    image_tag: String,
    base_image_tag: String,
}

impl K8sFactory {
    pub fn new(helm_repo: String, image_tag: String, base_image_tag: String) -> Result<K8sFactory> {
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
            helm_repo,
            image_tag,
            base_image_tag,
        })
    }
}

impl Factory for K8sFactory {
    fn versions<'a>(&'a self) -> Box<dyn Iterator<Item = Version> + 'a> {
        Box::new(std::iter::once(Version::new(0, self.image_tag.clone())))
    }

    fn launch_swarm(
        &self,
        _rng: &mut StdRng,
        node_num: NonZeroUsize,
        version: &Version,
    ) -> Result<Box<dyn Swarm>> {
        let _ = clean_k8s_cluster(
            self.helm_repo.clone(),
            node_num.get(),
            format!("{}", version),
            DEFAULT_TESTNET_IMAGE_TAG.to_string(),
            true,
        );
        let rt = Runtime::new().unwrap();
        let swarm = rt
            .block_on(K8sSwarm::new(
                &self.root_key,
                &self.treasury_compliance_key,
                &self.helm_repo,
                &self.image_tag,
                &self.base_image_tag,
            ))
            .unwrap();
        Ok(Box::new(swarm))
    }
}
