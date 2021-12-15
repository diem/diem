// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    smoke_test_environment::new_local_swarm,
    test_utils::{assert_balance, create_and_fund_account},
};
use diem_framework_releases::current_modules_with_blobs;
use diem_sdk::transaction_builder::Currency;
use diem_types::{on_chain_config::DIEM_MAX_KNOWN_VERSION, transaction::TransactionPayload};
use diem_validator_interface::{DiemValidatorInterface, JsonRpcDebuggerInterface};
use diem_writeset_generator::{
    create_release, release_flow::test_utils::release_modules, verify_release,
};
use forge::{Node, NodeExt, Swarm};
use std::collections::BTreeMap;

#[tokio::test]
async fn test_move_release_flow() {
    let mut swarm = new_local_swarm(1).await;
    let transaction_factory = swarm.chain_info().transaction_factory();
    let chain_id = swarm.chain_id();
    let validator = swarm.validators().next().unwrap();
    let json_rpc_endpoint = validator.json_rpc_endpoint();
    let url = json_rpc_endpoint.to_string();
    let client = validator.rest_client();

    let validator_interface = JsonRpcDebuggerInterface::new(&url).unwrap();

    let old_modules = current_modules_with_blobs()
        .into_iter()
        .map(|(bytes, modules)| (bytes.clone(), modules.clone()))
        .collect::<Vec<_>>();

    let release_modules = release_modules();

    // Execute some random transactions to make sure a new block is created.
    let account = create_and_fund_account(&mut swarm, 100).await;

    // With no artifact for TESTING, creating a release should fail.
    assert!(create_release(chain_id, url.clone(), 2, false, &release_modules, None, "").is_err());

    // Generate the first release package. It should pass and verify.
    let payload_1 =
        create_release(chain_id, url.clone(), 2, true, &release_modules, None, "").unwrap();
    // Verifying the generated payload against release modules should pass.
    verify_release(chain_id, url.clone(), &payload_1, &release_modules, false).unwrap();
    // Verifying the generated payload against older modules should pass due to hash mismatch.
    assert!(verify_release(chain_id, url.clone(), &payload_1, &old_modules, false).is_err());

    // Commit the release
    let txn = swarm
        .chain_info()
        .root_account
        .sign_with_transaction_builder(
            transaction_factory.payload(TransactionPayload::WriteSet(payload_1.clone())),
        );

    client.submit_and_wait(&txn).await.unwrap();

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
    swarm
        .chain_info()
        .fund(Currency::XUS, account.address(), 100)
        .await
        .unwrap();
    assert_balance(&client, &account, 200).await;

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
    let txn = swarm
        .chain_info()
        .root_account
        .sign_with_transaction_builder(
            transaction_factory.payload(TransactionPayload::WriteSet(payload_2)),
        );
    client.submit_and_wait(&txn).await.unwrap();

    let latest_version = validator_interface.get_latest_version().unwrap();
    let remote_modules = validator_interface
        .get_diem_framework_modules_by_version(latest_version)
        .unwrap();
    // Assert the remote module is the same as the release modules.

    assert_eq!(
        *client
            .get_diem_version()
            .await
            .unwrap()
            .into_inner()
            .payload
            .major
            .inner(),
        DIEM_MAX_KNOWN_VERSION.major + 1
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
