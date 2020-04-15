// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Tests for all of the script encoding functions in language/transaction_builder/lib.rs.
//! Thorough tests that exercise all of the behaviors of the script should live in the language
//! functional tests; these tests are only to ensure that the script encoding functions take the
//! correct types + produce a runnable script.

#![forbid(unsafe_code)]

use crate::{
    account::{Account, AccountData},
    executor::FakeExecutor,
    keygen::KeyGen,
};
use libra_crypto::{hash::HashValue, traits::TSigningKey};
use libra_types::account_config;
use transaction_builder::*;

#[test]
fn register_preburn_burn() {
    // create a FakeExecutor with a genesis from file
    let mut executor = FakeExecutor::from_genesis_file();
    // association account to do the actual burning
    let association = Account::new_association();

    // account to initiate preburning
    let preburner = {
        let data = AccountData::new(1_000_000, 0);
        executor.add_account_data(&data);
        data.into_account()
    };

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
        1,
    ));
    // Complete the second request by cancelling
    executor.execute_and_apply(association.signed_script_txn(
        encode_cancel_burn_script(account_config::lbr_type_tag(), *preburner.address()),
        2,
    ));
}

#[test]
fn approved_payment() {
    let mut executor = FakeExecutor::from_genesis_file();
    // account that will receive the approved payment
    let payment_receiver = {
        let data = AccountData::new(1_000_000, 0);
        executor.add_account_data(&data);
        data.into_account()
    };
    // account that will send the approved payment
    let payment_sender = {
        let data = AccountData::new(1_000_000, 0);
        executor.add_account_data(&data);
        data.into_account()
    };

    // Register the receiver account
    let mut keygen = KeyGen::from_seed([9u8; 32]);
    let (private_key, public_key) = keygen.generate_keypair();
    executor.execute_and_apply(payment_receiver.signed_script_txn(
        encode_register_approved_payment_script(public_key.to_bytes().to_vec()),
        0,
    ));

    // Do the offline protocol: generate a payment id, sign with the receiver's private key, include
    // in transaction from sender's account
    let payment_id = 9999;
    let message = HashValue::from_sha3_256(&lcs::to_bytes(&payment_id).expect("couldn't hash"));
    let signature = private_key.sign_message(&message);
    executor.execute_and_apply(payment_sender.signed_script_txn(
        encode_approved_payment_script(
            account_config::lbr_type_tag(),
            *payment_receiver.address(),
            100,
            message.to_vec(),
            signature.to_bytes().to_vec(),
        ),
        0,
    ));
}
