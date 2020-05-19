// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use executor_test_helpers::{
    extract_signer, gen_block_id, gen_ledger_info_with_sigs, get_test_signed_transaction,
};
use executor_types::BlockExecutor;
use libra_crypto::{ed25519::*, test_utils::TEST_SEED, PrivateKey, Uniform};
use libra_types::{
    account_config::{association_address, lbr_type_tag},
    transaction::authenticator::AuthenticationKey,
};
use rand::SeedableRng;
use transaction_builder::encode_create_account_script;

pub fn run_test_suite(func: fn() -> Box<dyn BlockExecutor>) {
    let (mut config, genesis_key) = config_builder::test_config();
    let signer = extract_signer(&mut config);
    let mut executor = func();
    let parent_block_id = executor.committed_block_id().unwrap();

    let seed = [1u8; 32];
    // TEST_SEED is also used to generate a random validator set in get_test_config. Each account
    // in this random validator set gets created in genesis. If one of {account1, account2,
    // account3} already exists in genesis, the code below will fail.
    assert!(seed != TEST_SEED);
    let mut rng = ::rand::rngs::StdRng::from_seed(seed);

    let privkey1 = Ed25519PrivateKey::generate(&mut rng);
    let pubkey1 = privkey1.public_key();
    let account1_auth_key = AuthenticationKey::ed25519(&pubkey1);
    let account1 = account1_auth_key.derived_address();

    let genesis_account = association_address();

    // Create account1 with 2M coins.
    let txn1 = get_test_signed_transaction(
        genesis_account,
        /* sequence_number = */ 1,
        genesis_key.clone(),
        genesis_key.public_key(),
        Some(encode_create_account_script(
            lbr_type_tag(),
            &account1,
            account1_auth_key.prefix().to_vec(),
            2_000_000,
        )),
    );

    let block1 = vec![txn1];
    let block1_id = gen_block_id(1);

    let output1 = executor
        .execute_block((block1_id, block1), parent_block_id)
        .unwrap();

    let ledger_info_with_sigs = gen_ledger_info_with_sigs(1, output1, block1_id, vec![&signer]);
    let (_, _) = executor
        .commit_blocks(vec![block1_id], ledger_info_with_sigs)
        .unwrap();
}
