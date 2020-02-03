// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::{Account, AccountData},
    common_transactions::{create_account_txn, rotate_key_txn},
    executor::test_all_genesis_default,
};
use libra_crypto::ed25519::compat;
use libra_types::{
    account_address::AccountAddress,
    transaction::TransactionStatus,
    vm_error::{StatusCode, VMStatus},
};

#[test]
fn rotate_key() {
    test_all_genesis_default(|mut executor| {
        // create and publish sender
        let mut sender = AccountData::new(1_000_000, 10);
        executor.add_account_data(&sender);

        let (privkey, pubkey) = compat::generate_keypair(None);
        let new_key_hash = AccountAddress::from_public_key(&pubkey);
        let txn = rotate_key_txn(sender.account(), new_key_hash, 10);

        // execute transaction
        let output = &executor.execute_transaction(txn);
        assert_eq!(
            output.status(),
            &TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED)),
        );
        executor.apply_write_set(output.write_set());

        // Check that numbers in store are correct.
        let gas = output.gas_used();
        let balance = 1_000_000 - gas;
        let updated_sender = executor
            .read_account_resource(sender.account())
            .expect("sender must exist");
        assert_eq!(
            new_key_hash.as_ref(),
            updated_sender.authentication_key().as_bytes(),
        );
        assert_eq!(balance, updated_sender.balance());
        assert_eq!(11, updated_sender.sequence_number());

        // Check that transactions cannot be sent with the old key any more.
        let new_account = Account::new();
        let old_key_txn = create_account_txn(sender.account(), &new_account, 11, 100_000);
        let old_key_output = &executor.execute_transaction(old_key_txn);
        assert_eq!(
            old_key_output.status(),
            &TransactionStatus::Discard(VMStatus::new(StatusCode::INVALID_AUTH_KEY)),
        );

        // Check that transactions can be sent with the new key.
        sender.rotate_key(privkey, pubkey);
        let new_key_txn = create_account_txn(sender.account(), &new_account, 11, 100_000);
        let new_key_output = &executor.execute_transaction(new_key_txn);
        assert_eq!(
            new_key_output.status(),
            &TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED)),
        );
    });
}
