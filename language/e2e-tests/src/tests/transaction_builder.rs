// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Tests for all of the script encoding functions in language/transaction_builder/lib.rs.
//! Thorough tests that exercise all of the behaviors of the script should live in the language
//! functional tests; these tests are only to ensure that the script encoding functions take the
//! correct types + produce a runnable script.

#![forbid(unsafe_code)]

use crate::{
    account::{self, Account, AccountData},
    common_transactions::{create_account_txn, rotate_key_txn},
    executor::FakeExecutor,
    keygen::KeyGen,
};
use libra_crypto::{ed25519::Ed25519PrivateKey, traits::SigningKey, PrivateKey, Uniform};
use libra_types::{
    account_config,
    transaction::{authenticator::AuthenticationKey, TransactionStatus},
    vm_status::{StatusCode, VMStatus},
};
use move_core_types::{
    identifier::Identifier,
    language_storage::{StructTag, TypeTag},
};
use transaction_builder::*;

#[test]
fn freeze_unfreeze_account() {
    // create a FakeExecutor with a genesis from file
    let mut executor = FakeExecutor::from_genesis_file();

    let account = {
        let data = AccountData::new(1_000_000, 0);
        executor.add_account_data(&data);
        data.into_account()
    };

    let _lbr_ty = TypeTag::Struct(StructTag {
        address: account_config::CORE_CODE_ADDRESS,
        module: Identifier::new("LibraAccount").unwrap(),
        name: Identifier::new("FreezingPrivilege").unwrap(),
        type_params: vec![],
    });
    let blessed = Account::new_blessed_tc();
    // Execute freeze on account
    executor.execute_and_apply(
        blessed.signed_script_txn(encode_freeze_account(1, *account.address()), 0),
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
        blessed.signed_script_txn(encode_unfreeze_account(2, *account.address()), 1),
    );
    // execute rotate key transaction from unfrozen account now succeeds
    let output = &executor.execute_transaction(txn);
    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED)),
    );
}

#[test]
fn create_parent_and_child_vasp() {
    let mut executor = FakeExecutor::from_genesis_file();
    let association = Account::new_association();
    let parent = Account::new();
    let child = Account::new();

    let mut keygen = KeyGen::from_seed([9u8; 32]);
    let (_vasp_compliance_private_key, vasp_compliance_public_key) = keygen.generate_keypair();

    // create a parent VASP
    let add_all_currencies = false;
    executor.execute_and_apply(association.signed_script_txn(
        encode_create_parent_vasp_account(
            account_config::lbr_type_tag(),
            *parent.address(),
            parent.auth_key_prefix(),
            vec![],
            vec![],
            vasp_compliance_public_key.to_bytes().to_vec(),
            add_all_currencies,
        ),
        1,
    ));

    // create a child VASP with a zero balance
    executor.execute_and_apply(parent.signed_script_txn(
        encode_create_child_vasp_account(
            account_config::lbr_type_tag(),
            *child.address(),
            child.auth_key_prefix(),
            add_all_currencies,
            0,
        ),
        0,
    ));
    // check for zero balance
    assert_eq!(
        executor
            .read_balance_resource(&child, account::lbr_currency_code())
            .unwrap()
            .coin(),
        0
    );

    let (_, new_compliance_public_key) = keygen.generate_keypair();
    // rotate parent's compliance public key
    executor.execute_and_apply(parent.signed_script_txn(
        encode_rotate_base_url_script(new_compliance_public_key.to_bytes().to_vec()),
        1,
    ));

    // rotate parent's base URL
    executor.execute_and_apply(
        parent.signed_script_txn(encode_rotate_base_url_script(b"new_name".to_vec()), 2),
    );
}

#[test]
fn create_child_vasp_all_currencies() {
    let mut executor = FakeExecutor::from_genesis_file();
    let association = Account::new_association();
    let blessed = Account::new_blessed_tc();
    let parent = Account::new();
    let child = Account::new();

    let mut keygen = KeyGen::from_seed([9u8; 32]);
    let (_vasp_compliance_private_key, vasp_compliance_public_key) = keygen.generate_keypair();

    // create a parent VASP
    let add_all_currencies = true;
    executor.execute_and_apply(association.signed_script_txn(
        encode_create_parent_vasp_account(
            account_config::coin1_tag(),
            *parent.address(),
            parent.auth_key_prefix(),
            vec![],
            vec![],
            vasp_compliance_public_key.to_bytes().to_vec(),
            add_all_currencies,
        ),
        1,
    ));

    let amount = 100;
    // mint to the parent VASP
    executor.execute_and_apply(blessed.signed_script_txn(
        encode_mint_script(
            account_config::coin1_tag(),
            parent.address(),
            vec![],
            amount,
        ),
        0,
    ));

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
    executor.execute_and_apply(parent.signed_script_txn(
        encode_create_child_vasp_account(
            account_config::coin1_tag(),
            *child.address(),
            child.auth_key_prefix(),
            add_all_currencies,
            amount,
        ),
        0,
    ));

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
    let association = Account::new_association();
    let blessed = Account::new_blessed_tc();
    let parent = Account::new();
    let child = Account::new();

    let mut keygen = KeyGen::from_seed([9u8; 32]);
    let (_vasp_compliance_private_key, vasp_compliance_public_key) = keygen.generate_keypair();

    // create a parent VASP
    let add_all_currencies = true;
    executor.execute_and_apply(association.signed_script_txn(
        encode_create_parent_vasp_account(
            account_config::coin1_tag(),
            *parent.address(),
            parent.auth_key_prefix(),
            vec![],
            vec![],
            vasp_compliance_public_key.to_bytes().to_vec(),
            add_all_currencies,
        ),
        1,
    ));

    let amount = 100;
    // mint to the parent VASP
    executor.execute_and_apply(blessed.signed_script_txn(
        encode_mint_script(
            account_config::coin1_tag(),
            parent.address(),
            vec![],
            amount,
        ),
        0,
    ));

    assert_eq!(
        executor
            .read_balance_resource(&parent, account::coin1_currency_code())
            .unwrap()
            .coin(),
        amount
    );

    // create a child VASP with a balance of amount
    executor.execute_and_apply(parent.signed_script_txn(
        encode_create_child_vasp_account(
            account_config::coin1_tag(),
            *child.address(),
            child.auth_key_prefix(),
            add_all_currencies,
            amount,
        ),
        0,
    ));

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
    let association = Account::new_association();
    let blessed = Account::new_blessed_tc();

    let mut keygen = KeyGen::from_seed([9u8; 32]);
    let (sender_vasp_compliance_private_key, sender_vasp_compliance_public_key) =
        keygen.generate_keypair();
    let (receiver_vasp_compliance_private_key, receiver_vasp_compliance_public_key) =
        keygen.generate_keypair();

    // choose an amount large enough to make multiple payments
    let mint_amount = 100_000_000_000;
    // choose an amount above the dual attestation threshold
    let payment_amount = 1_000_000_000u64;

    executor.execute_and_apply(association.signed_script_txn(
        encode_create_parent_vasp_account(
            account_config::lbr_type_tag(),
            *payment_sender.address(),
            payment_sender.auth_key_prefix(),
            vec![],
            vec![],
            sender_vasp_compliance_public_key.to_bytes().to_vec(),
            false,
        ),
        1,
    ));

    executor.execute_and_apply(association.signed_script_txn(
        encode_create_parent_vasp_account(
            account_config::lbr_type_tag(),
            *payment_receiver.address(),
            payment_receiver.auth_key_prefix(),
            vec![],
            vec![],
            receiver_vasp_compliance_public_key.to_bytes().to_vec(),
            false,
        ),
        2,
    ));

    executor.execute_and_apply(blessed.signed_script_txn(
        encode_mint_script(
            account_config::lbr_type_tag(),
            &payment_sender.address(),
            vec![],
            mint_amount,
        ),
        0,
    ));

    // create a child VASP with a balance of amount
    executor.execute_and_apply(payment_sender.signed_script_txn(
        encode_create_child_vasp_account(
            account_config::lbr_type_tag(),
            *sender_child.address(),
            sender_child.auth_key_prefix(),
            false,
            10,
        ),
        0,
    ));
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
        let output = executor.execute_and_apply(payment_sender.signed_script_txn(
            encode_transfer_with_metadata_script(
                account_config::lbr_type_tag(),
                *payment_receiver.address(),
                payment_amount,
                ref_id,
                signature.to_bytes().to_vec(),
            ),
            1,
        ));
        assert_eq!(
            output.status().vm_status().major_status,
            StatusCode::EXECUTED
        );
    }
    {
        // transaction >= 1_000_000 (set in DualAttestation.move) threshold goes through signature verification but has an
        // structurally invalid signature. Fails.
        let ref_id = [0u8; 32].to_vec();
        let output = executor.execute_transaction(payment_sender.signed_script_txn(
            encode_transfer_with_metadata_script(
                account_config::lbr_type_tag(),
                *payment_receiver.address(),
                payment_amount,
                ref_id,
                b"invalid signature".to_vec(),
            ),
            2,
        ));
        assert_eq!(
            output.status().vm_status().major_status,
            StatusCode::ABORTED
        );
        assert_eq!(output.status().vm_status().sub_status, Some(9001));
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
        let output = executor.execute_transaction(payment_sender.signed_script_txn(
            encode_transfer_with_metadata_script(
                account_config::lbr_type_tag(),
                *payment_receiver.address(),
                payment_amount,
                ref_id,
                signature.to_bytes().to_vec(),
            ),
            2,
        ));
        assert_eq!(
            output.status().vm_status().major_status,
            StatusCode::ABORTED
        );
        assert_eq!(output.status().vm_status().sub_status, Some(9002));
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
        let output = executor.execute_transaction(payment_sender.signed_script_txn(
            encode_transfer_with_metadata_script(
                account_config::lbr_type_tag(),
                *payment_receiver.address(),
                payment_amount,
                ref_id,
                signature.to_bytes().to_vec(),
            ),
            2,
        ));
        assert_eq!(
            output.status().vm_status().major_status,
            StatusCode::ABORTED
        );
        assert_eq!(output.status().vm_status().sub_status, Some(9002));
    }
    {
        // Intra-VASP transaction >= 1000 threshold, should go through with any signature since
        // checking isn't performed on intra-vasp transfers
        // parent->child
        executor.execute_and_apply(payment_sender.signed_script_txn(
            encode_transfer_with_metadata_script(
                account_config::lbr_type_tag(),
                *sender_child.address(),
                payment_amount * 2,
                vec![0],
                b"what a bad signature".to_vec(),
            ),
            2,
        ));
    }
    {
        // Checking isn't performed on intra-vasp transfers
        // child->parent
        executor.execute_and_apply(sender_child.signed_script_txn(
            encode_transfer_with_metadata_script(
                account_config::lbr_type_tag(),
                *payment_sender.address(),
                payment_amount,
                vec![0],
                b"what a bad signature".to_vec(),
            ),
            0,
        ));
    }

    // ======= tests for UHW =======

    // create two unhosted accounts + give some funds to the first one
    let unhosted = Account::new();
    let unhosted_other = Account::new();
    executor.execute_and_apply(create_account_txn(&association, &unhosted, 3, 0));
    executor.execute_and_apply(create_account_txn(&association, &unhosted_other, 4, 0));
    executor.execute_and_apply(blessed.signed_script_txn(
        encode_mint_script(
            account_config::lbr_type_tag(),
            &unhosted.address(),
            vec![],
            mint_amount,
        ),
        1,
    ));

    {
        // Check that unhosted wallet <-> VASP transactions do not require dual attestation
        // since checking isn't performed on VASP->UHW transfers.
        executor.execute_and_apply(payment_sender.signed_script_txn(
            encode_transfer_with_metadata_script(
                account_config::lbr_type_tag(),
                *unhosted.address(),
                payment_amount,
                vec![0],
                b"what a bad signature".to_vec(),
            ),
            3,
        ));
    }
    {
        // Checking isn't performed on VASP->UHW
        // Check from a child account.
        executor.execute_and_apply(sender_child.signed_script_txn(
            encode_transfer_with_metadata_script(
                account_config::lbr_type_tag(),
                *unhosted.address(),
                payment_amount,
                vec![0],
                b"what a bad signature".to_vec(),
            ),
            1,
        ));
    }
    {
        // Checking isn't performed on UHW->VASP
        executor.execute_and_apply(unhosted.signed_script_txn(
            encode_transfer_with_metadata_script(
                account_config::lbr_type_tag(),
                *payment_sender.address(),
                payment_amount,
                vec![0],
                b"what a bad signature".to_vec(),
            ),
            0,
        ));
    }
    {
        // Checking isn't performed on UHW->VASP
        executor.execute_and_apply(unhosted.signed_script_txn(
            encode_transfer_with_metadata_script(
                account_config::lbr_type_tag(),
                *sender_child.address(),
                payment_amount,
                vec![0],
                b"what a bad signature".to_vec(),
            ),
            1,
        ));
    }
    {
        // Finally, check that unhosted <-> unhosted transactions do not require dual attestation
        // Checking isn't performed on UHW->UHW
        executor.execute_and_apply(unhosted.signed_script_txn(
            encode_transfer_with_metadata_script(
                account_config::lbr_type_tag(),
                *unhosted_other.address(),
                payment_amount,
                vec![0],
                b"what a bad signature".to_vec(),
            ),
            2,
        ));
    }
    {
        // Rotate the parent VASP's compliance key
        let (_, new_compliance_public_key) = keygen.generate_keypair();
        executor.execute_and_apply(payment_receiver.signed_script_txn(
            encode_rotate_compliance_public_key_script(
                new_compliance_public_key.to_bytes().to_vec(),
            ),
            0,
        ));
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
        let output = executor.execute_transaction(payment_sender.signed_script_txn(
            encode_transfer_with_metadata_script(
                account_config::lbr_type_tag(),
                *payment_receiver.address(),
                payment_amount,
                ref_id,
                signature.to_bytes().to_vec(),
            ),
            4,
        ));
        assert_eq!(
            output.status().vm_status().major_status,
            StatusCode::ABORTED
        );
        assert_eq!(output.status().vm_status().sub_status, Some(9002));
    }
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

#[test]
fn recovery_address() {
    let mut executor = FakeExecutor::from_genesis_file();
    let association = Account::new_association();

    let parent = Account::new();
    let mut child = Account::new();
    let other_vasp = Account::new();

    let mut keygen = KeyGen::from_seed([9u8; 32]);
    let (_vasp_compliance_private_key, vasp_compliance_public_key) = keygen.generate_keypair();

    // create a parent VASP
    let add_all_currencies = false;
    executor.execute_and_apply(association.signed_script_txn(
        encode_create_parent_vasp_account(
            account_config::lbr_type_tag(),
            *parent.address(),
            parent.auth_key_prefix(),
            vec![],
            vec![],
            vasp_compliance_public_key.to_bytes().to_vec(),
            add_all_currencies,
        ),
        1,
    ));

    // create a child VASP with a zero balance
    executor.execute_and_apply(parent.signed_script_txn(
        encode_create_child_vasp_account(
            account_config::lbr_type_tag(),
            *child.address(),
            child.auth_key_prefix(),
            add_all_currencies,
            0,
        ),
        0,
    ));

    // publish a recovery address under the parent
    executor.execute_and_apply(parent.signed_script_txn(encode_create_recovery_address(), 1));

    // delegate authentication key of the child
    executor.execute_and_apply(child.signed_script_txn(
        encode_add_recovery_rotation_capability(*parent.address()),
        0,
    ));

    // rotate authentication key from the parent
    let (privkey1, pubkey1) = keygen.generate_keypair();
    let new_authentication_key1 = AuthenticationKey::ed25519(&pubkey1).to_vec();
    executor.execute_and_apply(parent.signed_script_txn(
        encode_rotate_authentication_key_with_recovery_address_script(
            *parent.address(),
            *child.address(),
            new_authentication_key1,
        ),
        2,
    ));

    // rotate authentication key from the child
    let (_, pubkey2) = keygen.generate_keypair();
    let new_authentication_key2 = AuthenticationKey::ed25519(&pubkey2).to_vec();
    child.rotate_key(privkey1, pubkey1);
    executor.execute_and_apply(child.signed_script_txn(
        encode_rotate_authentication_key_with_recovery_address_script(
            *parent.address(),
            *child.address(),
            new_authentication_key2,
        ),
        1,
    ));

    // create another VASP unrelated to parent/child
    let add_all_currencies = false;
    executor.execute_and_apply(association.signed_script_txn(
        encode_create_parent_vasp_account(
            account_config::lbr_type_tag(),
            *other_vasp.address(),
            other_vasp.auth_key_prefix(),
            vec![],
            vec![],
            vasp_compliance_public_key.to_bytes().to_vec(),
            add_all_currencies,
        ),
        2,
    ));

    // try to delegate other_vasp's rotation cap to child--should abort
    let output = executor.execute_transaction(other_vasp.signed_script_txn(
        encode_add_recovery_rotation_capability(*parent.address()),
        0,
    ));
    assert_eq!(
        output.status().vm_status().major_status,
        StatusCode::ABORTED
    );
    assert_eq!(output.status().vm_status().sub_status, Some(444));

    // try to rotate child's key from other_vasp--should abort
    let (_, pubkey3) = keygen.generate_keypair();
    let new_authentication_key3 = AuthenticationKey::ed25519(&pubkey3).to_vec();
    let output = executor.execute_transaction(other_vasp.signed_script_txn(
        encode_rotate_authentication_key_with_recovery_address_script(
            *parent.address(),
            *child.address(),
            new_authentication_key3,
        ),
        0,
    ));
    assert_eq!(
        output.status().vm_status().major_status,
        StatusCode::ABORTED
    );
    assert_eq!(output.status().vm_status().sub_status, Some(3333));
}
