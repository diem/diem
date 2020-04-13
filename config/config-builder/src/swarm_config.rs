// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use libra_config::config::NodeConfig;
use libra_crypto::{ed25519, test_utils::KeyPair};
use std::{fs::File, io::Write, path::PathBuf};

pub trait BuildSwarm {
    /// Generate the configs for a swarm
    fn build_swarm(&self) -> Result<(Vec<NodeConfig>, ed25519::SigningKey)>;
}

pub struct SwarmConfig {
    pub config_files: Vec<PathBuf>,
    pub faucet_key_path: PathBuf,
}

impl SwarmConfig {
    pub fn build<T: BuildSwarm>(config_builder: &T, output_dir: &PathBuf) -> Result<Self> {
        let (mut configs, faucet_key) = config_builder.build_swarm()?;
        let mut config_files = vec![];

        for (index, config) in configs.iter_mut().enumerate() {
            let node_dir = output_dir.join(index.to_string());
            std::fs::create_dir_all(&node_dir)?;

            let node_path = node_dir.join("node.config.toml");
            config.set_data_dir(node_dir);
            config.save(&node_path)?;
            config_files.push(node_path);
        }

        let faucet_key_path = output_dir.join("mint.key");
        let faucet_keypair =
            KeyPair::<ed25519::SigningKey, ed25519::VerifyingKey>::from(faucet_key);
        let serialized_keys = lcs::to_bytes(&faucet_keypair)?;
        let mut key_file = File::create(&faucet_key_path)?;
        key_file.write_all(&serialized_keys)?;

        Ok(SwarmConfig {
            config_files,
            faucet_key_path,
        })
    }
}
