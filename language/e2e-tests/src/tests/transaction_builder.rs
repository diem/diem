// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Tests for all of the script encoding functions in language/transaction_builder/lib.rs.
//! Thorough tests that exercise all of the behaviors of the script should live in the language
//! functional tests; these tests are only to ensure that the script encoding functions take the
//! correct types + produce a runnable script.

#![forbid(unsafe_code)]

use crate::{
    account::{Account, AccountData},
    common_transactions::{mint_txn, rotate_key_txn},
    executor::FakeExecutor,
    keygen::KeyGen,
};
use libra_crypto::{ed25519::Ed25519PrivateKey, traits::SigningKey, PrivateKey, Uniform};
use libra_types::{
    account_config,
    transaction::{authenticator::AuthenticationKey, TransactionStatus},
    vm_error::{StatusCode, VMStatus},
};
use move_core_types::{
    identifier::Identifier,
    language_storage::{StructTag, TypeTag},
};
use transaction_builder::*;

#[test]
fn register_preburn_burn() {
    // create a FakeExecutor with a genesis from file
    let mut executor = FakeExecutor::from_genesis_file();
    // association account to do the actual burning
    let association = Account::new_association();

    // account to initiate preburning
    let preburner = {
        let data = AccountData::new(0, 0);
        executor.add_account_data(&data);
        data.into_account()
    };

    // We need to mint in order to bump the market cap
    let txn = mint_txn(&association, &preburner, 1, 1_000_000);
    let output = executor.execute_transaction(txn);
    executor.apply_write_set(output.write_set());

    // Register preburner
    executor.execute_and_apply(preburner.signed_script_txn(
        encode_register_preburner_script(account_config::lbr_type_tag()),
        0,
    ));
    // Send a preburn request
    executor.execute_and_apply(preburner.signed_script_txn(
        encode_preburn_script(account_config::lbr_type_tag(), 100),
        1,
    ));
    // Send a second preburn request
    executor.execute_and_apply(preburner.signed_script_txn(
        encode_preburn_script(account_config::lbr_type_tag(), 200),
        2,
    ));

    // Complete the first request by burning
    executor.execute_and_apply(association.signed_script_txn(
        encode_burn_script(account_config::lbr_type_tag(), *preburner.address()),
        2,
    ));
    // Complete the second request by cancelling
    executor.execute_and_apply(association.signed_script_txn(
        encode_cancel_burn_script(account_config::lbr_type_tag(), *preburner.address()),
        3,
    ));
}

#[test]
fn freeze_unfreeze_account() {
    // create a FakeExecutor with a genesis from file
    let mut executor = FakeExecutor::from_genesis_file();
    // association account to do the actual burning
    let association = Account::new_association();

    let account = {
        let data = AccountData::new(1_000_000, 0);
        executor.add_account_data(&data);
        data.into_account()
    };

    let lbr_ty = TypeTag::Struct(StructTag {
        address: account_config::CORE_CODE_ADDRESS,
        module: Identifier::new("LibraAccount").unwrap(),
        name: Identifier::new("FreezingPrivilege").unwrap(),
        type_params: vec![],
    });

    // Request freezing privilege
    executor.execute_and_apply(
        association.signed_script_txn(encode_apply_for_association_privilege(lbr_ty.clone()), 1),
    );

    // Grant freezing privilege to assocation account
    executor.execute_and_apply(association.signed_script_txn(
        encode_grant_association_privilege(lbr_ty, *association.address()),
        2,
    ));
    // Execute freeze on account
    executor.execute_and_apply(
        association.signed_script_txn(encode_freeze_account(*account.address()), 3),
    );

    // Attempt rotate key txn from frozen account
    let privkey = Ed25519PrivateKey::generate_for_testing();
    let pubkey = privkey.public_key();
    let new_key_hash = AuthenticationKey::ed25519(&pubkey).to_vec();
    let txn = rotate_key_txn(&account, new_key_hash, 0);

    let output = &executor.execute_transaction(txn.clone());
    assert_eq!(
        output.status(),
        &TransactionStatus::Discard(VMStatus::new(StatusCode::SENDING_ACCOUNT_FROZEN)),
    );

    // Execute unfreeze on account
    executor.execute_and_apply(
        association.signed_script_txn(encode_unfreeze_account(*account.address()), 4),
    );
    // execute rotate key transaction from unfrozen account now succeeds
    let output = &executor.execute_transaction(txn);
    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED)),
    );
}

#[test]
fn dual_attestation_payment() {
    let mut executor = FakeExecutor::from_genesis_file();
    let association = Account::new_association();
    // account that will receive the dual attestation payment
    let payment_receiver = {
        let data = AccountData::new_empty();
        executor.add_account_data(&data);
        data.into_account()
    };
    // account that will send the dual attestation payment
    let payment_sender = {
        let data = AccountData::new(1_000_000, 0);
        executor.add_account_data(&data);
        data.into_account()
    };

    let mut keygen = KeyGen::from_seed([9u8; 32]);
    let (private_key, public_key) = keygen.generate_keypair();
    // apply for the recipient to be a vasp
    executor.execute_and_apply(payment_receiver.signed_script_txn(
        encode_apply_for_root_vasp(vec![], vec![], vec![], public_key.to_bytes().to_vec()),
        0,
    ));

    // approve the request from association account
    executor.execute_and_apply(
        association.signed_script_txn(encode_grant_vasp_account(*payment_receiver.address()), 1),
    );

    // Do the offline protocol: generate a payment id, sign with the receiver's private key, include
    // in transaction from sender's account
    let ref_id = lcs::to_bytes(&7777u64).unwrap();
    // choose an amount above the dual attestation threshold
    let payment_amount = 1_000_000u64;
    // UTF8-encoded string "@@$$LIBRA_ATTEST$$@@" without length prefix
    let mut domain_separator = vec![
        0x40, 0x40, 0x24, 0x24, 0x4C, 0x49, 0x42, 0x52, 0x41, 0x5F, 0x41, 0x54, 0x54, 0x45, 0x53,
        0x54, 0x24, 0x24, 0x40, 0x40,
    ];
    let message = {
        let mut msg = ref_id.clone();
        msg.append(&mut lcs::to_bytes(&payment_sender.address()).unwrap());
        msg.append(&mut lcs::to_bytes(&payment_amount).unwrap());
        msg.append(&mut domain_separator);
        msg
    };
    let signature =
        <Ed25519PrivateKey as SigningKey>::sign_arbitrary_message(&private_key, &message);
    executor.execute_and_apply(payment_sender.signed_script_txn(
        encode_transfer_with_metadata_script(
            account_config::lbr_type_tag(),
            payment_receiver.address(),
            vec![],
            payment_amount,
            ref_id,
            signature.to_bytes().to_vec(),
        ),
        0,
    ));
}

#[test]
fn publish_rotate_shared_ed25519_public_key() {
    let mut executor = FakeExecutor::from_genesis_file();
    let mut publisher = {
        let data = AccountData::new(1_000_000, 0);
        executor.add_account_data(&data);
        data.into_account()
    };
    // generate the key to initialize the SharedEd25519PublicKey resource
    let mut keygen = KeyGen::from_seed([9u8; 32]);
    let (private_key1, public_key1) = keygen.generate_keypair();
    executor.execute_and_apply(publisher.signed_script_txn(
        encode_publish_shared_ed25519_public_key_script(public_key1.to_bytes().to_vec()),
        0,
    ));
    // must rotate the key locally or sending subsequent txes will fail
    publisher.rotate_key(private_key1, public_key1);

    // send another transaction rotating to a new key
    let (private_key2, public_key2) = keygen.generate_keypair();
    executor.execute_and_apply(publisher.signed_script_txn(
        encode_rotate_shared_ed25519_public_key_script(public_key2.to_bytes().to_vec()),
        1,
    ));
    // must rotate the key in account data or sending subsequent txes will fail
    publisher.rotate_key(private_key2, public_key2.clone());

    // test that sending still works
    executor.execute_and_apply(publisher.signed_script_txn(
        encode_rotate_shared_ed25519_public_key_script(public_key2.to_bytes().to_vec()),
        2,
    ));
}
