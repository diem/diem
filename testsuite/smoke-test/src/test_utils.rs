// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::test_environment::TestEnvironment;
use cli::client_proxy::ClientProxy;
use libra_config::config::{Identity, NodeConfig, SecureBackend};
use rust_decimal::{prelude::FromPrimitive, Decimal};
use std::{collections::BTreeMap, str::FromStr, string::ToString};

pub fn compare_balances(
    expected_balances: Vec<(f64, String)>,
    extracted_balances: Vec<String>,
) -> bool {
    if extracted_balances.len() != extracted_balances.len() {
        return false;
    }

    let extracted_balances_dec: BTreeMap<_, _> = extracted_balances
        .into_iter()
        .map(|balance_str| {
            let (currency_code, stripped_str) = if balance_str.ends_with("Coin1") {
                ("Coin1", balance_str.trim_end_matches("Coin1"))
            } else if balance_str.ends_with("Coin2") {
                ("Coin2", balance_str.trim_end_matches("Coin2"))
            } else if balance_str.ends_with("LBR") {
                ("LBR", balance_str.trim_end_matches("LBR"))
            } else {
                panic!("Unexpected currency type returned for balance")
            };
            (currency_code, Decimal::from_str(stripped_str).ok())
        })
        .collect();

    expected_balances
        .into_iter()
        .all(|(balance, currency_code)| {
            if let Some(extracted_balance) = extracted_balances_dec.get(currency_code.as_str()) {
                Decimal::from_f64(balance) == *extracted_balance
            } else {
                false
            }
        })
}

pub fn test_smoke_script(mut client_proxy: ClientProxy) {
    client_proxy.create_next_account(false).unwrap();
    client_proxy
        .mint_coins(&["mintb", "0", "10", "Coin1"], true)
        .unwrap();
    assert!(compare_balances(
        vec![(10.0, "Coin1".to_string())],
        client_proxy.get_balances(&["b", "0"]).unwrap(),
    ));
    client_proxy.create_next_account(false).unwrap();
    client_proxy
        .mint_coins(&["mintb", "1", "1", "Coin1"], true)
        .unwrap();
    client_proxy
        .transfer_coins(&["tb", "0", "1", "3", "Coin1"], true)
        .unwrap();
    assert!(compare_balances(
        vec![(7.0, "Coin1".to_string())],
        client_proxy.get_balances(&["b", "0"]).unwrap(),
    ));
    assert!(compare_balances(
        vec![(4.0, "Coin1".to_string())],
        client_proxy.get_balances(&["b", "1"]).unwrap(),
    ));
    client_proxy.create_next_account(false).unwrap();
    client_proxy
        .mint_coins(&["mintb", "2", "15", "Coin1"], true)
        .unwrap();
    assert!(compare_balances(
        vec![(15.0, "Coin1".to_string())],
        client_proxy.get_balances(&["b", "2"]).unwrap(),
    ));
}

pub fn setup_swarm_and_client_proxy(
    num_nodes: usize,
    client_port_index: usize,
) -> (TestEnvironment, ClientProxy) {
    let mut env = TestEnvironment::new(num_nodes);
    env.validator_swarm.launch();
    let ac_client = env.get_validator_client(client_port_index, None);
    (env, ac_client)
}

/// Loads the validator's storage backend from the given node config
pub fn load_backend_storage(node_config: &&NodeConfig) -> SecureBackend {
    if let Identity::FromStorage(storage_identity) =
        &node_config.validator_network.as_ref().unwrap().identity
    {
        storage_identity.backend.clone()
    } else {
        panic!("Couldn't load identity from storage");
    }
}

pub fn load_libra_root_storage(node_config: &NodeConfig) -> SecureBackend {
    if let Identity::FromStorage(storage_identity) =
        &node_config.validator_network.as_ref().unwrap().identity
    {
        match storage_identity.backend.clone() {
            SecureBackend::OnDiskStorage(mut config) => {
                config.namespace = Some("libra_root".to_string());
                SecureBackend::OnDiskStorage(config)
            }
            _ => unimplemented!("only support on-disk storage in smoke tests"),
        }
    } else {
        panic!("Couldn't load identity from storage");
    }
}
