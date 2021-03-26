// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_transaction_builder::stdlib::*;
use diem_types::{account_config, transaction::TransactionOutput};
use language_e2e_tests::{account::Account, current_function_name, executor::FakeExecutor, utils};
use move_core_types::vm_status::{DiscardedVMStatus, KeptVMStatus};

fn create_preburn_balance(
    executor: &mut FakeExecutor,
    dd_account: &Account,
    preburn_amount: u64,
    dd_seqno: &mut u64,
    should_fail: bool,
) -> Option<TransactionOutput> {
    let script = encode_preburn_script(account_config::xus_tag(), preburn_amount);
    let txn = dd_account
        .transaction()
        .script(script)
        .sequence_number(*dd_seqno)
        .sign();
    if should_fail {
        Some(executor.execute_transaction(txn))
    } else {
        executor.execute_and_apply(txn);
        *dd_seqno = dd_seqno.checked_add(1).unwrap();
        None
    }
}

fn burn_old(
    executor: &mut FakeExecutor,
    tc_account: &Account,
    tc_seqno: &mut u64,
    should_fail: bool,
) -> Option<TransactionOutput> {
    let txn = tc_account
        .transaction()
        .script(encode_burn_script(
            account_config::xus_tag(),
            0,
            account_config::testnet_dd_account_address(),
        ))
        .sequence_number(*tc_seqno)
        .sign();
    if should_fail {
        Some(executor.execute_transaction(txn))
    } else {
        executor.execute_and_apply(txn);
        *tc_seqno = tc_seqno.checked_add(1).unwrap();
        None
    }
}

fn cancel_burn_old(
    executor: &mut FakeExecutor,
    tc_account: &Account,
    tc_seqno: &mut u64,
    should_fail: bool,
) -> Option<TransactionOutput> {
    let txn = tc_account
        .transaction()
        .script(encode_cancel_burn_script(
            account_config::xus_tag(),
            account_config::testnet_dd_account_address(),
        ))
        .sequence_number(*tc_seqno)
        .sign();
    if should_fail {
        Some(executor.execute_transaction(txn))
    } else {
        executor.execute_and_apply(txn);
        *tc_seqno = tc_seqno.checked_add(1).unwrap();
        None
    }
}

fn burn_with_amount_new(
    executor: &mut FakeExecutor,
    amount: u64,
    tc_account: &Account,
    tc_seqno: &mut u64,
    should_fail: bool,
) -> Option<TransactionOutput> {
    let txn = tc_account
        .transaction()
        .payload(encode_burn_with_amount_script_function(
            account_config::xus_tag(),
            0,
            account_config::testnet_dd_account_address(),
            amount,
        ))
        .sequence_number(*tc_seqno)
        .sign();
    if should_fail {
        Some(executor.execute_transaction(txn))
    } else {
        executor.execute_and_apply(txn);
        *tc_seqno = tc_seqno.checked_add(1).unwrap();
        None
    }
}

fn cancel_burn_with_amount_new(
    executor: &mut FakeExecutor,
    amount: u64,
    tc_account: &Account,
    tc_seqno: &mut u64,
    should_fail: bool,
) -> Option<TransactionOutput> {
    let txn = tc_account
        .transaction()
        .payload(encode_cancel_burn_with_amount_script_function(
            account_config::xus_tag(),
            account_config::testnet_dd_account_address(),
            amount,
        ))
        .sequence_number(*tc_seqno)
        .sign();
    if should_fail {
        Some(executor.execute_transaction(txn))
    } else {
        executor.execute_and_apply(txn);
        *tc_seqno = tc_seqno.checked_add(1).unwrap();
        None
    }
}

/// Make sure we can preburn and before the upgrade with the old scripts, and that we can
/// preburn and burn after the upgrade with the new scripts (and the old preburn script). Note
/// that the preburn balance at the time of upgrade is zero.
#[test]
fn concurrent_preburns_test_upgrade_empty_preburn() {
    let (mut executor, dr_account, tc_account, dd_account) = utils::start_with_released_df();
    let mut tc_seqno = 0;
    let mut dd_seqno = 0;
    let mut dr_seqno = 1;
    executor.set_golden_file(current_function_name!());

    create_preburn_balance(&mut executor, &dd_account, 10, &mut dd_seqno, false);
    burn_old(&mut executor, &tc_account, &mut tc_seqno, false);

    create_preburn_balance(&mut executor, &dd_account, 10, &mut dd_seqno, false);
    cancel_burn_old(&mut executor, &tc_account, &mut tc_seqno, false);

    // Upgrade the DF
    utils::upgrade_df(&mut executor, &dr_account, &mut dr_seqno, Some(2));

    // Now make sure that we can upgrade the DD
    create_preburn_balance(&mut executor, &dd_account, 100, &mut dd_seqno, false);
    burn_with_amount_new(&mut executor, 100, &tc_account, &mut tc_seqno, false);

    // Make sure now that we're upgraded that we can keep doing this
    create_preburn_balance(&mut executor, &dd_account, 10, &mut dd_seqno, false);
    cancel_burn_with_amount_new(&mut executor, 10, &tc_account, &mut tc_seqno, false);
}

/// Make sure we can preburn and before the upgrade with the old scripts, and that we can
/// preburn and burn after the upgrade with the new scripts (and the old preburn script) and that
/// concurrent preburns are supported immediately without any additional input from TC. Note that
/// the preburn balance at the time of upgrade is zero.
#[test]
fn concurrent_preburns_test_upgrade_empty_preburn_multiple_initial_preburns() {
    let (mut executor, dr_account, tc_account, dd_account) = utils::start_with_released_df();
    let mut tc_seqno = 0;
    let mut dd_seqno = 0;
    let mut dr_seqno = 1;
    executor.set_golden_file(current_function_name!());

    create_preburn_balance(&mut executor, &dd_account, 10, &mut dd_seqno, false);
    burn_old(&mut executor, &tc_account, &mut tc_seqno, false);

    create_preburn_balance(&mut executor, &dd_account, 10, &mut dd_seqno, false);
    cancel_burn_old(&mut executor, &tc_account, &mut tc_seqno, false);

    // Upgrade the DF
    utils::upgrade_df(&mut executor, &dr_account, &mut dr_seqno, Some(2));

    // Now make sure that we can upgrade the DD
    create_preburn_balance(&mut executor, &dd_account, 100, &mut dd_seqno, false);
    // Make sure we can now have multiple preburn requests
    create_preburn_balance(&mut executor, &dd_account, 20, &mut dd_seqno, false);

    // burn and cancel in a different order
    burn_with_amount_new(&mut executor, 20, &tc_account, &mut tc_seqno, false);
    cancel_burn_with_amount_new(&mut executor, 100, &tc_account, &mut tc_seqno, false);
}

/// If the DF is upgraded and a DD has a non-zero preburn balance at the time of the
/// upgrade, make sure that the system doesn't reach a stuck state. But, the DD will need to
/// upgrade with a nonzero preburn first.
#[test]
fn concurrent_preburns_test_upgrade_existing_balance_at_upgrade() {
    let (mut executor, dr_account, tc_account, dd_account) = utils::start_with_released_df();
    let mut tc_seqno = 0;
    let mut dd_seqno = 0;
    let mut dr_seqno = 1;
    executor.set_golden_file(current_function_name!());

    create_preburn_balance(&mut executor, &dd_account, 10, &mut dd_seqno, false);

    // Upgrade the DF
    utils::upgrade_df(&mut executor, &dr_account, &mut dr_seqno, Some(2));

    // The new burn/cancel burn scripts will fail since the preburn hasn't been upgraded yet
    assert!(matches!(
        cancel_burn_with_amount_new(&mut executor, 100, &tc_account, &mut tc_seqno, true)
            .unwrap()
            .status()
            .status()
            .unwrap(),
        KeptVMStatus::MoveAbort(_, 2821)
    ));
    assert!(matches!(
        burn_with_amount_new(&mut executor, 100, &tc_account, &mut tc_seqno, true)
            .unwrap()
            .status()
            .status()
            .unwrap(),
        KeptVMStatus::MoveAbort(_, 2821)
    ));

    // Now upgrade the preburn (recall it already has a nonzero balance).
    create_preburn_balance(&mut executor, &dd_account, 100, &mut dd_seqno, false);
    // Now the new burn/cancel burn scripts will succeed.
    cancel_burn_with_amount_new(&mut executor, 100, &tc_account, &mut tc_seqno, false);
    burn_with_amount_new(&mut executor, 10, &tc_account, &mut tc_seqno, false);
}

/// Make sure we can preburn before the upgrade with the old scripts.
/// Test that the old `burn` and `cancel_burn` scripts will fail after the upgrade though, even if
/// the old data hasn't been upgraded yet.
#[test]
fn concurrent_preburns_test_upgrade_old_data_new_df_old_scripts_fail() {
    let (mut executor, dr_account, tc_account, dd_account) = utils::start_with_released_df();
    let mut tc_seqno = 0;
    let mut dd_seqno = 0;
    let mut dr_seqno = 1;
    executor.set_golden_file(current_function_name!());

    create_preburn_balance(&mut executor, &dd_account, 10, &mut dd_seqno, false);
    burn_old(&mut executor, &tc_account, &mut tc_seqno, false);

    create_preburn_balance(&mut executor, &dd_account, 10, &mut dd_seqno, false);
    cancel_burn_old(&mut executor, &tc_account, &mut tc_seqno, false);

    // Upgrade the DF
    utils::upgrade_df(&mut executor, &dr_account, &mut dr_seqno, Some(2));

    // the old scripts will now fail with a `MISCELLANEOUS_ERROR`
    assert_eq!(
        burn_old(&mut executor, &tc_account, &mut tc_seqno, true)
            .unwrap()
            .status()
            .status()
            .unwrap(),
        KeptVMStatus::MiscellaneousError
    );

    assert_eq!(
        cancel_burn_old(&mut executor, &tc_account, &mut tc_seqno, true)
            .unwrap()
            .status()
            .status()
            .unwrap(),
        KeptVMStatus::MiscellaneousError
    );
}

/// Make sure we can preburn before the upgrade with the old scripts, and that we can
/// preburn and burn after the upgrade with the new scripts (and the old preburn script) and that
/// concurrent preburns are supported immediately without any additional input from TC. We test
/// that the old `burn` and `cancel_burn` scripts will fail after the upgrade though.
#[test]
fn concurrent_preburns_test_upgrade_new_data_new_df_old_scripts_fail() {
    let (mut executor, dr_account, tc_account, dd_account) = utils::start_with_released_df();
    let mut tc_seqno = 0;
    let mut dd_seqno = 0;
    let mut dr_seqno = 1;
    executor.set_golden_file(current_function_name!());

    create_preburn_balance(&mut executor, &dd_account, 10, &mut dd_seqno, false);
    burn_old(&mut executor, &tc_account, &mut tc_seqno, false);

    create_preburn_balance(&mut executor, &dd_account, 10, &mut dd_seqno, false);
    cancel_burn_old(&mut executor, &tc_account, &mut tc_seqno, false);

    // Upgrade the DF
    utils::upgrade_df(&mut executor, &dr_account, &mut dr_seqno, Some(2));

    // Now make sure that we can upgrade the DD
    create_preburn_balance(&mut executor, &dd_account, 100, &mut dd_seqno, false);
    // the old scripts will now fail with a `MISCELLANEOUS_ERROR`
    assert_eq!(
        burn_old(&mut executor, &tc_account, &mut tc_seqno, true)
            .unwrap()
            .status()
            .status()
            .unwrap(),
        KeptVMStatus::MiscellaneousError
    );

    assert_eq!(
        cancel_burn_old(&mut executor, &tc_account, &mut tc_seqno, true)
            .unwrap()
            .status()
            .status()
            .unwrap(),
        KeptVMStatus::MiscellaneousError
    );
}

/// Test that the new scripts with the old DF framework will fail. These will not work for multiple
/// reasons, but the first to catch is the gating on script functions.
#[test]
fn concurrent_preburns_old_data_old_df_new_scripts() {
    let (mut executor, _, tc_account, _) = utils::start_with_released_df();
    let mut tc_seqno = 0;
    executor.set_golden_file(current_function_name!());

    // The new burn/cancel burn scripts will fail since the preburn hasn't been upgraded yet
    assert!(matches!(
        cancel_burn_with_amount_new(&mut executor, 100, &tc_account, &mut tc_seqno, true)
            .unwrap()
            .status()
            .status()
            .unwrap_err(),
        DiscardedVMStatus::FEATURE_UNDER_GATING
    ));
    assert!(matches!(
        burn_with_amount_new(&mut executor, 100, &tc_account, &mut tc_seqno, true)
            .unwrap()
            .status()
            .status()
            .unwrap_err(),
        DiscardedVMStatus::FEATURE_UNDER_GATING
    ));
}
