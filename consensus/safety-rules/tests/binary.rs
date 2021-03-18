// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_config::{
    config::{NodeConfig, PersistableConfig, RemoteService, SafetyRulesService},
    utils,
};
use diem_types::validator_signer::ValidatorSigner;
use safety_rules::{test_utils, SafetyRulesManager};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

const BINARY: &str = env!("CARGO_BIN_EXE_safety-rules");

#[test]
fn test_consensus_state() {
    let mut config = NodeConfig::random().consensus.safety_rules;
    let test_config = config.test.as_mut().unwrap();
    let private_key = test_config.consensus_key.as_ref().unwrap().private_key();
    let signer = ValidatorSigner::new(test_config.author, private_key);
    let waypoint = test_utils::validator_signers_to_waypoint(&[&signer]);
    test_config.waypoint = Some(waypoint);

    let server_port = utils::get_available_port();
    let server_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), server_port).into();
    config.service = SafetyRulesService::Process(RemoteService { server_address });

    let config_path = diem_temppath::TempPath::new();
    config_path.create_as_file().unwrap();
    config.save_config(config_path.path()).unwrap();

    let mut command = std::process::Command::new(BINARY);
    command
        .arg(config_path.path())
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit());
    let mut child = command.spawn().unwrap();

    let safety_rules_manager = SafetyRulesManager::new(&config);
    let mut safety_rules = safety_rules_manager.client();
    let consensus_state = safety_rules.consensus_state();

    // Ensure the safety-rules subprocess is killed whether the test passes or fails.
    // Not doing this would result in a zombie process.
    child.kill().expect("could not kill safety-rules process");
    child
        .wait()
        .expect("could not wait on safety-rules process");
    consensus_state.unwrap();
}
