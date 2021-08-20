// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use compiler::Compiler;
use diem_crypto::{ed25519::*, PrivateKey, Uniform};
use diem_transaction_builder::stdlib::{
    encode_peer_to_peer_with_metadata_script, encode_set_validator_config_and_reconfigure_script,
};
use diem_types::{
    account_config::{diem_root_address, treasury_compliance_account_address, xus_tag},
    account_state::AccountState,
    block_metadata::BlockMetadata,
    transaction::{Script, Transaction, TransactionPayload, WriteSetPayload},
    trusted_state::TrustedState,
    validator_signer::ValidatorSigner,
};
use executor_test_helpers::{
    gen_block_id, gen_block_metadata, gen_ledger_info_with_sigs, get_test_signed_transaction,
    integration_test_impl::{
        create_db_and_executor, test_execution_with_storage_impl, verify_committed_txn_status,
    },
};
use executor_types::BlockExecutor;
use std::convert::TryFrom;

#[test]
fn test_genesis() {
    let path = diem_temppath::TempPath::new();
    path.create_as_dir().unwrap();
    let genesis = vm_genesis::test_genesis_transaction();
    let (_, db, _executor, waypoint) = create_db_and_executor(path.path(), &genesis);

    let trusted_state = TrustedState::from_epoch_waypoint(waypoint);
    let initial_accumulator = db
        .reader
        .get_accumulator_summary(trusted_state.version())
        .unwrap();
    let state_proof = db.reader.get_state_proof(trusted_state.version()).unwrap();

    trusted_state
        .verify_and_ratchet(&state_proof, Some(&initial_accumulator))
        .unwrap();
    let li = state_proof.latest_ledger_info();
    assert_eq!(li.version(), 0);

    let diem_root_account = db
        .reader
        .get_account_state_with_proof(diem_root_address(), 0, 0)
        .unwrap();
    diem_root_account
        .verify(li, 0, diem_root_address())
        .unwrap();
}

#[test]
fn test_reconfiguration() {
    // When executing a transaction emits a validator set change,
    // storage should propagate the new validator set

    let path = diem_temppath::TempPath::new();
    path.create_as_dir().unwrap();
    let (genesis, validators) = vm_genesis::test_genesis_change_set_and_validators(Some(1));
    let genesis_key = &vm_genesis::GENESIS_KEYPAIR.0;
    let genesis_txn = Transaction::GenesisTransaction(WriteSetPayload::Direct(genesis));
    let (_, db, executor, _waypoint) = create_db_and_executor(path.path(), &genesis_txn);
    let parent_block_id = executor.committed_block_id();
    let signer = ValidatorSigner::new(validators[0].data.address, validators[0].key.clone());
    let validator_account = signer.author();

    // test the current keys in the validator's account equals to the key in the validator set
    let state_proof = db.reader.get_state_proof(0).unwrap();
    let current_version = state_proof.latest_ledger_info().version();
    let validator_account_state_with_proof = db
        .reader
        .get_account_state_with_proof(validator_account, current_version, current_version)
        .unwrap();
    let diem_root_account_state_with_proof = db
        .reader
        .get_account_state_with_proof(diem_root_address(), current_version, current_version)
        .unwrap();
    assert_eq!(
        AccountState::try_from(&diem_root_account_state_with_proof.blob.unwrap())
            .unwrap()
            .get_validator_set()
            .unwrap()
            .unwrap()
            .payload()[0]
            .consensus_public_key(),
        &AccountState::try_from(&validator_account_state_with_proof.blob.unwrap())
            .unwrap()
            .get_validator_config_resource()
            .unwrap()
            .unwrap()
            .validator_config
            .unwrap()
            .consensus_public_key
    );

    // txn1 = give the validator some money so they can send a tx
    let txn1 = get_test_signed_transaction(
        treasury_compliance_account_address(),
        /* sequence_number = */ 0,
        genesis_key.clone(),
        genesis_key.public_key(),
        Some(TransactionPayload::Script(
            encode_peer_to_peer_with_metadata_script(
                xus_tag(),
                validator_account,
                1_000_000,
                vec![],
                vec![],
            ),
        )),
    );
    // txn2 = a dummy block prologue to bump the timer.
    let txn2 = Transaction::BlockMetadata(BlockMetadata::new(
        gen_block_id(1),
        1,
        300000001,
        vec![],
        validator_account,
    ));

    // txn3 = rotate the validator's consensus pubkey
    let operator_key = validators[0].key.clone();
    let operator_account = validators[0].data.operator_address;

    let new_pubkey = Ed25519PrivateKey::generate_for_testing().public_key();
    let txn3 = get_test_signed_transaction(
        operator_account,
        /* sequence_number = */ 0,
        operator_key.clone(),
        operator_key.public_key(),
        Some(TransactionPayload::Script(
            encode_set_validator_config_and_reconfigure_script(
                validator_account,
                new_pubkey.to_bytes().to_vec(),
                Vec::new(),
                Vec::new(),
            ),
        )),
    );

    let txn_block = vec![txn1, txn2, txn3];
    let block_id = gen_block_id(1);
    let vm_output = executor
        .execute_block((block_id, txn_block.clone()), parent_block_id)
        .unwrap();

    // Make sure the execution result sees the reconfiguration
    assert!(
        vm_output.has_reconfiguration(),
        "StateComputeResult does not see a reconfiguration"
    );
    let ledger_info_with_sigs = gen_ledger_info_with_sigs(1, &vm_output, block_id, vec![&signer]);
    executor
        .commit_blocks(vec![block_id], ledger_info_with_sigs)
        .unwrap();

    let state_proof = db.reader.get_state_proof(0).unwrap();
    let current_version = state_proof.latest_ledger_info().version();

    let t3 = db
        .reader
        .get_account_transaction(operator_account, 0, true, current_version)
        .unwrap();
    verify_committed_txn_status(t3.as_ref(), &txn_block[2]).unwrap();

    // test validator's key under validator_account is as expected
    let validator_account_state_with_proof = db
        .reader
        .get_account_state_with_proof(validator_account, current_version, current_version)
        .unwrap();
    assert_eq!(
        AccountState::try_from(&validator_account_state_with_proof.blob.unwrap())
            .unwrap()
            .get_validator_config_resource()
            .unwrap()
            .unwrap()
            .validator_config
            .unwrap()
            .consensus_public_key,
        new_pubkey
    );

    // test validator's key under validator's account is now equal to the key in the
    // validator set since the reconfiguration was invoked
    let validator_account_state_with_proof = db
        .reader
        .get_account_state_with_proof(validator_account, current_version, current_version)
        .unwrap();
    let diem_root_account_state_with_proof = db
        .reader
        .get_account_state_with_proof(diem_root_address(), current_version, current_version)
        .unwrap();
    assert_eq!(
        AccountState::try_from(&diem_root_account_state_with_proof.blob.unwrap())
            .unwrap()
            .get_validator_set()
            .unwrap()
            .unwrap()
            .payload()[0]
            .consensus_public_key(),
        &AccountState::try_from(&validator_account_state_with_proof.blob.unwrap())
            .unwrap()
            .get_validator_config_resource()
            .unwrap()
            .unwrap()
            .validator_config
            .unwrap()
            .consensus_public_key
    );

    // test validator's key in the validator set is as expected
    let diem_root_account_state_with_proof = db
        .reader
        .get_account_state_with_proof(diem_root_address(), current_version, current_version)
        .unwrap();
    let blob = &diem_root_account_state_with_proof.blob.unwrap();
    assert_eq!(
        AccountState::try_from(blob)
            .unwrap()
            .get_validator_set()
            .unwrap()
            .unwrap()
            .payload()[0]
            .consensus_public_key(),
        &new_pubkey
    );
}

#[test]
fn test_change_publishing_option_to_custom() {
    let path = diem_temppath::TempPath::new();
    path.create_as_dir().unwrap();
    let (genesis, validators) = vm_genesis::test_genesis_change_set_and_validators(Some(1));
    let genesis_key = &vm_genesis::GENESIS_KEYPAIR.0;
    let genesis_txn = Transaction::GenesisTransaction(WriteSetPayload::Direct(genesis));

    let (_, db, executor, waypoint) = create_db_and_executor(path.path(), &genesis_txn);
    let parent_block_id = executor.committed_block_id();

    let treasury_compliance_account = treasury_compliance_account_address();
    let genesis_account = diem_root_address();

    let signer = ValidatorSigner::new(validators[0].data.address, validators[0].key.clone());
    let validator_account = signer.author();
    let validator_privkey = signer.private_key();
    let validator_pubkey = validator_privkey.public_key();

    // give the validator some money so they can send a tx
    let txn1 = get_test_signed_transaction(
        treasury_compliance_account,
        /* sequence_number = */ 0,
        genesis_key.clone(),
        genesis_key.public_key(),
        Some(TransactionPayload::Script(
            encode_peer_to_peer_with_metadata_script(
                xus_tag(),
                validator_account,
                1_000_000,
                vec![],
                vec![],
            ),
        )),
    );

    let script1 = Script::new(vec![], vec![], vec![]);
    let script2 = Script::new(vec![1], vec![], vec![]);

    // Create a transaction that is not allowed with default publishing option and make sure it is
    // rejected.
    let txn2 = get_test_signed_transaction(
        validator_account,
        /* sequence_number = */ 0,
        validator_privkey.clone(),
        validator_pubkey.clone(),
        Some(TransactionPayload::Script(script1.clone())),
    );

    let txn3 = get_test_signed_transaction(
        validator_account,
        /* sequence_number = */ 0,
        validator_privkey.clone(),
        validator_pubkey.clone(),
        Some(TransactionPayload::Script(script2.clone())),
    );

    // Create a dummy block prologue transaction that will bump the timer.
    let txn4 = Transaction::BlockMetadata(gen_block_metadata(1, validator_account));

    let script_body = {
        let code = "
    import 0x1.DiemTransactionPublishingOption;

    main(account: signer) {
      DiemTransactionPublishingOption.set_open_script(&account);

      return;
    }
";

        let compiler = Compiler {
            deps: diem_framework_releases::current_modules().iter().collect(),
        };
        compiler
            .into_script_blob("file_name", code)
            .expect("Failed to compile")
    };
    let txn5 = get_test_signed_transaction(
        genesis_account,
        /* sequence_number = */ 1,
        genesis_key.clone(),
        genesis_key.public_key(),
        Some(TransactionPayload::Script(Script::new(
            script_body,
            vec![],
            vec![],
        ))),
    );

    let block1_id = gen_block_id(1);
    let block1 = vec![txn1, txn2, txn3, txn4, txn5];
    let output1 = executor
        .execute_block((block1_id, block1.clone()), parent_block_id)
        .unwrap();

    assert!(
        output1.has_reconfiguration(),
        "StateComputeResult has a new validator set"
    );

    let ledger_info_with_sigs = gen_ledger_info_with_sigs(1, &output1, block1_id, vec![&signer]);
    executor
        .commit_blocks(vec![block1_id], ledger_info_with_sigs)
        .unwrap();

    let trusted_state = TrustedState::from_epoch_waypoint(waypoint);
    let initial_accumulator = db
        .reader
        .get_accumulator_summary(trusted_state.version())
        .unwrap();
    let state_proof = db.reader.get_state_proof(trusted_state.version()).unwrap();
    let trusted_state_change = trusted_state
        .verify_and_ratchet(&state_proof, Some(&initial_accumulator))
        .unwrap();
    assert!(trusted_state_change.is_epoch_change());
    let trusted_state = trusted_state_change.new_state().unwrap();

    let current_version = trusted_state.version();
    assert_eq!(current_version, 3);
    // Transaction 1 is committed as it's in the allowlist
    let txn1 = db
        .reader
        .get_account_transaction(treasury_compliance_account, 0, false, current_version)
        .unwrap();
    verify_committed_txn_status(txn1.as_ref(), &block1[0]).unwrap();
    // Transaction 2, 3 are rejected
    assert!(db
        .reader
        .get_account_transaction(validator_account, 0, false, current_version)
        .unwrap()
        .is_none());

    // Now that the PublishingOption is modified to CustomScript, we can resubmit the script again.
    let txn2 = get_test_signed_transaction(
        validator_account,
        /* sequence_number = */ 0,
        validator_privkey.clone(),
        validator_pubkey.clone(),
        Some(TransactionPayload::Script(script1)),
    );

    let txn3 = get_test_signed_transaction(
        validator_account,
        /* sequence_number = */ 1,
        validator_privkey.clone(),
        validator_pubkey,
        Some(TransactionPayload::Script(script2)),
    );

    let block2_id = gen_block_id(2);
    let block2 = vec![txn2, txn3];
    let output2 = executor
        .execute_block((block2_id, block2.clone()), executor.committed_block_id())
        .unwrap();

    let ledger_info_with_sigs = gen_ledger_info_with_sigs(2, &output2, block2_id, vec![&signer]);
    executor
        .commit_blocks(vec![block2_id], ledger_info_with_sigs)
        .unwrap();

    let state_proof = db.reader.get_state_proof(current_version).unwrap();
    let trusted_state_change = trusted_state
        .verify_and_ratchet(&state_proof, None)
        .unwrap();
    assert!(!trusted_state_change.is_epoch_change());

    let current_version = state_proof.latest_ledger_info().version();
    assert_eq!(current_version, 5);
    // Transaction 2 is committed.
    let txn2 = db
        .reader
        .get_account_transaction(validator_account, 0, false, current_version)
        .unwrap();
    verify_committed_txn_status(txn2.as_ref(), &block2[0]).unwrap();
    // Transaction 3 is committed.
    let txn3 = db
        .reader
        .get_account_transaction(validator_account, 1, false, current_version)
        .unwrap();
    verify_committed_txn_status(txn3.as_ref(), &block2[1]).unwrap();
}

#[test]
fn test_execution_with_storage() {
    test_execution_with_storage_impl();
}
