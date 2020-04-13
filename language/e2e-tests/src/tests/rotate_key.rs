// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::{Account, AccountData},
    common_transactions::{create_account_txn, raw_rotate_key_txn, rotate_key_txn},
    executor::FakeExecutor,
    keygen::KeyGen,
};
use libra_crypto::{
    ed25519::Ed25519PrivateKey, hash::CryptoHash, multi_ed25519, PrivateKeyExt, SigningKey, Uniform,
};
use libra_types::{
    account_address::AccountAddress,
    transaction::{authenticator::AuthenticationKey, SignedTransaction, TransactionStatus},
    vm_error::{StatusCode, VMStatus},
};

#[test]
fn rotate_ed25519_key() {
    let mut executor = FakeExecutor::from_genesis_file();
    // create and publish sender
    let mut sender = AccountData::new(1_000_000, 10);
    executor.add_account_data(&sender);

    let privkey = Ed25519PrivateKey::generate_for_testing();
    let pubkey = privkey.public_key();
    let new_key_hash = AccountAddress::authentication_key(&pubkey).to_vec();
    let txn = rotate_key_txn(sender.account(), new_key_hash.clone(), 10);

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
    let (updated_sender, updated_sender_balance) = executor
        .read_account_info(sender.account())
        .expect("sender must exist");
    assert_eq!(new_key_hash, updated_sender.authentication_key().to_vec());
    assert_eq!(balance, updated_sender_balance.coin());
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
}

#[test]
fn rotate_ed25519_multisig_key() {
    let mut executor = FakeExecutor::from_genesis_file();
    let mut seq_number = 10;
    // create and publish sender
    let sender = AccountData::new(1_000_000, seq_number);
    executor.add_account_data(&sender);
    let sender_address = sender.address();

    // create a 1-of-2 multisig policy
    let mut keygen = KeyGen::from_seed([9u8; 32]);

    let (privkey1, pubkey1) = keygen.generate_keypair();
    let (privkey2, pubkey2) = keygen.generate_keypair();
    let threshold = 1;
    let multi_ed_public_key =
        multi_ed25519::PublicKey::new(vec![pubkey1, pubkey2], threshold).unwrap();
    let new_auth_key = AuthenticationKey::multi_ed25519(&multi_ed_public_key);

    // (1) rotate key to multisig
    let output = &executor.execute_transaction(rotate_key_txn(
        sender.account(),
        new_auth_key.to_vec(),
        seq_number,
    ));
    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED)),
    );
    executor.apply_write_set(output.write_set());
    seq_number += 1;

    // (2) send a tx signed by privkey 1
    let txn1 = raw_rotate_key_txn(*sender_address, new_auth_key.to_vec(), seq_number);
    let signature1 = multi_ed25519::Signature::from(privkey1.sign_message(&txn1.hash()));
    let signed_txn1 =
        SignedTransaction::new_multisig(txn1, multi_ed_public_key.clone(), signature1);
    let output = &executor.execute_transaction(signed_txn1);
    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED)),
    );
    executor.apply_write_set(output.write_set());
    seq_number += 1;

    // (3) send a tx signed by privkey 2
    let txn2 = raw_rotate_key_txn(*sender_address, new_auth_key.to_vec(), seq_number);
    let pubkey_index = 1;
    let signature2 =
        multi_ed25519::Signature::new(vec![(privkey2.sign_message(&txn2.hash()), pubkey_index)])
            .unwrap();
    let signed_txn2 = SignedTransaction::new_multisig(txn2, multi_ed_public_key, signature2);
    signed_txn2.clone().check_signature().unwrap();
    let output = &executor.execute_transaction(signed_txn2);
    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED)),
    );
}
