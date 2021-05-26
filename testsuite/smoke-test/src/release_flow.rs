// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::test_utils::{diem_swarm_utils::get_json_rpc_url, setup_swarm_and_client_proxy};
use diem_framework_releases::current_modules_with_blobs;
use diem_types::{
    chain_id::ChainId, on_chain_config::DIEM_MAX_KNOWN_VERSION, transaction::TransactionPayload,
};
use diem_validator_interface::{DiemValidatorInterface, JsonRpcDebuggerInterface};
use diem_writeset_generator::{
    create_release, release_flow::test_utils::release_modules, verify_release,
};
use std::collections::BTreeMap;

#[test]
fn test_move_release_flow() {
    let (env, mut client) = setup_swarm_and_client_proxy(1, 0);
    let url = get_json_rpc_url(&env.validator_swarm, 0);
    let validator_interface = JsonRpcDebuggerInterface::new(&url).unwrap();

    let chain_id = ChainId::test();
    let old_modules = current_modules_with_blobs()
        .into_iter()
        .map(|(bytes, modules)| (bytes.clone(), modules.clone()))
        .collect::<Vec<_>>();

    let release_modules = release_modules();

    // Execute some random transactions to make sure a new block is created.
    client.create_next_account(false).unwrap();
    client.mint_coins(&["mb", "0", "100", "XUS"], true).unwrap();

    // With no artifact for TESTING, creating a release should fail.
    assert!(create_release(chain_id, url.clone(), 1, false, &release_modules, None, "").is_err());

    // Generate the first release package. It should pass and verify.
    let payload_1 =
        create_release(chain_id, url.clone(), 1, true, &release_modules, None, "").unwrap();
    // Verifying the generated payload against release modules should pass.
    verify_release(chain_id, url.clone(), &payload_1, &release_modules, false).unwrap();
    // Verifying the generated payload against older modules should pass due to hash mismatch.
    assert!(verify_release(chain_id, url.clone(), &payload_1, &old_modules, false).is_err());

    // Commit the release
    client
        .association_transaction_with_local_diem_root_account(
            TransactionPayload::WriteSet(payload_1.clone()),
            true,
        )
        .unwrap();

    let latest_version = validator_interface.get_latest_version().unwrap();
    let remote_modules = validator_interface
        .get_diem_framework_modules_by_version(latest_version)
        .unwrap();
    // Assert the remote modules are the same as the release modules.
    assert_eq!(
        remote_modules
            .iter()
            .map(|m| (m.self_id(), m))
            .collect::<BTreeMap<_, _>>(),
        release_modules
            .iter()
            .map(|(_, m)| (m.self_id(), m))
            .collect::<BTreeMap<_, _>>(),
    );

    // Execute some random transactions to make sure a new block is created.
    client.mint_coins(&["mb", "0", "100", "XUS"], true).unwrap();

    let latest_version = validator_interface.get_latest_version().unwrap();
    // Now that we have artifact file checked in, we can get rid of the first_release flag
    // Let's flip the modules back to the older version
    let payload_2 = create_release(
        chain_id,
        url.clone(),
        latest_version,
        false,
        &old_modules,
        Some(DIEM_MAX_KNOWN_VERSION.major + 1),
        "",
    )
    .unwrap();
    // Verifying the generated payload against release modules should pass.
    verify_release(chain_id, url.clone(), &payload_2, &old_modules, false).unwrap();
    // Verifying the old payload would fail.
    assert!(verify_release(chain_id, url.clone(), &payload_1, &old_modules, false).is_err());
    assert!(verify_release(chain_id, url.clone(), &payload_1, &release_modules, false).is_err());

    // Cannot create a release with an older version.
    assert!(create_release(
        chain_id,
        url,
        latest_version - 1,
        false,
        &old_modules,
        None,
        ""
    )
    .is_err());

    // Commit the release
    client
        .association_transaction_with_local_diem_root_account(
            TransactionPayload::WriteSet(payload_2),
            true,
        )
        .unwrap();

    let latest_version = validator_interface.get_latest_version().unwrap();
    let remote_modules = validator_interface
        .get_diem_framework_modules_by_version(latest_version)
        .unwrap();
    // Assert the remote module is the same as the release modules.

    assert_eq!(
        client.client.get_metadata().unwrap().diem_version,
        Some(DIEM_MAX_KNOWN_VERSION.major + 1)
    );

    assert_eq!(
        remote_modules
            .iter()
            .map(|m| (m.self_id(), m))
            .collect::<BTreeMap<_, _>>(),
        old_modules
            .iter()
            .map(|(_, m)| (m.self_id(), m))
            .collect::<BTreeMap<_, _>>(),
    );
}
