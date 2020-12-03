// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_crypto::{
    ed25519::Ed25519PrivateKey,
    multi_ed25519::{MultiEd25519PublicKey, MultiEd25519Signature},
    PrivateKey, SigningKey, Uniform,
};
use diem_types::{
    transaction::{authenticator::AuthenticationKey, SignedTransaction, TransactionStatus},
    vm_status::{KeptVMStatus, StatusCode},
};
use language_e2e_tests::{
    account,
    common_transactions::{raw_rotate_key_txn, rotate_key_txn},
    current_function_name,
    executor::FakeExecutor,
    keygen::KeyGen,
};

#[test]
fn rotate_ed25519_key() {
    let balance = 1_000_000;
    let mut executor = FakeExecutor::from_genesis_file();
    executor.set_golden_file(current_function_name!());

    // create and publish sender
    let mut sender = executor.create_raw_account_data(balance, 10);
    executor.add_account_data(&sender);

    let privkey = Ed25519PrivateKey::generate_for_testing();
    let pubkey = privkey.public_key();
    let new_key_hash = AuthenticationKey::ed25519(&pubkey).to_vec();
    let txn = rotate_key_txn(sender.account(), new_key_hash.clone(), 10);

    // execute transaction
    let output = &executor.execute_transaction(txn);
    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(KeptVMStatus::Executed),
    );
    executor.apply_write_set(output.write_set());

    // Check that numbers in store are correct.
    let updated_sender = executor
        .read_account_resource(sender.account())
        .expect("sender must exist");
    let updated_sender_balance = executor
        .read_balance_resource(sender.account(), account::xus_currency_code())
        .expect("sender balance must exist");
    assert_eq!(new_key_hash, updated_sender.authentication_key().to_vec());
    assert_eq!(balance, updated_sender_balance.coin());
    assert_eq!(11, updated_sender.sequence_number());

    // Check that transactions cannot be sent with the old key any more.
    let old_key_txn = rotate_key_txn(sender.account(), vec![], 11);
    let old_key_output = &executor.execute_transaction(old_key_txn);
    assert_eq!(
        old_key_output.status(),
        &TransactionStatus::Discard(StatusCode::INVALID_AUTH_KEY),
    );

    // Check that transactions can be sent with the new key.
    sender.rotate_key(privkey, pubkey);
    let new_key_txn = rotate_key_txn(sender.account(), new_key_hash, 11);
    let new_key_output = &executor.execute_transaction(new_key_txn);
    assert_eq!(
        new_key_output.status(),
        &TransactionStatus::Keep(KeptVMStatus::Executed),
    );
}

#[test]
fn rotate_ed25519_multisig_key() {
    let mut executor = FakeExecutor::from_genesis_file();
    executor.set_golden_file(current_function_name!());

    let mut seq_number = 10;
    // create and publish sender
    let sender = executor.create_raw_account_data(1_000_000, seq_number);
    executor.add_account_data(&sender);
    let _sender_address = sender.address();

    // create a 1-of-2 multisig policy
    let mut keygen = KeyGen::from_seed([9u8; 32]);

    let (privkey1, pubkey1) = keygen.generate_keypair();
    let (privkey2, pubkey2) = keygen.generate_keypair();
    let threshold = 1;
    let multi_ed_public_key =
        MultiEd25519PublicKey::new(vec![pubkey1, pubkey2], threshold).unwrap();
    let new_auth_key = AuthenticationKey::multi_ed25519(&multi_ed_public_key);

    // (1) rotate key to multisig
    let output = &executor.execute_transaction(rotate_key_txn(
        sender.account(),
        new_auth_key.to_vec(),
        seq_number,
    ));
    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(KeptVMStatus::Executed),
    );
    executor.apply_write_set(output.write_set());
    seq_number += 1;

    // (2) send a tx signed by privkey 1
    let txn1 = raw_rotate_key_txn(sender.account(), new_auth_key.to_vec(), seq_number);
    let signature1 = MultiEd25519Signature::from(privkey1.sign(&txn1));
    let signed_txn1 =
        SignedTransaction::new_multisig(txn1, multi_ed_public_key.clone(), signature1);
    let output = &executor.execute_transaction(signed_txn1);
    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(KeptVMStatus::Executed),
    );
    executor.apply_write_set(output.write_set());
    seq_number += 1;

    // (3) send a tx signed by privkey 2
    let txn2 = raw_rotate_key_txn(sender.account(), new_auth_key.to_vec(), seq_number);
    let pubkey_index = 1;
    let signature2 =
        MultiEd25519Signature::new(vec![(privkey2.sign(&txn2), pubkey_index)]).unwrap();
    let signed_txn2 = SignedTransaction::new_multisig(txn2, multi_ed_public_key, signature2);
    signed_txn2.clone().check_signature().unwrap();
    let output = &executor.execute_transaction(signed_txn2);
    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(KeptVMStatus::Executed),
    );
}

#[test]

fn rotate_shared_ed25519_public_key() {}
