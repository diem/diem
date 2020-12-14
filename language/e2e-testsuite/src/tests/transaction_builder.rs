// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Tests for all of the script encoding functions in language/transaction_builder/lib.rs.
//! Thorough tests that exercise all of the behaviors of the script should live in the language
//! functional tests; these tests are only to ensure that the script encoding functions take the
//! correct types + produce a runnable script.

#![forbid(unsafe_code)]

use diem_crypto::{ed25519::Ed25519PrivateKey, traits::SigningKey, PrivateKey, Uniform};
use diem_types::{
    account_address::AccountAddress,
    account_config,
    transaction::{authenticator::AuthenticationKey, Script, TransactionOutput, TransactionStatus},
    vm_status::{KeptVMStatus, StatusCode},
};
use language_e2e_tests::{
    account::{self, Account},
    common_transactions::rotate_key_txn,
    currencies, current_function_name,
    executor::FakeExecutor,
    gas_costs,
    keygen::KeyGen,
};
use move_core_types::language_storage::TypeTag;
use transaction_builder::*;

const XUS_THRESHOLD: u64 = 10_000_000_000 / 5;
const BAD_METADATA_SIGNATURE_ERROR_CODE: u64 = 775;
const MISMATCHED_METADATA_SIGNATURE_ERROR_CODE: u64 = 1031;
const PAYEE_COMPLIANCE_KEY_NOT_SET_ERROR_CODE: u64 = 1281;

#[test]
fn freeze_unfreeze_account() {
    // create a FakeExecutor with a genesis from file
    let mut executor = FakeExecutor::from_genesis_file();
    executor.set_golden_file(current_function_name!());

    let account = executor.create_raw_account();

    let blessed = Account::new_blessed_tc();

    executor.execute_and_apply(
        blessed
            .transaction()
            .script(encode_create_parent_vasp_account_script(
                account_config::xus_tag(),
                0,
                *account.address(),
                account.auth_key_prefix(),
                vec![],
                true,
            ))
            .sequence_number(0)
            .sign(),
    );

    // Execute freeze on account
    executor.execute_and_apply(
        blessed
            .transaction()
            .script(encode_freeze_account_script(3, *account.address()))
            .sequence_number(1)
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
            .sequence_number(2)
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
    executor.set_golden_file(current_function_name!());

    let blessed = Account::new_blessed_tc();
    let parent = executor.create_raw_account();
    let child = executor.create_raw_account();

    let mut keygen = KeyGen::from_seed([9u8; 32]);

    // create a parent VASP
    let add_all_currencies = false;
    executor.execute_and_apply(
        blessed
            .transaction()
            .script(encode_create_parent_vasp_account_script(
                account_config::xus_tag(),
                0,
                *parent.address(),
                parent.auth_key_prefix(),
                vec![],
                add_all_currencies,
            ))
            .sequence_number(0)
            .sign(),
    );

    // create a child VASP with a zero balance
    executor.execute_and_apply(
        parent
            .transaction()
            .script(encode_create_child_vasp_account_script(
                account_config::xus_tag(),
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
            .read_balance_resource(&child, account::xus_currency_code())
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
    executor.set_golden_file(current_function_name!());

    let blessed = Account::new_blessed_tc();
    let dd = Account::new_genesis_account(account_config::testnet_dd_account_address());
    let parent = executor.create_raw_account();
    let child = executor.create_raw_account();

    // create a parent VASP
    let add_all_currencies = true;
    executor.execute_and_apply(
        blessed
            .transaction()
            .script(encode_create_parent_vasp_account_script(
                account_config::xus_tag(),
                0,
                *parent.address(),
                parent.auth_key_prefix(),
                vec![],
                add_all_currencies,
            ))
            .sequence_number(0)
            .sign(),
    );

    let amount = 100;
    // mint to the parent VASP
    executor.execute_and_apply(
        dd.transaction()
            .script(encode_peer_to_peer_with_metadata_script(
                account_config::xus_tag(),
                *parent.address(),
                amount,
                vec![],
                vec![],
            ))
            .sequence_number(0)
            .sign(),
    );

    assert!(executor
        .read_balance_resource(&parent, account::xus_currency_code())
        .is_some());

    // create a child VASP with a balance of amount
    executor.execute_and_apply(
        parent
            .transaction()
            .script(encode_create_child_vasp_account_script(
                account_config::xus_tag(),
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
        .read_balance_resource(&parent, account::xus_currency_code())
        .is_some());
}

#[test]
fn create_child_vasp_with_balance() {
    let mut executor = FakeExecutor::from_genesis_file();
    executor.set_golden_file(current_function_name!());

    let blessed = Account::new_blessed_tc();
    let dd = Account::new_genesis_account(account_config::testnet_dd_account_address());
    let parent = executor.create_raw_account();
    let child = executor.create_raw_account();

    // create a parent VASP
    let add_all_currencies = true;
    executor.execute_and_apply(
        blessed
            .transaction()
            .script(encode_create_parent_vasp_account_script(
                account_config::xus_tag(),
                0,
                *parent.address(),
                parent.auth_key_prefix(),
                vec![],
                add_all_currencies,
            ))
            .sequence_number(0)
            .sign(),
    );

    let amount = 100;
    // mint to the parent VASP
    executor.execute_and_apply(
        dd.transaction()
            .script(encode_peer_to_peer_with_metadata_script(
                account_config::xus_tag(),
                *parent.address(),
                amount,
                vec![],
                vec![],
            ))
            .sequence_number(0)
            .sign(),
    );

    assert_eq!(
        executor
            .read_balance_resource(&parent, account::xus_currency_code())
            .unwrap()
            .coin(),
        amount
    );

    // create a child VASP with a balance of amount
    executor.execute_and_apply(
        parent
            .transaction()
            .script(encode_create_child_vasp_account_script(
                account_config::xus_tag(),
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
            .read_balance_resource(&child, account::xus_currency_code())
            .unwrap()
            .coin(),
        amount
    );
}

#[test]
fn dual_attestation_payment() {
    let mut executor = FakeExecutor::from_genesis_file();
    executor.set_golden_file(current_function_name!());

    // account that will receive the dual attestation payment
    let payment_receiver = executor.create_raw_account();
    let payment_sender = executor.create_raw_account();
    let sender_child = executor.create_raw_account();
    let payee_child = executor.create_raw_account();
    let blessed = Account::new_blessed_tc();
    let dd = Account::new_genesis_account(account_config::testnet_dd_account_address());
    let mut keygen = KeyGen::from_seed([9u8; 32]);
    let (sender_vasp_compliance_private_key, _) = keygen.generate_keypair();
    let (receiver_vasp_compliance_private_key, receiver_vasp_compliance_public_key) =
        keygen.generate_keypair();

    let payment_amount = XUS_THRESHOLD;

    executor.execute_and_apply(
        blessed
            .transaction()
            .script(encode_create_parent_vasp_account_script(
                account_config::xus_tag(),
                0,
                *payment_sender.address(),
                payment_sender.auth_key_prefix(),
                vec![],
                false,
            ))
            .sequence_number(0)
            .sign(),
    );

    executor.execute_and_apply(
        blessed
            .transaction()
            .script(encode_create_parent_vasp_account_script(
                account_config::xus_tag(),
                0,
                *payment_receiver.address(),
                payment_receiver.auth_key_prefix(),
                vec![],
                false,
            ))
            .sequence_number(1)
            .sign(),
    );

    // set the dual attestation info for the receiver
    executor.execute_and_apply(
        payment_receiver
            .transaction()
            .script(encode_rotate_dual_attestation_info_script(
                b"any base_url works".to_vec(),
                receiver_vasp_compliance_public_key.to_bytes().to_vec(),
            ))
            .sequence_number(0)
            .sign(),
    );

    executor.execute_and_apply(
        dd.transaction()
            .script(encode_peer_to_peer_with_metadata_script(
                account_config::xus_tag(),
                *payment_sender.address(),
                XUS_THRESHOLD * 10,
                vec![],
                vec![],
            ))
            .sequence_number(0)
            .sign(),
    );

    // create a child VASP with a balance of amount
    executor.execute_and_apply(
        payment_sender
            .transaction()
            .script(encode_create_child_vasp_account_script(
                account_config::xus_tag(),
                *sender_child.address(),
                sender_child.auth_key_prefix(),
                false,
                10,
            ))
            .sequence_number(0)
            .sign(),
    );

    // create a child VASP for the payee too
    executor.execute_and_apply(
        payment_receiver
            .transaction()
            .script(encode_create_child_vasp_account_script(
                account_config::xus_tag(),
                *payee_child.address(),
                payee_child.auth_key_prefix(),
                false,
                0,
            ))
            .sequence_number(1)
            .sign(),
    );
    {
        // Transaction >= 1_000_000 threshold goes through signature verification with valid signature, passes
        // Do the offline protocol: generate a payment id, sign with the receiver's private key, include
        // in transaction from sender's account
        let ref_id = bcs::to_bytes(&7777u64).unwrap();
        let output = executor.execute_and_apply(
            payment_sender
                .transaction()
                .script(create_dual_attestation_payment(
                    *payment_sender.address(),
                    *payment_receiver.address(),
                    payment_amount,
                    account_config::xus_tag(),
                    ref_id,
                    &receiver_vasp_compliance_private_key,
                ))
                .sequence_number(1)
                .sign(),
        );
        assert_eq!(output.status().status(), Ok(KeptVMStatus::Executed));
    }
    {
        // Transaction >= 1_000_000 threshold goes through signature verification with valid signature, passes
        // Do the offline protocol: generate a payment id, sign with the receiver's private key, include
        // in transaction from sender's account. Make sure credential resolution is working for
        // children.
        let ref_id = bcs::to_bytes(&7777u64).unwrap();
        let output = executor.execute_and_apply(
            payment_sender
                .transaction()
                .script(create_dual_attestation_payment(
                    *payment_sender.address(),
                    *payee_child.address(),
                    payment_amount,
                    account_config::xus_tag(),
                    ref_id,
                    &receiver_vasp_compliance_private_key,
                ))
                .sequence_number(2)
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
                    account_config::xus_tag(),
                    *payment_receiver.address(),
                    payment_amount,
                    ref_id,
                    b"invalid signature".to_vec(),
                ))
                .sequence_number(3)
                .sign(),
        );

        assert!(matches!(
            output.status().status(),
            Ok(KeptVMStatus::MoveAbort(_, BAD_METADATA_SIGNATURE_ERROR_CODE))
        ));
    }

    {
        // transaction >= 1_000_000 threshold goes through signature verification with invalid signature, aborts
        let ref_id = bcs::to_bytes(&9999u64).unwrap();
        let output = executor.execute_transaction(
            payment_sender
                .transaction()
                .script(create_dual_attestation_payment(
                    *payment_sender.address(),
                    *payment_receiver.address(),
                    payment_amount,
                    account_config::xus_tag(),
                    ref_id,
                    // Sign with the wrong private key
                    &sender_vasp_compliance_private_key,
                ))
                .sequence_number(3)
                .sign(),
        );

        assert_aborted_with(output, MISMATCHED_METADATA_SIGNATURE_ERROR_CODE);
    }

    {
        // similar, but with empty payment ID (make sure signature is still invalid!)
        let ref_id = vec![];
        let output = executor.execute_transaction(
            payment_sender
                .transaction()
                .script(create_dual_attestation_payment(
                    *payment_sender.address(),
                    *payment_receiver.address(),
                    payment_amount,
                    account_config::xus_tag(),
                    ref_id,
                    &sender_vasp_compliance_private_key,
                ))
                .sequence_number(3)
                .sign(),
        );
        assert_aborted_with(output, MISMATCHED_METADATA_SIGNATURE_ERROR_CODE);
    }
    {
        // Intra-VASP transaction >= 1000 threshold, should go through with any signature since
        // checking isn't performed on intra-vasp transfers
        // parent->child
        executor.execute_and_apply(
            payment_sender
                .transaction()
                .script(encode_peer_to_peer_with_metadata_script(
                    account_config::xus_tag(),
                    *sender_child.address(),
                    payment_amount * 2,
                    vec![0],
                    vec![],
                ))
                .sequence_number(3)
                .sign(),
        );

        // However, should still fail if we opt-in to dual attestation with a bad signature
        let output = executor.execute_transaction(
            payment_sender
                .transaction()
                .script(encode_peer_to_peer_with_metadata_script(
                    account_config::xus_tag(),
                    *sender_child.address(),
                    payment_amount * 2,
                    vec![0],
                    b"invalid signature".to_vec(),
                ))
                .sequence_number(4)
                .sign(),
        );
        assert_aborted_with(output, BAD_METADATA_SIGNATURE_ERROR_CODE)
    }
    {
        // Checking isn't performed on intra-vasp transfers
        // child->parent
        executor.execute_and_apply(
            sender_child
                .transaction()
                .script(encode_peer_to_peer_with_metadata_script(
                    account_config::xus_tag(),
                    *payment_sender.address(),
                    payment_amount,
                    vec![0],
                    vec![],
                ))
                .sequence_number(0)
                .sign(),
        );
    }
    {
        // Checking isn't performed on intra-vasp transfers (self payment)
        executor.execute_and_apply(
            sender_child
                .transaction()
                .script(encode_peer_to_peer_with_metadata_script(
                    account_config::xus_tag(),
                    *sender_child.address(),
                    payment_amount,
                    vec![0],
                    vec![],
                ))
                .sequence_number(1)
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
                .sequence_number(2)
                .sign(),
        );
    }
    {
        // This previously succeeded, but should now fail since their public key has changed
        // in transaction from sender's account. This tests to make sure their public key was
        // rotated.
        let output = executor.execute_transaction(
            payment_sender
                .transaction()
                .script(create_dual_attestation_payment(
                    *payment_sender.address(),
                    *payment_receiver.address(),
                    payment_amount,
                    account_config::xus_tag(),
                    // pick an arbitrary ref_id
                    bcs::to_bytes(&9999u64).unwrap(),
                    &receiver_vasp_compliance_private_key,
                ))
                .sequence_number(4)
                .sign(),
        );
        assert_aborted_with(output, MISMATCHED_METADATA_SIGNATURE_ERROR_CODE)
    }
    {
        // trying to send a tx to a recipient who has not set up dual attestation should fail
        let output = executor.execute_transaction(
            payment_receiver
                .transaction()
                .script(create_dual_attestation_payment(
                    *payment_receiver.address(),
                    *payment_sender.address(),
                    payment_amount,
                    account_config::xus_tag(),
                    // pick an arbitrary ref_id
                    bcs::to_bytes(&9999u64).unwrap(),
                    &receiver_vasp_compliance_private_key,
                ))
                .sequence_number(3)
                .sign(),
        );
        assert_aborted_with(output, PAYEE_COMPLIANCE_KEY_NOT_SET_ERROR_CODE)
    }
}

fn create_dual_attestation_payment(
    sender_address: AccountAddress,
    receiver_address: AccountAddress,
    amount: u64,
    coin_type: TypeTag,
    ref_id: Vec<u8>,
    receiver_compliance_private_key: &Ed25519PrivateKey,
) -> Script {
    let mut domain_separator = b"@@$$DIEM_ATTEST$$@@".to_vec();
    let message = {
        let mut msg = ref_id.clone();
        msg.append(&mut bcs::to_bytes(&sender_address).unwrap());
        msg.append(&mut bcs::to_bytes(&amount).unwrap());
        msg.append(&mut domain_separator);
        msg
    };
    let signature = <Ed25519PrivateKey as SigningKey>::sign_arbitrary_message(
        &receiver_compliance_private_key,
        &message,
    );
    encode_peer_to_peer_with_metadata_script(
        coin_type,
        receiver_address,
        amount,
        ref_id,
        signature.to_bytes().to_vec(),
    )
}

fn assert_aborted_with(output: TransactionOutput, error_code: u64) {
    if let Ok(KeptVMStatus::MoveAbort(_, code)) = output.status().status() {
        assert_eq!(error_code, code);
    } else {
        assert!(matches!(
            output.status().status(),
            Ok(KeptVMStatus::MoveAbort(..))
        ));
    }
}

// Check that DD <-> DD and DD <-> VASP payments over the threshold work with and without dual attesation.
#[test]
fn dd_dual_attestation_payments() {
    let mut executor = FakeExecutor::from_genesis_file();
    executor.set_golden_file(current_function_name!());

    // account that will receive the dual attestation payment
    let parent_vasp = executor.create_raw_account();
    let dd1 = executor.create_raw_account();
    let dd2 = executor.create_raw_account();
    let blessed = Account::new_blessed_tc();
    let mint_dd = Account::new_genesis_account(account_config::testnet_dd_account_address());
    let mut keygen = KeyGen::from_seed([9u8; 32]);
    let (parent_vasp_compliance_private_key, parent_vasp_compliance_public_key) =
        keygen.generate_keypair();
    let (dd1_compliance_private_key, dd1_compliance_public_key) = keygen.generate_keypair();
    let (dd2_compliance_private_key, dd2_compliance_public_key) = keygen.generate_keypair();

    // create the VASP account
    executor.execute_and_apply(
        blessed
            .transaction()
            .script(encode_create_parent_vasp_account_script(
                account_config::xus_tag(),
                0,
                *parent_vasp.address(),
                parent_vasp.auth_key_prefix(),
                vec![],
                false,
            ))
            .sequence_number(0)
            .sign(),
    );
    // create the DD1 account
    executor.execute_and_apply(
        blessed
            .transaction()
            .script(encode_create_designated_dealer_script(
                account_config::xus_tag(),
                0,
                *dd1.address(),
                dd1.auth_key_prefix(),
                vec![],
                false,
            ))
            .sequence_number(1)
            .sign(),
    );
    // create the DD2 account
    executor.execute_and_apply(
        blessed
            .transaction()
            .script(encode_create_designated_dealer_script(
                account_config::xus_tag(),
                0,
                *dd2.address(),
                dd2.auth_key_prefix(),
                vec![],
                false,
            ))
            .sequence_number(2)
            .sign(),
    );

    // set up dual attestation info for VASP, DD1, DD2
    executor.execute_and_apply(
        parent_vasp
            .transaction()
            .script(encode_rotate_dual_attestation_info_script(
                b"any base_url works".to_vec(),
                parent_vasp_compliance_public_key.to_bytes().to_vec(),
            ))
            .sequence_number(0)
            .sign(),
    );
    executor.execute_and_apply(
        dd1.transaction()
            .script(encode_rotate_dual_attestation_info_script(
                b"any base_url works".to_vec(),
                dd1_compliance_public_key.to_bytes().to_vec(),
            ))
            .sequence_number(0)
            .sign(),
    );
    executor.execute_and_apply(
        dd2.transaction()
            .script(encode_rotate_dual_attestation_info_script(
                b"any base_url works".to_vec(),
                dd2_compliance_public_key.to_bytes().to_vec(),
            ))
            .sequence_number(0)
            .sign(),
    );

    // give DD1 some funds
    executor.execute_and_apply(
        mint_dd
            .transaction()
            .script(encode_peer_to_peer_with_metadata_script(
                account_config::xus_tag(),
                *dd1.address(),
                XUS_THRESHOLD * 4,
                vec![],
                vec![],
            ))
            .sequence_number(0)
            .sign(),
    );
    // Give VASP some funds
    executor.execute_and_apply(
        mint_dd
            .transaction()
            .script(encode_peer_to_peer_with_metadata_script(
                account_config::xus_tag(),
                *parent_vasp.address(),
                XUS_THRESHOLD * 2,
                vec![],
                vec![],
            ))
            .sequence_number(1)
            .sign(),
    );

    // DD <-> DD over threshold without attestation succeeds
    executor.execute_and_apply(
        dd1.transaction()
            .script(encode_peer_to_peer_with_metadata_script(
                account_config::xus_tag(),
                *dd2.address(),
                XUS_THRESHOLD,
                vec![0],
                vec![],
            ))
            .sequence_number(1)
            .sign(),
    );
    // DD <-> DD over threshold with attestation succeeds
    executor.execute_and_apply(
        dd1.transaction()
            .script(create_dual_attestation_payment(
                *dd1.address(),
                *dd2.address(),
                XUS_THRESHOLD,
                account_config::xus_tag(),
                // pick an arbitrary ref_id
                bcs::to_bytes(&9999u64).unwrap(),
                &dd2_compliance_private_key,
            ))
            .sequence_number(2)
            .sign(),
    );

    // DD -> VASP over threshold without attestation succeeds
    executor.execute_and_apply(
        dd1.transaction()
            .script(encode_peer_to_peer_with_metadata_script(
                account_config::xus_tag(),
                *parent_vasp.address(),
                XUS_THRESHOLD,
                vec![0],
                vec![],
            ))
            .sequence_number(3)
            .sign(),
    );
    // DD -> VASP over threshold with attestation succeeds
    executor.execute_and_apply(
        dd1.transaction()
            .script(create_dual_attestation_payment(
                *dd1.address(),
                *parent_vasp.address(),
                XUS_THRESHOLD,
                account_config::xus_tag(),
                // pick an arbitrary ref_id
                bcs::to_bytes(&9999u64).unwrap(),
                &parent_vasp_compliance_private_key,
            ))
            .sequence_number(4)
            .sign(),
    );

    // VASP -> DD over threshold without attestation succeeds
    executor.execute_and_apply(
        parent_vasp
            .transaction()
            .script(encode_peer_to_peer_with_metadata_script(
                account_config::xus_tag(),
                *dd1.address(),
                XUS_THRESHOLD,
                vec![0],
                vec![],
            ))
            .sequence_number(1)
            .sign(),
    );
    // VASP -> DD over threshold with attestation succeeds
    executor.execute_and_apply(
        parent_vasp
            .transaction()
            .script(create_dual_attestation_payment(
                *parent_vasp.address(),
                *dd1.address(),
                XUS_THRESHOLD,
                account_config::xus_tag(),
                // pick an arbitrary ref_id
                bcs::to_bytes(&9999u64).unwrap(),
                &dd1_compliance_private_key,
            ))
            .sequence_number(2)
            .sign(),
    );

    // VASP -> DD over threshold with opt-in, but bad attestation fails
    let output = executor.execute_transaction(
        parent_vasp
            .transaction()
            .script(encode_peer_to_peer_with_metadata_script(
                account_config::xus_tag(),
                *dd1.address(),
                XUS_THRESHOLD,
                vec![0],
                b"what a bad signature".to_vec(),
            ))
            .sequence_number(3)
            .sign(),
    );
    assert_aborted_with(output, BAD_METADATA_SIGNATURE_ERROR_CODE)
}

#[test]
fn publish_rotate_shared_ed25519_public_key() {
    let mut executor = FakeExecutor::from_genesis_file();
    executor.set_golden_file(current_function_name!());

    let mut publisher = {
        let data = executor.create_raw_account_data(1_000_000, 0);
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
    executor.set_golden_file(current_function_name!());

    let blessed = Account::new_blessed_tc();

    let parent = executor.create_raw_account();
    let mut child = executor.create_raw_account();
    let other_vasp = executor.create_raw_account();

    let mut keygen = KeyGen::from_seed([9u8; 32]);
    let (_vasp_compliance_private_key, _) = keygen.generate_keypair();

    // create a parent VASP
    let add_all_currencies = false;
    executor.execute_and_apply(
        blessed
            .transaction()
            .script(encode_create_parent_vasp_account_script(
                account_config::xus_tag(),
                0,
                *parent.address(),
                parent.auth_key_prefix(),
                vec![],
                add_all_currencies,
            ))
            .sequence_number(0)
            .sign(),
    );

    // create a child VASP with a zero balance
    executor.execute_and_apply(
        parent
            .transaction()
            .script(encode_create_child_vasp_account_script(
                account_config::xus_tag(),
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
        blessed
            .transaction()
            .script(encode_create_parent_vasp_account_script(
                account_config::xus_tag(),
                0,
                *other_vasp.address(),
                other_vasp.auth_key_prefix(),
                vec![],
                add_all_currencies,
            ))
            .sequence_number(1)
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
    assert_aborted_with(output, 775);

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
    assert_aborted_with(output, 519);
}

#[test]
fn add_child_currencies() {
    let mut executor = FakeExecutor::from_genesis_file();
    executor.set_golden_file(current_function_name!());

    let vasp_a = executor.create_raw_account();
    let vasp_a_child1 = executor.create_raw_account();
    let vasp_b = executor.create_raw_account();
    let vasp_b_child1 = executor.create_raw_account();
    let vasp_b_child2 = executor.create_raw_account();
    let blessed = Account::new_blessed_tc();

    currencies::add_currency_to_system(&mut executor, "COIN", 1);

    executor.execute_and_apply(
        blessed
            .transaction()
            .script(encode_create_parent_vasp_account_script(
                account_config::xus_tag(),
                0,
                *vasp_a.address(),
                vasp_a.auth_key_prefix(),
                vec![],
                false,
            ))
            .sequence_number(0)
            .sign(),
    );

    // Adding a child with the same currency is no issue
    executor.execute_and_apply(
        vasp_a
            .transaction()
            .script(encode_create_child_vasp_account_script(
                account_config::xus_tag(),
                *vasp_a_child1.address(),
                vasp_a_child1.auth_key_prefix(),
                false,
                0,
            ))
            .sequence_number(0)
            .sign(),
    );

    executor.execute_and_apply(
        vasp_a
            .transaction()
            .script(encode_add_currency_to_account_script(
                account_config::type_tag_for_currency_code(account::currency_code("COIN")),
            ))
            .sequence_number(1)
            .sign(),
    );

    ///////////////////////////////////////////////////////////////////////////
    // Now make a parent with all currencies, and make sure the children are fine
    ///////////////////////////////////////////////////////////////////////////

    executor.execute_and_apply(
        blessed
            .transaction()
            .script(encode_create_parent_vasp_account_script(
                account_config::xus_tag(),
                0,
                *vasp_b.address(),
                vasp_b.auth_key_prefix(),
                vec![],
                true,
            ))
            .sequence_number(1)
            .sign(),
    );

    // Adding a child with the same currency and all other currencies isn't an issue
    executor.execute_and_apply(
        vasp_b
            .transaction()
            .script(encode_create_child_vasp_account_script(
                account_config::xus_tag(),
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
                account_config::type_tag_for_currency_code(account::currency_code("COIN")),
                *vasp_b_child2.address(),
                vasp_b_child2.auth_key_prefix(),
                false,
                0,
            ))
            .sequence_number(1)
            .sign(),
    );
}
