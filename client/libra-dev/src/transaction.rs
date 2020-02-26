// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    data::{
        LibraP2PTransferTransactionArgument, LibraRawTransaction, LibraSignedTransaction,
        LibraStatus, LibraTransactionPayload, TransactionType,
    },
    error::*,
};
use lcs::to_bytes;
use libra_crypto::{ed25519::*, test_utils::KeyPair};
use libra_types::{
    account_address::{AccountAddress, ADDRESS_LENGTH},
    transaction::{
        helpers::TransactionSigner, RawTransaction, SignedTransaction, TransactionArgument,
        TransactionPayload,
    },
};
use std::{convert::TryFrom, slice, time::Duration};
use transaction_builder::{encode_transfer_script, get_transaction_name};

#[no_mangle]
pub unsafe extern "C" fn libra_SignedTransactionBytes_from(
    sender_private_key_bytes: *const u8,
    receiver: *const u8,
    sequence: u64,
    num_coins: u64,
    max_gas_amount: u64,
    gas_unit_price: u64,
    expiration_time_secs: u64,
    ptr_buf: *mut *mut u8,
    ptr_len: *mut usize,
) -> LibraStatus {
    clear_error();

    if sender_private_key_bytes.is_null() {
        update_last_error("sender_private_key_bytes parameter must not be null.".to_string());
        return LibraStatus::InvalidArgument;
    }
    let private_key_buf: &[u8] =
        slice::from_raw_parts(sender_private_key_bytes, ED25519_PRIVATE_KEY_LENGTH);
    let private_key = match Ed25519PrivateKey::try_from(private_key_buf) {
        Ok(result) => result,
        Err(e) => {
            update_last_error(format!("Invalid private key bytes: {}", e.to_string()));
            return LibraStatus::InvalidArgument;
        }
    };

    let public_key: Ed25519PublicKey = (&private_key).into();
    let sender_address = AccountAddress::from_public_key(&public_key);

    if receiver.is_null() {
        update_last_error("receiver parameter must not be null.".to_string());
        return LibraStatus::InvalidArgument;
    }
    let receiver_buf = slice::from_raw_parts(receiver, ADDRESS_LENGTH);
    let receiver_address = match AccountAddress::try_from(receiver_buf) {
        Ok(result) => result,
        Err(e) => {
            update_last_error(format!("Invalid receiver address: {}", e.to_string()));
            return LibraStatus::InvalidArgument;
        }
    };

    let expiration_time = Duration::from_secs(expiration_time_secs);

    let program = encode_transfer_script(&receiver_address, num_coins);
    let payload = TransactionPayload::Script(program);
    let raw_txn = RawTransaction::new(
        sender_address,
        sequence,
        payload,
        max_gas_amount,
        gas_unit_price,
        expiration_time,
    );

    let keypair = KeyPair::from(private_key);
    let signer: Box<&dyn TransactionSigner> = Box::new(&keypair);
    let signed_txn = match signer.sign_txn(raw_txn) {
        Ok(result) => result,
        Err(e) => {
            update_last_error(format!("Error signing transaction: {}", e.to_string()));
            return LibraStatus::InvalidArgument;
        }
    };

    let signed_txn_bytes = match to_bytes(&signed_txn) {
        Ok(result) => result,
        Err(e) => {
            update_last_error(format!(
                "Error serializing signed transaction: {}",
                e.to_string()
            ));
            return LibraStatus::InternalError;
        }
    };
    let txn_buf: *mut u8 = libc::malloc(signed_txn_bytes.len()).cast();
    txn_buf.copy_from(signed_txn_bytes.as_ptr(), signed_txn_bytes.len());

    *ptr_buf = txn_buf;
    *ptr_len = signed_txn_bytes.len();

    LibraStatus::Ok
}

#[no_mangle]
pub unsafe extern "C" fn libra_free_bytes_buffer(buf: *const u8) {
    assert!(!buf.is_null());
    libc::free(buf as *mut libc::c_void);
}

#[no_mangle]
pub unsafe extern "C" fn libra_RawTransactionBytes_from(
    sender: *const u8,
    receiver: *const u8,
    sequence: u64,
    num_coins: u64,
    max_gas_amount: u64,
    gas_unit_price: u64,
    expiration_time_secs: u64,
    buf: *mut *mut u8,
    len: *mut usize,
) -> LibraStatus {
    clear_error();

    if sender.is_null() {
        update_last_error("sender parameter must not be null.".to_string());
        return LibraStatus::InvalidArgument;
    }
    let sender_buf = slice::from_raw_parts(sender, ADDRESS_LENGTH);
    let sender_address = match AccountAddress::try_from(sender_buf) {
        Ok(result) => result,
        Err(e) => {
            update_last_error(format!("Invalid sender address: {}", e.to_string()));
            return LibraStatus::InvalidArgument;
        }
    };

    if receiver.is_null() {
        update_last_error("receiver parameter must not be null.".to_string());
        return LibraStatus::InvalidArgument;
    }
    let receiver_buf = slice::from_raw_parts(receiver, ADDRESS_LENGTH);
    let receiver_address = match AccountAddress::try_from(receiver_buf) {
        Ok(result) => result,
        Err(e) => {
            update_last_error(format!("Invalid receiver address: {}", e.to_string()));
            return LibraStatus::InvalidArgument;
        }
    };
    let expiration_time = Duration::from_secs(expiration_time_secs);

    let program = encode_transfer_script(&receiver_address, num_coins);
    let payload = TransactionPayload::Script(program);
    let raw_txn = RawTransaction::new(
        sender_address,
        sequence,
        payload,
        max_gas_amount,
        gas_unit_price,
        expiration_time,
    );

    let raw_txn_bytes = match to_bytes(&raw_txn) {
        Ok(result) => result,
        Err(e) => {
            update_last_error(format!(
                "Error serializing raw transaction: {}",
                e.to_string()
            ));
            return LibraStatus::InternalError;
        }
    };

    let txn_buf: *mut u8 = libc::malloc(raw_txn_bytes.len()).cast();
    txn_buf.copy_from(raw_txn_bytes.as_ptr(), raw_txn_bytes.len());

    *buf = txn_buf;
    *len = raw_txn_bytes.len();

    LibraStatus::Ok
}

#[no_mangle]
pub unsafe extern "C" fn libra_RawTransaction_sign(
    buf_raw_txn: *const u8,
    len_raw_txn: usize,
    buf_public_key: *const u8,
    len_public_key: usize,
    buf_signature: *const u8,
    len_signature: usize,
    buf_result: *mut *mut u8,
    len_result: *mut usize,
) -> LibraStatus {
    clear_error();
    if buf_raw_txn.is_null() {
        update_last_error("buf_raw_txn parameter must not be null.".to_string());
        return LibraStatus::InvalidArgument;
    }
    let raw_txn_bytes: &[u8] = slice::from_raw_parts(buf_raw_txn, len_raw_txn);
    let raw_txn: RawTransaction = match lcs::from_bytes(&raw_txn_bytes) {
        Ok(result) => result,
        Err(e) => {
            update_last_error(format!(
                "Error deserializing raw transaction, invalid raw_txn bytes or length: {}",
                e.to_string()
            ));
            return LibraStatus::InvalidArgument;
        }
    };

    if buf_public_key.is_null() {
        update_last_error("buf_public_key parameter must not be null.".to_string());
        return LibraStatus::InvalidArgument;
    }
    let public_key_bytes: &[u8] = slice::from_raw_parts(buf_public_key, len_public_key);
    let public_key = match Ed25519PublicKey::try_from(public_key_bytes) {
        Ok(result) => result,
        Err(e) => {
            update_last_error(format!("Invalid public key bytes: {}", e.to_string()));
            return LibraStatus::InvalidArgument;
        }
    };

    if buf_signature.is_null() {
        update_last_error("buf_signature parameter must not be null.".to_string());
        return LibraStatus::InvalidArgument;
    }
    let signature_bytes: &[u8] = slice::from_raw_parts(buf_signature, len_signature);
    let signature = match Ed25519Signature::try_from(signature_bytes) {
        Ok(result) => result,
        Err(e) => {
            update_last_error(format!("Invalid signature bytes: {}", e.to_string()));
            return LibraStatus::InvalidArgument;
        }
    };

    let signed_txn = SignedTransaction::new(raw_txn, public_key, signature);
    let signed_txn_bytes = match to_bytes(&signed_txn) {
        Ok(result) => result,
        Err(e) => {
            update_last_error(format!(
                "Error serializing signed transaction: {}",
                e.to_string()
            ));
            return LibraStatus::InternalError;
        }
    };

    let txn_buf: *mut u8 = libc::malloc(signed_txn_bytes.len()).cast();
    txn_buf.copy_from(signed_txn_bytes.as_ptr(), signed_txn_bytes.len());

    *buf_result = txn_buf;
    *len_result = signed_txn_bytes.len();

    LibraStatus::Ok
}

#[no_mangle]
pub unsafe extern "C" fn libra_LibraSignedTransaction_from(
    buf: *const u8,
    len: usize,
    out: *mut LibraSignedTransaction,
) -> LibraStatus {
    clear_error();
    if buf.is_null() {
        update_last_error("buf parameter must not be null.".to_string());
        return LibraStatus::InvalidArgument;
    }
    let buffer = slice::from_raw_parts(buf, len);
    let signed_txn: SignedTransaction = match lcs::from_bytes(buffer) {
        Ok(result) => result,
        Err(e) => {
            update_last_error(format!("Error deserializing signed transaction, invalid signed transaction bytes or length: {}", e.to_string()));
            return LibraStatus::InvalidArgument;
        }
    };

    let mut sender = [0u8; ADDRESS_LENGTH];
    sender.copy_from_slice(signed_txn.sender().as_ref());
    let sequence_number = signed_txn.sequence_number();
    let payload = signed_txn.payload();
    let max_gas_amount = signed_txn.max_gas_amount();
    let gas_unit_price = signed_txn.gas_unit_price();
    let expiration_time_secs = signed_txn.expiration_time().as_secs();
    let public_key = signed_txn.public_key();
    let signature = signed_txn.signature();

    let mut txn_payload = None;

    if let TransactionPayload::Script(script) = payload {
        match get_transaction_name(script.code()).as_str() {
            "peer_to_peer_transaction" => {
                let args = script.args();
                let mut value = None;
                let mut address = None;
                args.iter().for_each(|txn_arg| match txn_arg {
                    TransactionArgument::U64(val) => {
                        value = Some(*val);
                    }
                    TransactionArgument::Address(addr) => {
                        let mut addr_buffer = [0u8; ADDRESS_LENGTH];
                        addr_buffer.copy_from_slice(addr.as_ref());
                        address = Some(addr_buffer);
                    }
                    _ => {}
                });
                if let (Some(val), Some(add)) = (value, address) {
                    txn_payload = Some(LibraTransactionPayload {
                        txn_type: TransactionType::PeerToPeer,
                        args: LibraP2PTransferTransactionArgument {
                            value: val,
                            address: add,
                        },
                    });
                } else {
                    return LibraStatus::InternalError;
                }
            }
            "mint_transaction" => {
                let args = script.args();
                let mut value = None;
                let mut address = None;
                args.iter().for_each(|txn_arg| match txn_arg {
                    TransactionArgument::U64(val) => {
                        value = Some(*val);
                    }
                    TransactionArgument::Address(addr) => {
                        let mut addr_buffer = [0u8; ADDRESS_LENGTH];
                        addr_buffer.copy_from_slice(addr.as_ref());
                        address = Some(addr_buffer);
                    }
                    _ => {}
                });
                if let (Some(val), Some(add)) = (value, address) {
                    txn_payload = Some(LibraTransactionPayload {
                        txn_type: TransactionType::Mint,
                        args: LibraP2PTransferTransactionArgument {
                            value: val,
                            address: add,
                        },
                    });
                } else {
                    return LibraStatus::InternalError;
                }
            }
            &_ => {
                update_last_error("Transaction type not supported".to_string());
                return LibraStatus::InternalError;
            }
        }
    }

    let raw_txn = match txn_payload {
        Some(payload) => LibraRawTransaction {
            sender,
            sequence_number,
            payload,
            max_gas_amount,
            gas_unit_price,
            expiration_time_secs,
        },
        None => {
            let raw_txn_other = LibraRawTransaction {
                sender,
                sequence_number,
                payload: LibraTransactionPayload {
                    txn_type: TransactionType::Unknown,
                    args: LibraP2PTransferTransactionArgument::default(),
                },
                max_gas_amount,
                gas_unit_price,
                expiration_time_secs,
            };
            *out = LibraSignedTransaction {
                raw_txn: raw_txn_other,
                public_key: public_key.to_bytes(),
                signature: signature.to_bytes(),
            };
            return LibraStatus::Ok;
        }
    };

    *out = LibraSignedTransaction {
        raw_txn,
        public_key: public_key.to_bytes(),
        signature: signature.to_bytes(),
    };

    LibraStatus::Ok
}

/// Generate a Signed Transaction and deserialize
#[test]
fn test_lcs_signed_transaction() {
    use lcs::from_bytes;
    use libra_crypto::test_utils::TEST_SEED;
    use libra_types::transaction::{SignedTransaction, TransactionArgument};
    use rand::{rngs::StdRng, SeedableRng};

    // generate key pair
    let mut rng = StdRng::from_seed(TEST_SEED);
    let key_pair = compat::generate_keypair(&mut rng);
    let private_key = key_pair.0;
    let public_key = key_pair.1;
    let private_key_bytes = private_key.to_bytes();

    // create transfer parameters
    let sender_address = AccountAddress::from_public_key(&public_key);
    let receiver_address = AccountAddress::random();
    let sequence = 0;
    let amount = 100_000_000;
    let gas_unit_price = 123;
    let max_gas_amount = 1000;
    let expiration_time_secs = 0;

    let mut buf: *mut u8 = std::ptr::null_mut();
    let buf_ptr = &mut buf;
    let mut len: usize = 0;

    let result = unsafe {
        libra_SignedTransactionBytes_from(
            private_key_bytes.as_ptr(),
            receiver_address.as_ref().as_ptr(),
            sequence,
            amount,
            max_gas_amount,
            gas_unit_price,
            expiration_time_secs,
            buf_ptr,
            &mut len,
        )
    };

    assert_eq!(result, LibraStatus::Ok);

    let signed_txn_bytes_buf: &[u8] = unsafe { slice::from_raw_parts(buf, len) };
    let deserialized_signed_txn: SignedTransaction =
        from_bytes(signed_txn_bytes_buf).expect("LCS deserialization failed");

    if let TransactionPayload::Script(program) = deserialized_signed_txn.payload() {
        if let TransactionArgument::U64(val) = program.args()[1] {
            assert_eq!(val, amount);
        }
    }
    assert_eq!(deserialized_signed_txn.sender(), sender_address);
    assert_eq!(deserialized_signed_txn.sequence_number(), 0);
    assert_eq!(deserialized_signed_txn.gas_unit_price(), gas_unit_price);
    assert_eq!(deserialized_signed_txn.public_key(), public_key);
    assert!(deserialized_signed_txn.check_signature().is_ok());

    // Test signature is stable
    let mut buf2: *mut u8 = std::ptr::null_mut();
    let buf_ptr2 = &mut buf2;
    let mut len2: usize = 0;
    let result2 = unsafe {
        libra_SignedTransactionBytes_from(
            private_key_bytes.as_ptr(),
            receiver_address.as_ref().as_ptr(),
            sequence,
            amount,
            max_gas_amount,
            gas_unit_price,
            expiration_time_secs,
            buf_ptr2,
            &mut len2,
        )
    };

    assert_eq!(result2, LibraStatus::Ok);
    assert_ne!(buf, buf2);
    assert_eq!(len, len2);

    unsafe {
        let data2 = slice::from_raw_parts(buf2, len2);
        assert_eq!(signed_txn_bytes_buf, data2);
    }

    unsafe {
        libra_free_bytes_buffer(buf);
        libra_free_bytes_buffer(buf2);
    };
}

/// Generate a Raw Transaction, sign it, then deserialize
#[test]
fn test_libra_raw_transaction_bytes_from() {
    use lcs::from_bytes;
    use libra_crypto::{hash::CryptoHash, test_utils::TEST_SEED, traits::SigningKey};
    use libra_types::transaction::{SignedTransaction, TransactionArgument};
    use rand::{rngs::StdRng, SeedableRng};

    // generate key pair
    let mut rng = StdRng::from_seed(TEST_SEED);
    let key_pair = compat::generate_keypair(&mut rng);
    let private_key = key_pair.0;
    let public_key = key_pair.1;

    // create transfer parameters
    let sender_address = AccountAddress::from_public_key(&public_key);
    let receiver_address = AccountAddress::random();
    let sequence = 0;
    let amount = 100_000_000;
    let gas_unit_price = 123;
    let max_gas_amount = 1000;
    let expiration_time_secs = 0;

    // get raw transaction in bytes
    let mut buf: u8 = 0;
    let mut buf_ptr: *mut u8 = &mut buf;
    let mut len: usize = 0;
    unsafe {
        libra_RawTransactionBytes_from(
            sender_address.as_ref().as_ptr(),
            receiver_address.as_ref().as_ptr(),
            sequence,
            amount,
            max_gas_amount,
            gas_unit_price,
            expiration_time_secs,
            &mut buf_ptr,
            &mut len,
        )
    };

    // deserialize raw txn and sign
    let raw_txn_bytes: &[u8] = unsafe { slice::from_raw_parts(buf_ptr, len) };
    let deserialized_raw_txn: RawTransaction =
        from_bytes(raw_txn_bytes).expect("LCS deserialization failed for raw transaction");
    let signature = private_key.sign_message(&deserialized_raw_txn.hash());

    // get signed transaction by signing raw transaction
    let mut signed_txn_buf: u8 = 0;
    let mut signed_txn_buf_ptr: *mut u8 = &mut signed_txn_buf;
    let mut signed_txn_len: usize = 0;
    unsafe {
        libra_RawTransaction_sign(
            raw_txn_bytes.as_ptr(),
            raw_txn_bytes.len(),
            public_key.to_bytes().as_ptr(),
            public_key.to_bytes().len(),
            signature.to_bytes().as_ptr(),
            signature.to_bytes().len(),
            &mut signed_txn_buf_ptr,
            &mut signed_txn_len,
        )
    };

    let signed_txn_bytes: &[u8] =
        unsafe { slice::from_raw_parts(signed_txn_buf_ptr, signed_txn_len) };
    let deserialized_signed_txn: SignedTransaction =
        from_bytes(signed_txn_bytes).expect("LCS deserialization failed for signed transaction");

    // test values equal
    if let TransactionPayload::Script(program) = deserialized_signed_txn.payload() {
        if let TransactionArgument::U64(val) = program.args()[1] {
            assert_eq!(val, amount);
        }
    }
    assert_eq!(deserialized_signed_txn.sender(), sender_address);
    assert_eq!(deserialized_signed_txn.sequence_number(), 0);
    assert_eq!(deserialized_signed_txn.gas_unit_price(), gas_unit_price);
    assert_eq!(deserialized_signed_txn.public_key(), public_key);
    assert!(deserialized_signed_txn.check_signature().is_ok());

    // free memory
    unsafe {
        libra_free_bytes_buffer(buf_ptr);
        libra_free_bytes_buffer(signed_txn_buf_ptr);
    };
}

/// Generate a Signed Transaction and deserialize
#[test]
fn test_libra_signed_transaction_deserialize() {
    use std::ffi::CStr;

    let keypair = compat::generate_keypair(None);
    let sender = AccountAddress::random();
    let receiver = AccountAddress::random();
    let sequence_number = 1;
    let amount = 10_000_000;
    let max_gas_amount = 10;
    let gas_unit_price = 1;
    let expiration_time_secs = 5;
    let public_key = keypair.1;
    let signature = Ed25519Signature::try_from(&[1u8; ED25519_SIGNATURE_LENGTH][..]).unwrap();

    let program = encode_transfer_script(&receiver, amount);
    let signed_txn = SignedTransaction::new(
        RawTransaction::new_script(
            sender,
            sequence_number,
            program,
            max_gas_amount,
            gas_unit_price,
            Duration::from_secs(expiration_time_secs),
        ),
        public_key.clone(),
        signature.clone(),
    );
    let proto_txn: libra_types::proto::types::SignedTransaction = signed_txn.clone().into();

    let mut libra_signed_txn = LibraSignedTransaction::default();
    let result = unsafe {
        libra_LibraSignedTransaction_from(
            proto_txn.txn_bytes.as_ptr(),
            proto_txn.txn_bytes.len() - 1, // pass in wrong length so that SignedTransaction cannot deserialize
            &mut libra_signed_txn,
        )
    };
    assert_eq!(result, LibraStatus::InvalidArgument);

    unsafe {
        let error_msg = libra_strerror();
        let error_string: &CStr = CStr::from_ptr(error_msg);
        assert_eq!(error_string.to_str().unwrap(), "Error deserializing signed transaction, invalid signed transaction bytes or length: unexpected end of input");
    };

    let result = unsafe {
        libra_LibraSignedTransaction_from(
            proto_txn.txn_bytes.as_ptr(),
            proto_txn.txn_bytes.len(),
            &mut libra_signed_txn,
        )
    };

    assert_eq!(result, LibraStatus::Ok);

    unsafe {
        let error_msg = libra_strerror();
        let error_string: &CStr = CStr::from_ptr(error_msg);
        assert_eq!(error_string.to_str().unwrap(), "");
    };

    let payload = signed_txn.payload();
    if let TransactionPayload::Script(_script) = payload {
        assert_eq!(
            TransactionType::PeerToPeer,
            libra_signed_txn.raw_txn.payload.txn_type
        );
        assert_eq!(
            receiver,
            AccountAddress::new(libra_signed_txn.raw_txn.payload.args.address)
        );
        assert_eq!(amount, libra_signed_txn.raw_txn.payload.args.value);
    }
    assert_eq!(sender, AccountAddress::new(libra_signed_txn.raw_txn.sender));
    assert_eq!(sequence_number, libra_signed_txn.raw_txn.sequence_number);
    assert_eq!(max_gas_amount, libra_signed_txn.raw_txn.max_gas_amount);
    assert_eq!(gas_unit_price, libra_signed_txn.raw_txn.gas_unit_price);
    assert_eq!(public_key.to_bytes(), libra_signed_txn.public_key);
    assert_eq!(
        signature,
        Ed25519Signature::try_from(libra_signed_txn.signature.as_ref()).unwrap()
    );
    assert_eq!(
        expiration_time_secs,
        libra_signed_txn.raw_txn.expiration_time_secs
    );
}
