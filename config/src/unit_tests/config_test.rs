// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use std::fs;

static EXPECTED_SINGLE_NODE_CONFIG: &[u8] = include_bytes!("../../data/configs/single.node.config.toml");

#[test]
fn verify_test_config() {
    // This test likely failed because there was a breaking change in the NodeConfig. It may be
    // desirable to reverse the change or to change the test config and potentially documentation.
    let _ = NodeConfigHelpers::get_single_node_test_config(false);
    let _ = NodeConfig::parse(&String::from_utf8_lossy(EXPECTED_SINGLE_NODE_CONFIG)).expect("Error parsing expected single node config");
}

#[test]
fn verify_all_configs() {
    // This test verifies that all configs in data/config are valid
    let paths = fs::read_dir("data/configs").expect("cannot read config dir");

    for path in paths {
        let config_path = path.unwrap().path();
        let config_path_str = config_path.to_str().unwrap();
        if config_path_str.ends_with(".toml") {
            println!("Loading {}", config_path_str);
            let _ = NodeConfig::load(config_path_str).expect("NodeConfig");
        } else {
            println!("Invalid file {} for verifying", config_path_str);
        }
    }
}
