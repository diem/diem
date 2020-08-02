// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Tests for all of the script encoding functions in language/transaction_builder/lib.rs.
//! Thorough tests that exercise all of the behaviors of the script should live in the language
//! functional tests; these tests are only to ensure that the script encoding functions take the
//! correct types + produce a runnable script.

#![forbid(unsafe_code)]

use crate::{
    account::{self, Account, AccountData},
    common_transactions::rotate_key_txn,
    executor::FakeExecutor,
    gas_costs,
    keygen::KeyGen,
};
use libra_crypto::{ed25519::Ed25519PrivateKey, traits::SigningKey, PrivateKey, Uniform};
use libra_types::{
    account_config,
    transaction::{authenticator::AuthenticationKey, TransactionOutput, TransactionStatus},
    vm_status::{KeptVMStatus, StatusCode},
};
use transaction_builder::*;

const COIN1_THRESHOLD: u64 = 10_000_000_000 / 5;
const BAD_METADATA_SIGNATURE_ERROR_CODE: u64 = 6;
const MISMATCHED_METADATA_SIGNATURE_ERROR_CODE: u64 = 7;

#[test]
fn freeze_unfreeze_account() {
    // create a FakeExecutor with a genesis from file
    let mut executor = FakeExecutor::from_genesis_file();

    let account = Account::new();

    let mut keygen = KeyGen::from_seed([9u8; 32]);
    let (_, cpubkey) = keygen.generate_keypair();

    let blessed = Account::new_blessed_tc();
    let libra_root = Account::new_libra_root();

    executor.execute_and_apply(
        libra_root
            .transaction()
            .script(encode_create_parent_vasp_account_script(
                account_config::coin1_tag(),
                0,
                *account.address(),
                account.auth_key_prefix(),
                vec![],
                vec![],
                cpubkey.to_bytes().to_vec(),
                true,
            ))
            .sequence_number(1)
            .sign(),
    );

    // Execute freeze on account
    executor.execute_and_apply(
        blessed
            .transaction()
            .script(encode_freeze_account_script(3, *account.address()))
            .sequence_number(0)
            .sign(),
    );

    // Attempt rotate key txn from frozen account
    let privkey = Ed25519PrivateKey::generate_for_testing();
    let pubkey = privkey.public_key();
    let new_key_hash = AuthenticationKey::ed25519(&pubkey).to_vec();
    let txn = rotate_key_txn(&account, new_key_hash, 0);

    let output = &executor.execute_transaction(txn.clone());
    assert_eq!(
        output.status(),
        &TransactionStatus::Discard(StatusCode::SENDING_ACCOUNT_FROZEN),
    );

    // Execute unfreeze on account
    executor.execute_and_apply(
        blessed
            .transaction()
            .script(encode_unfreeze_account_script(4, *account.address()))
            .sequence_number(1)
            .sign(),
    );
    // execute rotate key transaction from unfrozen account now succeeds
    let output = &executor.execute_transaction(txn);
    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(KeptVMStatus::Executed),
    );
}

#[test]
fn create_parent_and_child_vasp() {
    let mut executor = FakeExecutor::from_genesis_file();
    let libra_root = Account::new_libra_root();
    let parent = Account::new();
    let child = Account::new();

    let mut keygen = KeyGen::from_seed([9u8; 32]);
    let (_vasp_compliance_private_key, vasp_compliance_public_key) = keygen.generate_keypair();

    // create a parent VASP
    let add_all_currencies = false;
    executor.execute_and_apply(
        libra_root
            .transaction()
            .script(encode_create_parent_vasp_account_script(
                account_config::lbr_type_tag(),
                0,
                *parent.address(),
                parent.auth_key_prefix(),
                vec![],
                vec![],
                vasp_compliance_public_key.to_bytes().to_vec(),
                add_all_currencies,
            ))
            .sequence_number(1)
            .sign(),
    );

    // create a child VASP with a zero balance
    executor.execute_and_apply(
        parent
            .transaction()
            .script(encode_create_child_vasp_account_script(
                account_config::lbr_type_tag(),
                *child.address(),
                child.auth_key_prefix(),
                add_all_currencies,
                0,
            ))
            .sequence_number(0)
            .sign(),
    );
    // check for zero balance
    assert_eq!(
        executor
            .read_balance_resource(&child, account::lbr_currency_code())
            .unwrap()
            .coin(),
        0
    );

    let (_, new_compliance_public_key) = keygen.generate_keypair();
    // rotate parent's base_url and compliance public key
    executor.execute_and_apply(
        parent
            .transaction()
            .script(encode_rotate_dual_attestation_info_script(
                b"new_name".to_vec(),
                new_compliance_public_key.to_bytes().to_vec(),
            ))
            .sequence_number(1)
            .sign(),
    );
}

#[test]
fn create_child_vasp_all_currencies() {
    let mut executor = FakeExecutor::from_genesis_file();
    let libra_root = Account::new_libra_root();
    let dd = Account::new_genesis_account(account_config::testnet_dd_account_address());
    let parent = Account::new();
    let child = Account::new();

    let mut keygen = KeyGen::from_seed([9u8; 32]);
    let (_vasp_compliance_private_key, vasp_compliance_public_key) = keygen.generate_keypair();

    // create a parent VASP
    let add_all_currencies = true;
    executor.execute_and_apply(
        libra_root
            .transaction()
            .script(encode_create_parent_vasp_account_script(
                account_config::coin1_tag(),
                0,
                *parent.address(),
                parent.auth_key_prefix(),
                vec![],
                vec![],
                vasp_compliance_public_key.to_bytes().to_vec(),
                add_all_currencies,
            ))
            .sequence_number(1)
            .sign(),
    );

    let amount = 100;
    // mint to the parent VASP
    executor.execute_and_apply(
        dd.transaction()
            .script(encode_testnet_mint_script(
                account_config::coin1_tag(),
                *parent.address(),
                amount,
            ))
            .sequence_number(0)
            .sign(),
    );

    assert!(executor
        .read_balance_resource(&parent, account::coin1_currency_code())
        .is_some());
    assert!(executor
        .read_balance_resource(&parent, account::coin2_currency_code())
        .is_some());
    assert!(executor
        .read_balance_resource(&parent, account::lbr_currency_code())
        .is_some());

    // create a child VASP with a balance of amount
    executor.execute_and_apply(
        parent
            .transaction()
            .script(encode_create_child_vasp_account_script(
                account_config::coin1_tag(),
                *child.address(),
                child.auth_key_prefix(),
                add_all_currencies,
                amount,
            ))
            .sequence_number(0)
            .max_gas_amount(gas_costs::TXN_RESERVED * 3)
            .sign(),
    );

    assert!(executor
        .read_balance_resource(&parent, account::coin1_currency_code())
        .is_some());
    assert!(executor
        .read_balance_resource(&child, account::coin2_currency_code())
        .is_some());
    assert!(executor
        .read_balance_resource(&child, account::lbr_currency_code())
        .is_some());
}

#[test]
fn create_child_vasp_with_balance() {
    let mut executor = FakeExecutor::from_genesis_file();
    let libra_root = Account::new_libra_root();
    let dd = Account::new_genesis_account(account_config::testnet_dd_account_address());
    let parent = Account::new();
    let child = Account::new();

    let mut keygen = KeyGen::from_seed([9u8; 32]);
    let (_vasp_compliance_private_key, vasp_compliance_public_key) = keygen.generate_keypair();

    // create a parent VASP
    let add_all_currencies = true;
    executor.execute_and_apply(
        libra_root
            .transaction()
            .script(encode_create_parent_vasp_account_script(
                account_config::coin1_tag(),
                0,
                *parent.address(),
                parent.auth_key_prefix(),
                vec![],
                vec![],
                vasp_compliance_public_key.to_bytes().to_vec(),
                add_all_currencies,
            ))
            .sequence_number(1)
            .sign(),
    );

    let amount = 100;
    // mint to the parent VASP
    executor.execute_and_apply(
        dd.transaction()
            .script(encode_testnet_mint_script(
                account_config::coin1_tag(),
                *parent.address(),
                amount,
            ))
            .sequence_number(0)
            .sign(),
    );

    assert_eq!(
        executor
            .read_balance_resource(&parent, account::coin1_currency_code())
            .unwrap()
            .coin(),
        amount
    );

    // create a child VASP with a balance of amount
    executor.execute_and_apply(
        parent
            .transaction()
            .script(encode_create_child_vasp_account_script(
                account_config::coin1_tag(),
                *child.address(),
                child.auth_key_prefix(),
                add_all_currencies,
                amount,
            ))
            .sequence_number(0)
            .max_gas_amount(gas_costs::TXN_RESERVED * 3)
            .sign(),
    );

    // check balance
    assert_eq!(
        executor
            .read_balance_resource(&child, account::coin1_currency_code())
            .unwrap()
            .coin(),
        amount
    );
}

#[test]
fn dual_attestation_payment() {
    let mut executor = FakeExecutor::from_genesis_file();
    // account that will receive the dual attestation payment
    let payment_receiver = Account::new();
    let payment_sender = Account::new();
    let sender_child = Account::new();
    let libra_root = Account::new_libra_root();
    let dd = Account::new_genesis_account(account_config::testnet_dd_account_address());
    let mut keygen = KeyGen::from_seed([9u8; 32]);
    let (sender_vasp_compliance_private_key, sender_vasp_compliance_public_key) =
        keygen.generate_keypair();
    let (receiver_vasp_compliance_private_key, receiver_vasp_compliance_public_key) =
        keygen.generate_keypair();

    let payment_amount = COIN1_THRESHOLD;

    executor.execute_and_apply(
        libra_root
            .transaction()
            .script(encode_create_parent_vasp_account_script(
                account_config::coin1_tag(),
                0,
                *payment_sender.address(),
                payment_sender.auth_key_prefix(),
                vec![],
                vec![],
                sender_vasp_compliance_public_key.to_bytes().to_vec(),
                false,
            ))
            .sequence_number(1)
            .sign(),
    );

    executor.execute_and_apply(
        libra_root
            .transaction()
            .script(encode_create_parent_vasp_account_script(
                account_config::coin1_tag(),
                0,
                *payment_receiver.address(),
                payment_receiver.auth_key_prefix(),
                vec![],
                vec![],
                receiver_vasp_compliance_public_key.to_bytes().to_vec(),
                false,
            ))
            .sequence_number(2)
            .sign(),
    );

    // give `payment_sender` enough coins to make a `num_payments` payments at or above the dual
    // attestation threshold. We have to split this into multiple txes because DD -> VASP txes are
    // subject to the travel rule too!
    let num_payments = 5;
    for i in 0..num_payments {
        executor.execute_and_apply(
            dd.transaction()
                .script(encode_testnet_mint_script(
                    account_config::coin1_tag(),
                    *payment_sender.address(),
                    COIN1_THRESHOLD - 1,
                ))
                .sequence_number(i)
                .sign(),
        );
    }

    // create a child VASP with a balance of amount
    executor.execute_and_apply(
        payment_sender
            .transaction()
            .script(encode_create_child_vasp_account_script(
                account_config::coin1_tag(),
                *sender_child.address(),
                sender_child.auth_key_prefix(),
                false,
                10,
            ))
            .sequence_number(0)
            .sign(),
    );
    {
        // Transaction >= 1_000_000 threshold goes through signature verification with valid signature, passes
        // Do the offline protocol: generate a payment id, sign with the receiver's private key, include
        // in transaction from sender's account
        let ref_id = lcs::to_bytes(&7777u64).unwrap();
        // UTF8-encoded string "@@$$LIBRA_ATTEST$$@@" without length prefix
        let mut domain_separator = vec![
            0x40, 0x40, 0x24, 0x24, 0x4C, 0x49, 0x42, 0x52, 0x41, 0x5F, 0x41, 0x54, 0x54, 0x45,
            0x53, 0x54, 0x24, 0x24, 0x40, 0x40,
        ];
        let message = {
            let mut msg = ref_id.clone();
            msg.append(&mut lcs::to_bytes(&payment_sender.address()).unwrap());
            msg.append(&mut lcs::to_bytes(&payment_amount).unwrap());
            msg.append(&mut domain_separator);
            msg
        };
        let signature = <Ed25519PrivateKey as SigningKey>::sign_arbitrary_message(
            &receiver_vasp_compliance_private_key,
            &message,
        );
        let output = executor.execute_and_apply(
            payment_sender
                .transaction()
                .script(encode_peer_to_peer_with_metadata_script(
                    account_config::coin1_tag(),
                    *payment_receiver.address(),
                    payment_amount,
                    ref_id,
                    signature.to_bytes().to_vec(),
                ))
                .sequence_number(1)
                .sign(),
        );
        assert_eq!(output.status().status(), Ok(KeptVMStatus::Executed));
    }
    {
        // transaction >= 1_000_000 (set in DualAttestation.move) threshold goes through signature verification but has an
        // structurally invalid signature. Fails.
        let ref_id = [0u8; 32].to_vec();
        let output = executor.execute_transaction(
            payment_sender
                .transaction()
                .script(encode_peer_to_peer_with_metadata_script(
                    account_config::coin1_tag(),
                    *payment_receiver.address(),
                    payment_amount,
                    ref_id,
                    b"invalid signature".to_vec(),
                ))
                .sequence_number(2)
                .sign(),
        );

        assert!(matches!(
            output.status().status(),
            Ok(KeptVMStatus::MoveAbort(_, BAD_METADATA_SIGNATURE_ERROR_CODE))
        ));
    }

    {
        // transaction >= 1_000_000 threshold goes through signature verification with invalid signature, aborts
        let ref_id = lcs::to_bytes(&9999u64).unwrap();
        // UTF8-encoded string "@@$$LIBRA_ATTEST$$@@" without length prefix
        let mut domain_separator = vec![
            0x40, 0x40, 0x24, 0x24, 0x4C, 0x49, 0x42, 0x52, 0x41, 0x5F, 0x41, 0x54, 0x54, 0x45,
            0x53, 0x54, 0x24, 0x24, 0x40, 0x40,
        ];
        let message = {
            let mut msg = ref_id.clone();
            msg.append(&mut lcs::to_bytes(&payment_sender.address()).unwrap());
            msg.append(&mut lcs::to_bytes(&payment_amount).unwrap());
            msg.append(&mut domain_separator);
            msg
        };
        // Sign with the wrong private key
        let signature = <Ed25519PrivateKey as SigningKey>::sign_arbitrary_message(
            &sender_vasp_compliance_private_key,
            &message,
        );
        let output = executor.execute_transaction(
            payment_sender
                .transaction()
                .script(encode_peer_to_peer_with_metadata_script(
                    account_config::coin1_tag(),
                    *payment_receiver.address(),
                    payment_amount,
                    ref_id,
                    signature.to_bytes().to_vec(),
                ))
                .sequence_number(2)
                .sign(),
        );

        assert!(matches!(
            output.status().status(),
            Ok(KeptVMStatus::MoveAbort(_, MISMATCHED_METADATA_SIGNATURE_ERROR_CODE))
        ));
    }

    {
        // similar, but with empty payment ID (make sure signature is still invalid!)
        let ref_id = vec![];
        // UTF8-encoded string "@@$$LIBRA_ATTEST$$@@" without length prefix
        let mut domain_separator = vec![
            0x40, 0x40, 0x24, 0x24, 0x4C, 0x49, 0x42, 0x52, 0x41, 0x5F, 0x41, 0x54, 0x54, 0x45,
            0x53, 0x54, 0x24, 0x24, 0x40, 0x40,
        ];
        let message = {
            let mut msg = ref_id.clone();
            msg.append(&mut lcs::to_bytes(&payment_sender.address()).unwrap());
            msg.append(&mut lcs::to_bytes(&payment_amount).unwrap());
            msg.append(&mut domain_separator);
            msg
        };
        // Sign with the wrong private key
        let signature = <Ed25519PrivateKey as SigningKey>::sign_arbitrary_message(
            &sender_vasp_compliance_private_key,
            &message,
        );
        let output = executor.execute_transaction(
            payment_sender
                .transaction()
                .script(encode_peer_to_peer_with_metadata_script(
                    account_config::coin1_tag(),
                    *payment_receiver.address(),
                    payment_amount,
                    ref_id,
                    signature.to_bytes().to_vec(),
                ))
                .sequence_number(2)
                .sign(),
        );
        assert!(matches!(
            output.status().status(),
            Ok(KeptVMStatus::MoveAbort(_, MISMATCHED_METADATA_SIGNATURE_ERROR_CODE))
        ));
    }
    {
        // Intra-VASP transaction >= 1000 threshold, should go through with any signature since
        // checking isn't performed on intra-vasp transfers
        // parent->child
        executor.execute_and_apply(
            payment_sender
                .transaction()
                .script(encode_peer_to_peer_with_metadata_script(
                    account_config::coin1_tag(),
                    *sender_child.address(),
                    payment_amount * 2,
                    vec![0],
                    b"what a bad signature".to_vec(),
                ))
                .sequence_number(2)
                .sign(),
        );
    }
    {
        // Checking isn't performed on intra-vasp transfers
        // child->parent
        executor.execute_and_apply(
            sender_child
                .transaction()
                .script(encode_peer_to_peer_with_metadata_script(
                    account_config::coin1_tag(),
                    *payment_sender.address(),
                    payment_amount,
                    vec![0],
                    b"what a bad signature".to_vec(),
                ))
                .sequence_number(0)
                .sign(),
        );
    }
    {
        // Rotate the parent VASP's compliance key and base URL
        let (_, new_compliance_public_key) = keygen.generate_keypair();
        executor.execute_and_apply(
            payment_receiver
                .transaction()
                .script(encode_rotate_dual_attestation_info_script(
                    b"any base_url works".to_vec(),
                    new_compliance_public_key.to_bytes().to_vec(),
                ))
                .sequence_number(0)
                .sign(),
        );
    }
    {
        // This previously succeeded, but should now fail since their public key has changed
        // in transaction from sender's account. This tests to make sure their public key was
        // rotated.
        let ref_id = lcs::to_bytes(&9999u64).unwrap();
        // UTF8-encoded string "@@$$LIBRA_ATTEST$$@@" without length prefix
        let mut domain_separator = vec![
            0x40, 0x40, 0x24, 0x24, 0x4C, 0x49, 0x42, 0x52, 0x41, 0x5F, 0x41, 0x54, 0x54, 0x45,
            0x53, 0x54, 0x24, 0x24, 0x40, 0x40,
        ];
        let message = {
            let mut msg = ref_id.clone();
            msg.append(&mut lcs::to_bytes(&payment_sender.address()).unwrap());
            msg.append(&mut lcs::to_bytes(&payment_amount).unwrap());
            msg.append(&mut domain_separator);
            msg
        };
        let signature = <Ed25519PrivateKey as SigningKey>::sign_arbitrary_message(
            &receiver_vasp_compliance_private_key,
            &message,
        );
        let output = executor.execute_transaction(
            payment_sender
                .transaction()
                .script(encode_peer_to_peer_with_metadata_script(
                    account_config::coin1_tag(),
                    *payment_receiver.address(),
                    payment_amount,
                    ref_id,
                    signature.to_bytes().to_vec(),
                ))
                .sequence_number(3)
                .sign(),
        );
        assert_aborted_with(output, MISMATCHED_METADATA_SIGNATURE_ERROR_CODE)
    }
}

fn assert_aborted_with(output: TransactionOutput, error_code: u64) {
    assert!(matches!(
        output.status().status(),
        Ok(KeptVMStatus::MoveAbort(_, code)) if code == error_code
    ));
}

// Check that DD <-> DD and DD <-> VASP payments over the threshold fail without dual attesation.
#[test]
fn dd_dual_attestation_payments() {
    let mut executor = FakeExecutor::from_genesis_file();
    // account that will receive the dual attestation payment
    let parent_vasp = Account::new();
    let dd1 = Account::new();
    let dd2 = Account::new();
    let libra_root = Account::new_libra_root();
    let blessed = Account::new_blessed_tc();
    let mint_dd = Account::new_genesis_account(account_config::testnet_dd_account_address());
    let mut keygen = KeyGen::from_seed([9u8; 32]);
    let (_parent_vasp_compliance_private_key, parent_vasp_compliance_public_key) =
        keygen.generate_keypair();
    let (_dd1_compliance_private_key, dd1_compliance_public_key) = keygen.generate_keypair();
    let (_dd2_compliance_private_key, dd2_compliance_public_key) = keygen.generate_keypair();

    // create the VASP account
    executor.execute_and_apply(
        libra_root
            .transaction()
            .script(encode_create_parent_vasp_account_script(
                account_config::coin1_tag(),
                0,
                *parent_vasp.address(),
                parent_vasp.auth_key_prefix(),
                vec![],
                vec![],
                parent_vasp_compliance_public_key.to_bytes().to_vec(),
                false,
            ))
            .sequence_number(1)
            .sign(),
    );
    // create the DD1 account
    executor.execute_and_apply(
        blessed
            .transaction()
            .script(encode_create_designated_dealer_script(
                account_config::coin1_tag(),
                0,
                *dd1.address(),
                dd1.auth_key_prefix(),
                vec![],
                vec![],
                dd1_compliance_public_key.to_bytes().to_vec(),
                false,
            ))
            .sequence_number(0)
            .sign(),
    );
    // create the DD2 account
    executor.execute_and_apply(
        blessed
            .transaction()
            .script(encode_create_designated_dealer_script(
                account_config::coin1_tag(),
                0,
                *dd2.address(),
                dd1.auth_key_prefix(),
                vec![],
                vec![],
                dd2_compliance_public_key.to_bytes().to_vec(),
                false,
            ))
            .sequence_number(1)
            .sign(),
    );

    // give DD1 some funds
    executor.execute_and_apply(
        mint_dd
            .transaction()
            .script(encode_testnet_mint_script(
                account_config::coin1_tag(),
                *dd1.address(),
                COIN1_THRESHOLD - 1,
            ))
            .sequence_number(0)
            .sign(),
    );
    executor.execute_and_apply(
        mint_dd
            .transaction()
            .script(encode_testnet_mint_script(
                account_config::coin1_tag(),
                *dd1.address(),
                COIN1_THRESHOLD - 1,
            ))
            .sequence_number(1)
            .sign(),
    );
    // Give VASP some funds
    executor.execute_and_apply(
        mint_dd
            .transaction()
            .script(encode_testnet_mint_script(
                account_config::coin1_tag(),
                *parent_vasp.address(),
                COIN1_THRESHOLD - 1,
            ))
            .sequence_number(2)
            .sign(),
    );
    executor.execute_and_apply(
        mint_dd
            .transaction()
            .script(encode_testnet_mint_script(
                account_config::coin1_tag(),
                *parent_vasp.address(),
                COIN1_THRESHOLD - 1,
            ))
            .sequence_number(3)
            .sign(),
    );

    // DD <-> DD over threshold without attestation fails
    // Checking isn't performed on UHW->VASP
    let output = executor.execute_transaction(
        dd1.transaction()
            .script(encode_peer_to_peer_with_metadata_script(
                account_config::coin1_tag(),
                *dd2.address(),
                COIN1_THRESHOLD,
                vec![0],
                b"what a bad signature".to_vec(),
            ))
            .sequence_number(0)
            .sign(),
    );
    assert_aborted_with(output, BAD_METADATA_SIGNATURE_ERROR_CODE);

    // DD -> VASP over threshold without attestation fails
    let output = executor.execute_transaction(
        dd1.transaction()
            .script(encode_peer_to_peer_with_metadata_script(
                account_config::coin1_tag(),
                *parent_vasp.address(),
                COIN1_THRESHOLD,
                vec![0],
                b"what a bad signature".to_vec(),
            ))
            .sequence_number(0) // didn't apply result of previous tx, so seq doesn't change
            .sign(),
    );
    assert_aborted_with(output, BAD_METADATA_SIGNATURE_ERROR_CODE);

    // VASP -> DD over threshold without attestation fails
    let output = executor.execute_transaction(
        parent_vasp
            .transaction()
            .script(encode_peer_to_peer_with_metadata_script(
                account_config::coin1_tag(),
                *dd1.address(),
                COIN1_THRESHOLD,
                vec![0],
                b"what a bad signature".to_vec(),
            ))
            .sequence_number(0)
            .sign(),
    );
    assert_aborted_with(output, BAD_METADATA_SIGNATURE_ERROR_CODE);
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
    executor.execute_and_apply(
        publisher
            .transaction()
            .script(encode_publish_shared_ed25519_public_key_script(
                public_key1.to_bytes().to_vec(),
            ))
            .sequence_number(0)
            .sign(),
    );
    // must rotate the key locally or sending subsequent txes will fail
    publisher.rotate_key(private_key1, public_key1);

    // send another transaction rotating to a new key
    let (private_key2, public_key2) = keygen.generate_keypair();
    executor.execute_and_apply(
        publisher
            .transaction()
            .script(encode_rotate_shared_ed25519_public_key_script(
                public_key2.to_bytes().to_vec(),
            ))
            .sequence_number(1)
            .sign(),
    );
    // must rotate the key in account data or sending subsequent txes will fail
    publisher.rotate_key(private_key2, public_key2.clone());

    // test that sending still works
    executor.execute_and_apply(
        publisher
            .transaction()
            .script(encode_rotate_shared_ed25519_public_key_script(
                public_key2.to_bytes().to_vec(),
            ))
            .sequence_number(2)
            .sign(),
    );
}

#[test]
fn recovery_address() {
    let mut executor = FakeExecutor::from_genesis_file();
    let libra_root = Account::new_libra_root();

    let parent = Account::new();
    let mut child = Account::new();
    let other_vasp = Account::new();

    let mut keygen = KeyGen::from_seed([9u8; 32]);
    let (_vasp_compliance_private_key, vasp_compliance_public_key) = keygen.generate_keypair();

    // create a parent VASP
    let add_all_currencies = false;
    executor.execute_and_apply(
        libra_root
            .transaction()
            .script(encode_create_parent_vasp_account_script(
                account_config::lbr_type_tag(),
                0,
                *parent.address(),
                parent.auth_key_prefix(),
                vec![],
                vec![],
                vasp_compliance_public_key.to_bytes().to_vec(),
                add_all_currencies,
            ))
            .sequence_number(1)
            .sign(),
    );

    // create a child VASP with a zero balance
    executor.execute_and_apply(
        parent
            .transaction()
            .script(encode_create_child_vasp_account_script(
                account_config::lbr_type_tag(),
                *child.address(),
                child.auth_key_prefix(),
                add_all_currencies,
                0,
            ))
            .sequence_number(0)
            .sign(),
    );

    // publish a recovery address under the parent
    executor.execute_and_apply(
        parent
            .transaction()
            .script(encode_create_recovery_address_script())
            .sequence_number(1)
            .sign(),
    );

    // delegate authentication key of the child
    executor.execute_and_apply(
        child
            .transaction()
            .script(encode_add_recovery_rotation_capability_script(
                *parent.address(),
            ))
            .sequence_number(0)
            .sign(),
    );

    // rotate authentication key from the parent
    let (privkey1, pubkey1) = keygen.generate_keypair();
    let new_authentication_key1 = AuthenticationKey::ed25519(&pubkey1).to_vec();
    executor.execute_and_apply(
        parent
            .transaction()
            .script(
                encode_rotate_authentication_key_with_recovery_address_script(
                    *parent.address(),
                    *child.address(),
                    new_authentication_key1,
                ),
            )
            .sequence_number(2)
            .sign(),
    );

    // rotate authentication key from the child
    let (_, pubkey2) = keygen.generate_keypair();
    let new_authentication_key2 = AuthenticationKey::ed25519(&pubkey2).to_vec();
    child.rotate_key(privkey1, pubkey1);
    executor.execute_and_apply(
        child
            .transaction()
            .script(
                encode_rotate_authentication_key_with_recovery_address_script(
                    *parent.address(),
                    *child.address(),
                    new_authentication_key2,
                ),
            )
            .sequence_number(1)
            .sign(),
    );

    // create another VASP unrelated to parent/child
    let add_all_currencies = false;
    executor.execute_and_apply(
        libra_root
            .transaction()
            .script(encode_create_parent_vasp_account_script(
                account_config::lbr_type_tag(),
                0,
                *other_vasp.address(),
                other_vasp.auth_key_prefix(),
                vec![],
                vec![],
                vasp_compliance_public_key.to_bytes().to_vec(),
                add_all_currencies,
            ))
            .sequence_number(2)
            .sign(),
    );

    // try to delegate other_vasp's rotation cap to child--should abort
    let output = executor.execute_transaction(
        other_vasp
            .transaction()
            .script(encode_add_recovery_rotation_capability_script(
                *parent.address(),
            ))
            .sequence_number(0)
            .sign(),
    );
    assert_aborted_with(output, 3);

    // try to rotate child's key from other_vasp--should abort
    let (_, pubkey3) = keygen.generate_keypair();
    let new_authentication_key3 = AuthenticationKey::ed25519(&pubkey3).to_vec();
    let output = executor.execute_transaction(
        other_vasp
            .transaction()
            .script(
                encode_rotate_authentication_key_with_recovery_address_script(
                    *parent.address(),
                    *child.address(),
                    new_authentication_key3,
                ),
            )
            .sequence_number(0)
            .sign(),
    );
    assert_aborted_with(output, 2);
}

#[test]
fn account_limits() {
    let mut executor = FakeExecutor::from_genesis_file();
    let mut keygen = KeyGen::from_seed([9u8; 32]);

    let vasp_a = Account::new();
    let vasp_b = Account::new();
    let vasp_a_child = Account::new();
    let vasp_b_child = Account::new();
    let libra_root = Account::new_libra_root();
    let tc = Account::new_blessed_tc();
    let dd = Account::new_genesis_account(account_config::testnet_dd_account_address());

    let mint_amount = 1_000_000;
    let window_micros = 86400000000;
    let ttl = window_micros;

    let (_, vasp_a_cpubkey) = keygen.generate_keypair();
    let (_, vasp_b_cpubkey) = keygen.generate_keypair();

    // Create vasp accounts
    executor.execute_and_apply(
        libra_root
            .transaction()
            .script(encode_create_parent_vasp_account_script(
                account_config::coin1_tag(),
                0,
                *vasp_a.address(),
                vasp_a.auth_key_prefix(),
                vec![],
                vec![],
                vasp_a_cpubkey.to_bytes().to_vec(),
                true,
            ))
            .sequence_number(1)
            .ttl(ttl)
            .sign(),
    );
    executor.execute_and_apply(
        libra_root
            .transaction()
            .script(encode_create_parent_vasp_account_script(
                account_config::coin1_tag(),
                0,
                *vasp_b.address(),
                vasp_b.auth_key_prefix(),
                vec![],
                vec![],
                vasp_b_cpubkey.to_bytes().to_vec(),
                true,
            ))
            .sequence_number(2)
            .ttl(ttl)
            .sign(),
    );

    // Create child vasp accounts
    executor.execute_and_apply(
        vasp_a
            .transaction()
            .script(encode_create_child_vasp_account_script(
                account_config::coin1_tag(),
                *vasp_a_child.address(),
                vasp_a_child.auth_key_prefix(),
                true,
                0,
            ))
            .sequence_number(0)
            .ttl(ttl)
            .sign(),
    );
    executor.execute_and_apply(
        vasp_b
            .transaction()
            .script(encode_create_child_vasp_account_script(
                account_config::coin1_tag(),
                *vasp_b_child.address(),
                vasp_b_child.auth_key_prefix(),
                true,
                0,
            ))
            .sequence_number(0)
            .ttl(ttl)
            .sign(),
    );

    // mint money to both vasp A & B
    executor.execute_and_apply(
        dd.transaction()
            .script(encode_testnet_mint_script(
                account_config::coin1_tag(),
                *vasp_a.address(),
                2 * mint_amount,
            ))
            .sequence_number(0)
            .ttl(ttl)
            .sign(),
    );
    executor.execute_and_apply(
        dd.transaction()
            .script(encode_testnet_mint_script(
                account_config::coin1_tag(),
                *vasp_b.address(),
                2 * mint_amount,
            ))
            .sequence_number(1)
            .ttl(ttl)
            .sign(),
    );

    executor.execute_and_apply(
        vasp_a
            .transaction()
            .script(encode_publish_account_limit_definition_script(
                account_config::coin1_tag(),
            ))
            .sequence_number(1)
            .ttl(ttl)
            .sign(),
    );

    executor.execute_and_apply(
        tc.transaction()
            .script(encode_update_account_limit_window_info_script(
                account_config::coin1_tag(),
                *vasp_a.address(),
                0,
                *vasp_a.address(),
            ))
            .sequence_number(0)
            .ttl(ttl)
            .sign(),
    );

    ///////////////////////////////////////////////////////////////////////////
    // Inflow tests
    /////////////////////////////////////////////////////////////////////////////

    // Set vasp A's inflow limit to half of what we just minted them
    executor.execute_and_apply(
        tc.transaction()
            .script(encode_update_account_limit_definition_script(
                account_config::coin1_tag(),
                *vasp_a.address(),
                0,
                mint_amount,
                0,
                0,
                0,
            ))
            .sequence_number(1)
            .ttl(ttl)
            .sign(),
    );

    {
        // Now try and pay in to vasp A; fails since inflow is exceeded
        let output = executor.execute_transaction(
            vasp_b
                .transaction()
                .script(encode_peer_to_peer_with_metadata_script(
                    account_config::coin1_tag(),
                    *vasp_a.address(),
                    mint_amount + 1,
                    vec![],
                    vec![],
                ))
                .sequence_number(1)
                .ttl(ttl)
                .sign(),
        );
        assert_aborted_with(output, 3);
    }

    {
        // Now try and pay in to child of vasp A; fails since inflow is exceeded
        let output = executor.execute_transaction(
            vasp_b
                .transaction()
                .script(encode_peer_to_peer_with_metadata_script(
                    account_config::coin1_tag(),
                    *vasp_a_child.address(),
                    mint_amount + 1,
                    vec![],
                    vec![],
                ))
                .sequence_number(1)
                .ttl(ttl)
                .sign(),
        );
        assert_aborted_with(output, 3);
    }

    // Intra-vasp transfer isn't limited
    executor.execute_and_apply(
        vasp_a
            .transaction()
            .script(encode_peer_to_peer_with_metadata_script(
                account_config::coin1_tag(),
                *vasp_a_child.address(),
                mint_amount + 1,
                vec![],
                vec![],
            ))
            .sequence_number(2)
            .ttl(ttl)
            .sign(),
    );

    // Only inflow is limited; can send from vasp a still
    executor.execute_and_apply(
        vasp_a_child
            .transaction()
            .script(encode_peer_to_peer_with_metadata_script(
                account_config::coin1_tag(),
                *vasp_b_child.address(),
                mint_amount + 1,
                vec![],
                vec![],
            ))
            .sequence_number(0)
            .ttl(ttl)
            .sign(),
    );

    // The previous mints don't count in this window since it wasn't a vasp->vasp transfer
    executor.execute_and_apply(
        vasp_b_child
            .transaction()
            .script(encode_peer_to_peer_with_metadata_script(
                account_config::coin1_tag(),
                *vasp_a_child.address(),
                mint_amount,
                vec![],
                vec![],
            ))
            .sequence_number(0)
            .ttl(ttl)
            .sign(),
    );

    {
        // DD deposit fails since vasp A is at inflow limit
        let output = executor.execute_transaction(
            dd.transaction()
                .script(encode_testnet_mint_script(
                    account_config::coin1_tag(),
                    *vasp_a_child.address(),
                    1,
                ))
                .sequence_number(2)
                .ttl(ttl)
                .sign(),
        );
        assert_aborted_with(output, 3);

        // Reset the window
        let prev_block_time = executor.get_block_time();
        executor.set_block_time(prev_block_time + window_micros);
        executor.new_block();

        // DD deposit now succeeds since window is reset
        let output = executor.execute_transaction(
            dd.transaction()
                .script(encode_testnet_mint_script(
                    account_config::coin1_tag(),
                    *vasp_a_child.address(),
                    1,
                ))
                .sequence_number(2)
                .ttl(ttl)
                .sign(),
        );
        assert_eq!(output.status().status(), Ok(KeptVMStatus::Executed));
    }

    ///////////////////////////////////////////////////////////////////////////
    // Outflow tests
    /////////////////////////////////////////////////////////////////////////////

    // Set vasp A's outflow to 1000
    executor.execute_and_apply(
        tc.transaction()
            .script(encode_update_account_limit_definition_script(
                account_config::coin1_tag(),
                *vasp_a.address(),
                0,
                std::u64::MAX, // unlimit inflow
                1000,          // set outflow to 1000
                0,
                0,
            ))
            .sequence_number(2)
            .ttl(ttl)
            .sign(),
    );

    // Intra-vasp transfer isn't limited
    executor.execute_and_apply(
        vasp_a
            .transaction()
            .script(encode_peer_to_peer_with_metadata_script(
                account_config::coin1_tag(),
                *vasp_a_child.address(),
                1001,
                vec![],
                vec![],
            ))
            .sequence_number(3)
            .ttl(ttl)
            .sign(),
    );

    // Can send up to the limit inter-vasp:
    executor.execute_and_apply(
        vasp_a_child
            .transaction()
            .script(encode_peer_to_peer_with_metadata_script(
                account_config::coin1_tag(),
                *vasp_b_child.address(),
                1000,
                vec![],
                vec![],
            ))
            .sequence_number(1)
            .ttl(ttl)
            .sign(),
    );

    {
        // Inter-vasp transfer is limited
        let output = executor.execute_transaction(
            vasp_a
                .transaction()
                .script(encode_peer_to_peer_with_metadata_script(
                    account_config::coin1_tag(),
                    *vasp_b.address(),
                    1,
                    vec![],
                    vec![],
                ))
                .sequence_number(4)
                .ttl(ttl)
                .sign(),
        );
        assert_aborted_with(output, 6);
    }

    {
        // Inter-vasp transfer is limited; holds between children too
        let output = executor.execute_transaction(
            vasp_a_child
                .transaction()
                .script(encode_peer_to_peer_with_metadata_script(
                    account_config::coin1_tag(),
                    *vasp_b_child.address(),
                    1,
                    vec![],
                    vec![],
                ))
                .sequence_number(2)
                .ttl(ttl)
                .sign(),
        );
        assert_aborted_with(output, 6);
    }

    {
        // vasp->anything transfer is limited
        let output = executor.execute_transaction(
            vasp_a_child
                .transaction()
                .script(encode_peer_to_peer_with_metadata_script(
                    account_config::coin1_tag(),
                    *dd.address(),
                    1,
                    vec![],
                    vec![],
                ))
                .sequence_number(2)
                .ttl(ttl)
                .sign(),
        );
        assert_aborted_with(output, 6);

        // update block time
        let prev_block_time = executor.get_block_time();
        executor.set_block_time(prev_block_time + window_micros);
        executor.new_block();

        let output = executor.execute_transaction(
            vasp_a_child
                .transaction()
                .script(encode_peer_to_peer_with_metadata_script(
                    account_config::coin1_tag(),
                    *dd.address(),
                    1,
                    vec![],
                    vec![],
                ))
                .sequence_number(2)
                .ttl(window_micros)
                .ttl(ttl)
                .sign(),
        );
        assert_eq!(output.status().status(), Ok(KeptVMStatus::Executed),);
    }

    ///////////////////////////////////////////////////////////////////////////
    // Holding tests
    /////////////////////////////////////////////////////////////////////////////

    // Set vasp A's max holding to its current balance across all accounts
    {
        let a_parent_balance = executor
            .read_balance_resource(&vasp_a, account::coin1_currency_code())
            .unwrap()
            .coin();
        let a_child_balance = executor
            .read_balance_resource(&vasp_a_child, account::coin1_currency_code())
            .unwrap()
            .coin();
        let a_balance = a_parent_balance + a_child_balance;
        executor.execute_and_apply(
            tc.transaction()
                .script(encode_update_account_limit_definition_script(
                    account_config::coin1_tag(),
                    *vasp_a.address(),
                    0,
                    0,
                    std::u64::MAX, // unlimit outflow
                    a_balance,     // set max holding to the current balance of A
                    0,
                ))
                .sequence_number(3)
                .ttl(ttl)
                .sign(),
        );
        // TC needs to set the current aggregate balance for vasp a's window
        executor.execute_and_apply(
            tc.transaction()
                .script(encode_update_account_limit_window_info_script(
                    account_config::coin1_tag(),
                    *vasp_a.address(),
                    a_balance,
                    *vasp_a.address(),
                ))
                .sequence_number(4)
                .ttl(ttl)
                .sign(),
        );
    }

    // inter-vasp: fails since limit is set at A's current balance
    {
        let output = executor.execute_transaction(
            vasp_b
                .transaction()
                .script(encode_peer_to_peer_with_metadata_script(
                    account_config::coin1_tag(),
                    *vasp_a_child.address(),
                    1,
                    vec![],
                    vec![],
                ))
                .sequence_number(1)
                .ttl(ttl)
                .sign(),
        );
        assert_aborted_with(output, 3);
    }

    // Fine since A can still send
    executor.execute_and_apply(
        vasp_a
            .transaction()
            .script(encode_peer_to_peer_with_metadata_script(
                account_config::coin1_tag(),
                *vasp_b_child.address(),
                10,
                vec![],
                vec![],
            ))
            .sequence_number(4)
            .ttl(ttl)
            .sign(),
    );

    // inter-vasp: OK since A's total balance = limit - 10
    executor.execute_and_apply(
        vasp_b
            .transaction()
            .script(encode_peer_to_peer_with_metadata_script(
                account_config::coin1_tag(),
                *vasp_a_child.address(),
                10,
                vec![],
                vec![],
            ))
            .sequence_number(1)
            .ttl(ttl)
            .sign(),
    );

    {
        // inter-vasp: should now fail again
        let output = executor.execute_transaction(
            vasp_b
                .transaction()
                .script(encode_peer_to_peer_with_metadata_script(
                    account_config::coin1_tag(),
                    *vasp_a_child.address(),
                    1,
                    vec![],
                    vec![],
                ))
                .sequence_number(2)
                .ttl(ttl)
                .sign(),
        );
        assert_aborted_with(output, 3);
    }

    // intra-vasp: OK since it isn't checked/contributes to the total balance
    executor.execute_and_apply(
        vasp_a_child
            .transaction()
            .script(encode_peer_to_peer_with_metadata_script(
                account_config::coin1_tag(),
                *vasp_a.address(),
                1100,
                vec![],
                vec![],
            ))
            .sequence_number(2)
            .ttl(ttl)
            .sign(),
    );

    {
        // DD deposit fails since vasp A is at holding limit
        let output = executor.execute_transaction(
            dd.transaction()
                .script(encode_testnet_mint_script(
                    account_config::coin1_tag(),
                    *vasp_a_child.address(),
                    1,
                ))
                .sequence_number(2)
                .ttl(ttl)
                .sign(),
        );
        assert_aborted_with(output, 3);

        // Reset window
        let prev_block_time = executor.get_block_time();
        executor.set_block_time(prev_block_time + window_micros);
        executor.new_block();

        // DD deposit fails since vasp A is at holding limit
        // and because holdings are not reset from one window to the next.
        let output = executor.execute_transaction(
            dd.transaction()
                .script(encode_testnet_mint_script(
                    account_config::coin1_tag(),
                    *vasp_a_child.address(),
                    1,
                ))
                .sequence_number(2)
                .ttl(ttl)
                .sign(),
        );
        assert_aborted_with(output, 3);
    }
}

#[test]
fn add_child_currencies() {
    let mut executor = FakeExecutor::from_genesis_file();
    let mut keygen = KeyGen::from_seed([9u8; 32]);

    let vasp_a = Account::new();
    let vasp_a_child1 = Account::new();
    let vasp_a_child2 = Account::new();
    let vasp_b = Account::new();
    let vasp_b_child1 = Account::new();
    let vasp_b_child2 = Account::new();
    let libra_root = Account::new_libra_root();

    let (_, vasp_a_cpubkey) = keygen.generate_keypair();
    let (_, vasp_b_cpubkey) = keygen.generate_keypair();

    executor.execute_and_apply(
        libra_root
            .transaction()
            .script(encode_create_parent_vasp_account_script(
                account_config::coin1_tag(),
                0,
                *vasp_a.address(),
                vasp_a.auth_key_prefix(),
                vec![],
                vec![],
                vasp_a_cpubkey.to_bytes().to_vec(),
                false,
            ))
            .sequence_number(1)
            .sign(),
    );

    // Adding a child with the same currency is no issue
    executor.execute_and_apply(
        vasp_a
            .transaction()
            .script(encode_create_child_vasp_account_script(
                account_config::coin1_tag(),
                *vasp_a_child1.address(),
                vasp_a_child1.auth_key_prefix(),
                false,
                0,
            ))
            .sequence_number(0)
            .sign(),
    );

    {
        // Adding a child with add_all_currencies = true will error since the parent doesn't have limits defintions for the other currencies.
        let output = executor.execute_transaction(
            vasp_a
                .transaction()
                .script(encode_create_child_vasp_account_script(
                    account_config::coin1_tag(),
                    *vasp_a_child2.address(),
                    vasp_a_child2.auth_key_prefix(),
                    true,
                    0,
                ))
                .sequence_number(1)
                .sign(),
        );
        assert_aborted_with(output, 4);
    }

    {
        // Adding a child with a different currency from one that the parent has a limits definition for will abort as well
        let output = executor.execute_transaction(
            vasp_a
                .transaction()
                .script(encode_create_child_vasp_account_script(
                    account_config::type_tag_for_currency_code(account::coin2_currency_code()),
                    *vasp_a_child2.address(),
                    vasp_a_child2.auth_key_prefix(),
                    false,
                    0,
                ))
                .sequence_number(1)
                .sign(),
        );
        assert_aborted_with(output, 4);
    }

    executor.execute_and_apply(
        vasp_a
            .transaction()
            .script(encode_add_currency_to_account_script(
                account_config::type_tag_for_currency_code(account::coin2_currency_code()),
            ))
            .sequence_number(1)
            .sign(),
    );
    // This now works since the parent now has limits for this currency
    executor.execute_and_apply(
        vasp_a
            .transaction()
            .script(encode_create_child_vasp_account_script(
                account_config::type_tag_for_currency_code(account::coin2_currency_code()),
                *vasp_a_child2.address(),
                vasp_a_child2.auth_key_prefix(),
                false,
                0,
            ))
            .sequence_number(2)
            .sign(),
    );

    ///////////////////////////////////////////////////////////////////////////
    // Now make a parent with all currencies, and make sure the children are fine
    ///////////////////////////////////////////////////////////////////////////

    executor.execute_and_apply(
        libra_root
            .transaction()
            .script(encode_create_parent_vasp_account_script(
                account_config::coin1_tag(),
                0,
                *vasp_b.address(),
                vasp_b.auth_key_prefix(),
                vec![],
                vec![],
                vasp_b_cpubkey.to_bytes().to_vec(),
                true,
            ))
            .sequence_number(2)
            .sign(),
    );

    // Adding a child with the same currency  and all other currencies  isn't an issue
    executor.execute_and_apply(
        vasp_b
            .transaction()
            .script(encode_create_child_vasp_account_script(
                account_config::coin1_tag(),
                *vasp_b_child1.address(),
                vasp_b_child1.auth_key_prefix(),
                true,
                0,
            ))
            .sequence_number(0)
            .sign(),
    );
    // Adding a child with a different currency than the parent VASP is OK
    executor.execute_and_apply(
        vasp_b
            .transaction()
            .script(encode_create_child_vasp_account_script(
                account_config::type_tag_for_currency_code(account::coin2_currency_code()),
                *vasp_b_child2.address(),
                vasp_b_child2.auth_key_prefix(),
                false,
                0,
            ))
            .sequence_number(1)
            .sign(),
    );
}
