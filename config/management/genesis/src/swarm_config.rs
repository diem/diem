// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::validator_builder::ValidatorBuilder;
use anyhow::Result;
use diem_config::config::{NodeConfig, OnDiskStorageConfig};
use diem_crypto::ed25519::Ed25519PrivateKey;
use diem_global_constants::{DIEM_ROOT_KEY, TREASURY_COMPLIANCE_KEY};
use diem_secure_storage::{CryptoStorage, OnDiskStorage};
use diem_types::waypoint::Waypoint;
use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

pub trait BuildSwarm {
    /// Generate the configs for a swarm
    fn build_swarm<R>(&self, rng: R) -> Result<(Vec<NodeConfig>, Ed25519PrivateKey)>
    where
        R: ::rand::RngCore + ::rand::CryptoRng;
}

pub struct SwarmConfig {
    pub config_files: Vec<PathBuf>,
    pub diem_root_key_path: PathBuf,
    pub root_storage: OnDiskStorageConfig,
    pub waypoint: Waypoint,
}

impl SwarmConfig {
    pub fn build_with_rng<T, R>(config_builder: &T, output_dir: &Path, rng: R) -> Result<Self>
    where
        T: BuildSwarm,
        R: ::rand::RngCore + ::rand::CryptoRng,
    {
        let (mut configs, diem_root_key) = config_builder.build_swarm(rng)?;
        let mut config_files = vec![];

        for (index, config) in configs.iter_mut().enumerate() {
            let node_dir = output_dir.join(index.to_string());
            std::fs::create_dir_all(&node_dir)?;

            let node_path = node_dir.join("node.yaml");
            config.set_data_dir(node_dir);
            config.save(&node_path)?;
            config_files.push(node_path);
        }

        let diem_root_key_path = output_dir.join("mint.key");
        let serialized_keys = bcs::to_bytes(&diem_root_key)?;
        let mut key_file = File::create(&diem_root_key_path)?;
        key_file.write_all(&serialized_keys)?;

        let mut root_storage_config = OnDiskStorageConfig::default();
        root_storage_config.path = output_dir.join("root-storage.json");
        let mut root_storage = OnDiskStorage::new(root_storage_config.path());
        root_storage.import_private_key(DIEM_ROOT_KEY, diem_root_key)?;
        let diem_root_key = root_storage.export_private_key(DIEM_ROOT_KEY).unwrap();
        root_storage.import_private_key(TREASURY_COMPLIANCE_KEY, diem_root_key)?;

        Ok(SwarmConfig {
            config_files,
            diem_root_key_path,
            root_storage: root_storage_config,
            waypoint: configs[0].base.waypoint.waypoint(),
        })
    }

    pub fn build<T: BuildSwarm>(config_builder: &T, output_dir: &Path) -> Result<Self> {
        Self::build_with_rng(config_builder, output_dir, rand::rngs::OsRng)
    }
}

impl BuildSwarm for ValidatorBuilder {
    fn build_swarm(&self) -> Result<(Vec<NodeConfig>, Ed25519PrivateKey)> {
        let (root_keys, validators) = self.clone().build(rand::rngs::OsRng)?;

        Ok((
            validators.into_iter().map(|v| v.config).collect(),
            root_keys.root_key,
        ))
    }
}
