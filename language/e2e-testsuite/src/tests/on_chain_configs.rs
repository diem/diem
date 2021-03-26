// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_framework_releases::legacy::transaction_scripts::LegacyStdlibScript;
use diem_transaction_builder::stdlib::encode_update_dual_attestation_limit_script;
use diem_types::{
    account_config::CORE_CODE_ADDRESS,
    on_chain_config::DiemVersion,
    transaction::{Script, ScriptFunction, TransactionArgument, TransactionStatus},
    vm_status::{KeptVMStatus, StatusCode},
};
use diem_vm::DiemVM;
use language_e2e_tests::{
    account::{self, Account},
    assert_prologue_parity,
    common_transactions::peer_to_peer_txn,
    current_function_name,
    executor::FakeExecutor,
    test_with_different_versions, transaction_status_eq,
    versioning::CURRENT_RELEASE_VERSIONS,
};
use move_core_types::{
    identifier::Identifier, language_storage::ModuleId, transaction_argument::convert_txn_args,
};

#[test]
fn initial_diem_version() {
    test_with_different_versions! {CURRENT_RELEASE_VERSIONS, |test_env| {
        let mut executor = test_env.executor;

        let vm = DiemVM::new(executor.get_state_view());

        assert_eq!(
            vm.internals().diem_version().unwrap(),
            DiemVersion { major: test_env.version_number }
        );

        let account = test_env.dr_account;
        let txn = account
            .transaction()
            .script(Script::new(
                LegacyStdlibScript::UpdateDiemVersion
                    .compiled_bytes()
                    .into_vec(),
                vec![],
                vec![TransactionArgument::U64(0), TransactionArgument::U64(test_env.version_number + 1)],
            ))
            .sequence_number(test_env.dr_sequence_number)
            .sign();
        executor.new_block();
        executor.execute_and_apply(txn);

        let new_vm = DiemVM::new(executor.get_state_view());
        assert_eq!(
            new_vm.internals().diem_version().unwrap(),
            DiemVersion { major: test_env.version_number + 1 }
        );
    }
    }
}

#[test]
fn drop_txn_after_reconfiguration() {
    test_with_different_versions! {CURRENT_RELEASE_VERSIONS, |test_env| {
        let mut executor = test_env.executor;
        let vm = DiemVM::new(executor.get_state_view());

        assert_eq!(
            vm.internals().diem_version().unwrap(),
            DiemVersion { major: test_env.version_number }
        );

        let account = test_env.dr_account;
        let txn = account
            .transaction()
            .script(Script::new(
                LegacyStdlibScript::UpdateDiemVersion
                    .compiled_bytes()
                    .into_vec(),
                vec![],
                vec![TransactionArgument::U64(0), TransactionArgument::U64(test_env.version_number + 1)],
            ))
            .sequence_number(test_env.dr_sequence_number)
            .sign();
        executor.new_block();

        let sender = executor.create_raw_account_data(1_000_000, 10);
        let receiver = executor.create_raw_account_data(100_000, 10);
        let txn2 = peer_to_peer_txn(&sender.account(), &receiver.account(), 11, 1000);

        let mut output = executor.execute_block(vec![txn, txn2]).unwrap();
        assert_eq!(output.pop().unwrap().status(), &TransactionStatus::Retry)
    }
    }
}

#[test]
fn updated_limit_allows_txn() {
    test_with_different_versions! {CURRENT_RELEASE_VERSIONS, |test_env| {
        let mut executor = test_env.executor;
        let blessed = test_env.tc_account;
        // create and publish a sender with 5_000_000 coins and a receiver with 0 coins
        let sender = executor.create_raw_account_data(5_000_000, 10);
        let receiver = executor.create_raw_account_data(0, 10);
        executor.add_account_data(&sender);
        executor.add_account_data(&receiver);

        // Execute updated dual attestation limit
        let new_micro_xdx_limit = 1_000_011;
        let output = executor.execute_and_apply(
            blessed
                .transaction()
                .script(encode_update_dual_attestation_limit_script(
                    3,
                    new_micro_xdx_limit,
                ))
                .sequence_number(test_env.tc_sequence_number)
                .sign(),
        );
        assert_eq!(
            output.status(),
            &TransactionStatus::Keep(KeptVMStatus::Executed)
        );

        // higher transaction works with higher limit
        let transfer_amount = 1_000_010;
        let txn = peer_to_peer_txn(sender.account(), receiver.account(), 10, transfer_amount);
        let output = executor.execute_and_apply(txn);
        assert!(transaction_status_eq(
            &output.status(),
            &TransactionStatus::Keep(KeptVMStatus::Executed)
        ));
        let sender_balance = executor
            .read_balance_resource(sender.account(), account::xus_currency_code())
            .expect("sender balance must exist");
        let receiver_balance = executor
            .read_balance_resource(receiver.account(), account::xus_currency_code())
            .expect("receiver balance must exist");

        assert_eq!(3_999_990, sender_balance.coin());
        assert_eq!(1_000_010, receiver_balance.coin());
    }
    }
}

#[test]
fn update_script_allow_list() {
    // create a FakeExecutor with a genesis from file
    let mut executor = FakeExecutor::allowlist_genesis();
    executor.set_golden_file(current_function_name!());
    let dr = Account::new_diem_root();
    // create and publish a sender with 5_000_000 coins and a receiver with 0 coins
    let sender = executor.create_raw_account_data(5_000_000, 10);
    executor.add_account_data(&sender);

    // Regular accounts cannot send arbitrary txn to the network.
    let random_script = vec![];
    let txn = sender
        .account()
        .transaction()
        .script(Script::new(random_script, vec![], vec![]))
        .sequence_number(10)
        .max_gas_amount(100_000)
        .gas_unit_price(1)
        .sign();

    assert_prologue_parity!(
        executor.verify_transaction(txn.clone()).status(),
        executor.execute_transaction(txn).status(),
        StatusCode::UNKNOWN_SCRIPT
    );

    // DIEM_ROOT can send arbitrary txn to the network.
    let random_script = vec![];
    let txn = dr
        .transaction()
        .script(Script::new(random_script, vec![], vec![]))
        .sequence_number(1)
        .sign();

    assert_eq!(
        executor.execute_transaction(txn).status(),
        &TransactionStatus::Keep(KeptVMStatus::MiscellaneousError)
    );
}

#[test]
fn update_consensus_config() {
    test_with_different_versions! {CURRENT_RELEASE_VERSIONS, |test_env| {
        let mut executor = test_env.executor;

        let account = test_env.dr_account;
        let generate_txn = |seq_num, fun_name, config| {
            // sliding nonce
            let mut args = vec![TransactionArgument::U64(seq_num)];
            if let Some(config) = config {
                args.push(TransactionArgument::U8Vector(config));
            }
            account
                .transaction()
                .script_function(ScriptFunction::new(
                    ModuleId::new(
                        CORE_CODE_ADDRESS,
                        Identifier::new("SystemAdministrationScripts").unwrap(),
                    ),
                    Identifier::new(fun_name).unwrap(),
                    vec![],
                    convert_txn_args(&args),
                ))
                .sequence_number(seq_num)
                .sign()
        };
        let seq_num = test_env.dr_sequence_number;

        if test_env.version_number == 1 {
            assert_eq!(executor.execute_transaction(generate_txn(seq_num, "update_dime_consensus_config", Some(vec![1,2,3]))).status(), &TransactionStatus::Discard(StatusCode::FEATURE_UNDER_GATING));
        }

        if test_env.version_number == 2 {
            // update abort when uninitialized
            assert!(matches!(executor.execute_transaction(generate_txn(seq_num, "update_diem_consensus_config", Some(vec![1,2,3]))).status(), &TransactionStatus::Keep(KeptVMStatus::MoveAbort(_, _))));
            assert_eq!(executor.execute_and_apply(generate_txn(seq_num, "initialize_diem_consensus_config", None)).status(), &TransactionStatus::Keep(KeptVMStatus::Executed));
            assert_eq!(executor.execute_and_apply(generate_txn(seq_num + 1, "update_diem_consensus_config", Some(vec![1,2,3]))).status(), &TransactionStatus::Keep(KeptVMStatus::Executed));
        }

    }
    }
}
