// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use libra_config::config::NodeConfig;
use libra_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    test_utils::KeyPair,
};
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

pub struct SwarmConfig {
    pub config_files: Vec<PathBuf>,
    pub faucet_key_path: PathBuf,
}

impl SwarmConfig {
    pub fn build<B: BuildSwarm, P: AsRef<Path>>(
        builder: &B,
        config_path: P,
    ) -> Result<SwarmConfig> {
        let (mut configs, faucet_key) = builder.build_swarm()?;
        let mut config_files = Vec::new();
        let config_dir = config_path.as_ref().to_path_buf();

        for (index, config) in configs.iter_mut().enumerate() {
            let node_dir = config_dir.join(format!("{}", index));
            let node_path = node_dir.join("node.config.toml");
            fs::create_dir_all(&node_dir)?;
            config.set_data_dir(node_dir);
            config.save(&node_path)?;
            config_files.push(node_path);
        }

        let faucet_key_path = config_dir.join("mint.key");
        let faucet_keypair = KeyPair::<Ed25519PrivateKey, Ed25519PublicKey>::from(faucet_key);
        let serialized_key = lcs::to_bytes(&faucet_keypair)?;
        let mut faucet_key_file = File::create(&faucet_key_path)?;
        faucet_key_file.write_all(&serialized_key)?;

        Ok(Self {
            config_files,
            faucet_key_path,
        })
    }
}

pub trait BuildSwarm {
    fn build_swarm(&self) -> Result<(Vec<NodeConfig>, Ed25519PrivateKey)>;
}
