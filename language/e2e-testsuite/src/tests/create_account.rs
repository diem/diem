// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_types::{account_config, transaction::TransactionStatus, vm_status::KeptVMStatus};
use language_e2e_tests::{
    account, common_transactions::create_account_txn, test_with_different_versions,
    versioning::CURRENT_RELEASE_VERSIONS,
};

#[test]
fn create_account() {
    test_with_different_versions! {CURRENT_RELEASE_VERSIONS, |test_env| {
        let mut executor = test_env.executor;

        // create and publish a sender with 1_000_000 coins
        let sender = test_env.tc_account;
        let new_account = executor.create_raw_account();

        // define the arguments to the create account transaction
        let initial_amount = 0;
        let txn = create_account_txn(
            &sender,
            &new_account,
            test_env.tc_sequence_number,
            initial_amount,
            account_config::xus_tag(),
        );

        // execute transaction
        let output = executor.execute_transaction(txn);
        assert_eq!(
            output.status(),
            &TransactionStatus::Keep(KeptVMStatus::Executed)
        );
        executor.apply_write_set(output.write_set());

        // check that numbers in stored DB are correct
        let updated_sender = executor
            .read_account_resource(&sender)
            .expect("sender must exist");

        let updated_receiver_balance = executor
            .read_balance_resource(&new_account, account::xus_currency_code())
            .expect("receiver balance must exist");
        assert_eq!(initial_amount, updated_receiver_balance.coin());
        assert_eq!(1, updated_sender.sequence_number());
    }
    }
}
