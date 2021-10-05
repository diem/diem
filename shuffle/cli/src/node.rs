// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{shared, shared::Home};
use anyhow::Result;
use diem_config::config::NodeConfig;
use diem_types::{
    account_address::AccountAddress, chain_id::ChainId, on_chain_config::VMPublishingOption,
};
use std::path::{Path, PathBuf};

const LAZY_ENABLED: bool = true;

pub fn handle(home: &Home, genesis: Option<String>) -> Result<()> {
    home.generate_shuffle_path_if_nonexistent()?;
    home.write_default_networks_config_into_toml_if_nonexistent()?;
    if !home.get_node_config_path().is_dir() {
        println!(
            "Creating node config in {}",
            home.get_shuffle_path().display()
        );

        create_node(home, genesis)
    } else {
        println!(
            "Accessing node config in {}",
            home.get_shuffle_path().display()
        );
        if genesis.is_some() {
            return Err(
                anyhow::anyhow!(
                    "Unable to set genesis on an already created node. rm -rf ~/.shuffle to recreate node; you will lose state"
                )
            );
        }

        start_node(home)
    }
}

fn create_node(home: &Home, genesis: Option<String>) -> Result<()> {
    let publishing_option = VMPublishingOption::open();
    let genesis_modules = genesis_modules_from_path(&genesis)?;
    diem_node::load_test_environment(
        Some(PathBuf::from(home.get_node_config_path())),
        false,
        LAZY_ENABLED,
        Some(publishing_option),
        genesis_modules,
        None,
        rand::rngs::OsRng,
    );
    Ok(())
}

fn start_node(home: &Home) -> Result<()> {
    println!("\tLog file: {:?}", home.get_validator_log_path());
    println!("\tConfig path: {:?}", home.get_validator_config_path());
    println!("\tDiem root key path: {:?}", home.get_root_key_path());
    println!("\tWaypoint: {}", home.read_genesis_waypoint()?);
    println!("\tChainId: {}", ChainId::test());
    let config = NodeConfig::load(home.get_validator_config_path()).unwrap();
    diem_node::print_api_config(&config, LAZY_ENABLED);

    println!("Diem is running, press ctrl-c to exit");
    println!();

    diem_node::start(&config, Some(PathBuf::from(home.get_validator_log_path())));
    Ok(())
}

fn genesis_modules_from_path(genesis: &Option<String>) -> Result<Vec<Vec<u8>>> {
    let path = match genesis {
        None => return Ok(diem_framework_releases::current_module_blobs().to_vec()),
        Some(path_str) => Path::new(path_str),
    };

    println!("Using custom genesis: {}", path.display());
    let mut genesis_modules: Vec<Vec<u8>> = Vec::new();
    let compiled_package =
        shared::build_move_package(path, &AccountAddress::from_hex_literal("0x1")?)?;
    for module in compiled_package
        .transitive_compiled_modules()
        .compute_dependency_graph()
        .compute_topological_order()?
    {
        println!("Genesis Module: {}", module.self_id());
        let mut binary = vec![];
        module.serialize(&mut binary)?;
        genesis_modules.push(binary);
    }

    Ok(genesis_modules)
}
