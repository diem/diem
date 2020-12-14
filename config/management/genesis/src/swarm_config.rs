// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use diem_config::config::NodeConfig;
use diem_crypto::ed25519::Ed25519PrivateKey;
use diem_types::waypoint::Waypoint;
use std::{fs::File, io::Write, path::PathBuf};

pub trait BuildSwarm {
    /// Generate the configs for a swarm
    fn build_swarm(&self) -> Result<(Vec<NodeConfig>, Ed25519PrivateKey)>;
}

pub struct SwarmConfig {
    pub config_files: Vec<PathBuf>,
    pub diem_root_key_path: PathBuf,
    pub waypoint: Waypoint,
}

impl SwarmConfig {
    pub fn build<T: BuildSwarm>(config_builder: &T, output_dir: &PathBuf) -> Result<Self> {
        let (mut configs, diem_root_key) = config_builder.build_swarm()?;
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

        Ok(SwarmConfig {
            config_files,
            diem_root_key_path,
            waypoint: configs[0].base.waypoint.waypoint(),
        })
    }
}
