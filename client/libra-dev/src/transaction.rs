// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    error::*,
    interface::{
        LibraP2PTransferTransactionArgument, LibraRawTransaction, LibraSignedTransaction,
        LibraStatus, LibraTransactionPayload, TransactionType, LIBRA_PUBKEY_SIZE,
        LIBRA_SIGNATURE_SIZE,
    },
};
use lcs::{from_bytes, to_bytes};
use libra_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey, Ed25519Signature, ED25519_PUBLIC_KEY_LENGTH},
    test_utils::KeyPair,
    PrivateKey,
};
use libra_types::{
    account_address::{self, AccountAddress},
    account_config::{
        from_currency_code_string, lbr_type_tag, type_tag_for_currency_code, LBR_NAME,
    },
    chain_id::ChainId,
    transaction::{
        authenticator::AuthenticationKey, helpers::TransactionSigner, RawTransaction, Script,
        SignedTransaction, TransactionArgument, TransactionPayload,
    },
};
use std::{convert::TryFrom, ffi::CStr, slice};
use transaction_builder::{
    encode_add_currency_to_account_script, encode_peer_to_peer_with_metadata_script,
    encode_rotate_dual_attestation_info_script, get_transaction_name,
};

#[no_mangle]
pub unsafe extern "C" fn libra_SignedTransactionBytes_from(
    sender_private_key_bytes: *const u8,
    sequence: u64,
    max_gas_amount: u64,
    gas_unit_price: u64,
    gas_identifier: *const i8,
    expiration_timestamp_secs: u64,
    chain_id: u8,
    script_bytes: *const u8,
    script_len: usize,
    ptr_buf: *mut *mut u8,
    ptr_len: *mut usize,
) -> LibraStatus {
    clear_error();

    if sender_private_key_bytes.is_null() {
        update_last_error("sender_private_key_bytes parameter must not be null.".to_string());
        return LibraStatus::InvalidArgument;
    }
    let private_key_buf: &[u8] =
        slice::from_raw_parts(sender_private_key_bytes, Ed25519PrivateKey::LENGTH);
    let private_key = match Ed25519PrivateKey::try_from(private_key_buf) {
        Ok(result) => result,
        Err(e) => {
            update_last_error(format!("Invalid private key bytes: {}", e.to_string()));
            return LibraStatus::InvalidArgument;
        }
    };

    if script_bytes.is_null() {
        update_last_error("script_bytes parameter must not be null.".to_string());
        return LibraStatus::InvalidArgument;
    }
    let script_buf: &[u8] = slice::from_raw_parts(script_bytes, script_len);

    let script: Script = match from_bytes(script_buf) {
        Ok(result) => result,
        Err(e) => {
            update_last_error(format!("Invalid script bytes: {}", e.to_string()));
            return LibraStatus::InvalidArgument;
        }
    };

    let public_key = private_key.public_key();
    let sender_address = account_address::from_public_key(&public_key);

    let payload = TransactionPayload::Script(script);
    let raw_txn = RawTransaction::new(
        sender_address,
        sequence,
        payload,
        max_gas_amount,
        gas_unit_price,
        CStr::from_ptr(gas_identifier)
            .to_string_lossy()
            .into_owned(),
        expiration_timestamp_secs,
        ChainId::new(chain_id),
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
pub unsafe extern "C" fn libra_TransactionP2PScript_from(
    receiver: *const u8,
    identifier: *const i8,
    num_coins: u64,
    metadata_bytes: *const u8,
    metadata_len: usize,
    metadata_signature_bytes: *const u8,
    metadata_signature_len: usize,
    ptr_buf: *mut *mut u8,
    ptr_len: *mut usize,
) -> LibraStatus {
    clear_error();

    if receiver.is_null() {
        update_last_error("receiver parameter must not be null.".to_string());
        return LibraStatus::InvalidArgument;
    }
    let receiver_buf = slice::from_raw_parts(receiver, AccountAddress::LENGTH);
    let receiver_address = match AccountAddress::try_from(receiver_buf) {
        Ok(result) => result,
        Err(e) => {
            update_last_error(format!(
                "Invalid receiver account address: {}",
                e.to_string()
            ));
            return LibraStatus::InvalidArgument;
        }
    };

    let metadata = if metadata_bytes.is_null() {
        vec![]
    } else {
        slice::from_raw_parts(metadata_bytes, metadata_len).to_vec()
    };
    let metadata_signature = if metadata_signature_bytes.is_null() {
        vec![]
    } else {
        slice::from_raw_parts(metadata_signature_bytes, metadata_signature_len).to_vec()
    };

    let coin_type_tag = match from_currency_code_string(
        CStr::from_ptr(identifier)
            .to_string_lossy()
            .into_owned()
            .as_str(),
    ) {
        Ok(coin_ident) => type_tag_for_currency_code(coin_ident),
        Err(e) => {
            update_last_error(format!("Invalid coin identifier: {}", e.to_string()));
            return LibraStatus::InvalidArgument;
        }
    };

    let script = encode_peer_to_peer_with_metadata_script(
        coin_type_tag,
        receiver_address,
        num_coins,
        metadata,
        metadata_signature,
    );

    let script_bytes = match to_bytes(&script) {
        Ok(result) => result,
        Err(e) => {
            update_last_error(format!("Error serializing P2P Script: {}", e.to_string()));
            return LibraStatus::InternalError;
        }
    };

    let script_buf: *mut u8 = libc::malloc(script_bytes.len()).cast();
    script_buf.copy_from(script_bytes.as_ptr(), script_bytes.len());

    *ptr_buf = script_buf;
    *ptr_len = script_bytes.len();

    LibraStatus::Ok
}

#[no_mangle]
pub unsafe extern "C" fn libra_TransactionAddCurrencyScript_from(
    identifier: *const i8,
    ptr_buf: *mut *mut u8,
    ptr_len: *mut usize,
) -> LibraStatus {
    clear_error();

    let coin_type_tag = match from_currency_code_string(
        CStr::from_ptr(identifier)
            .to_string_lossy()
            .into_owned()
            .as_str(),
    ) {
        Ok(coin_ident) => type_tag_for_currency_code(coin_ident),
        Err(e) => {
            update_last_error(format!("Invalid coin identifier: {}", e.to_string()));
            return LibraStatus::InvalidArgument;
        }
    };

    let script = encode_add_currency_to_account_script(coin_type_tag);

    let script_bytes = match to_bytes(&script) {
        Ok(result) => result,
        Err(e) => {
            update_last_error(format!(
                "Error serializing add currency to account Script: {}",
                e.to_string()
            ));
            return LibraStatus::InternalError;
        }
    };

    let script_buf: *mut u8 = libc::malloc(script_bytes.len()).cast();
    script_buf.copy_from(script_bytes.as_ptr(), script_bytes.len());

    *ptr_buf = script_buf;
    *ptr_len = script_bytes.len();

    LibraStatus::Ok
}

#[no_mangle]
pub unsafe extern "C" fn libra_TransactionRotateDualAttestationInfoScript_from(
    new_url_bytes: *const u8,
    new_url_len: usize,
    new_key_bytes: *const u8,
    ptr_buf: *mut *mut u8,
    ptr_len: *mut usize,
) -> LibraStatus {
    clear_error();

    let new_url = if new_url_bytes.is_null() {
        vec![]
    } else {
        slice::from_raw_parts(new_url_bytes, new_url_len).to_vec()
    };

    if new_key_bytes.is_null() {
        update_last_error("new_key_bytes parameter must not be null.".to_string());
        return LibraStatus::InvalidArgument;
    }
    let new_key_buf: &[u8] = slice::from_raw_parts(new_key_bytes, ED25519_PUBLIC_KEY_LENGTH);
    let new_compliance_public_key = match Ed25519PublicKey::try_from(new_key_buf) {
        Ok(result) => result,
        Err(e) => {
            update_last_error(format!(
                "Invalid compliance public key bytes: {}",
                e.to_string()
            ));
            return LibraStatus::InvalidArgument;
        }
    };

    let script = encode_rotate_dual_attestation_info_script(
        new_url,
        new_compliance_public_key.to_bytes().to_vec(),
    );

    let script_bytes = match to_bytes(&script) {
        Ok(result) => result,
        Err(e) => {
            update_last_error(format!(
                "Error serializing rotate compliance public key Script: {}",
                e.to_string()
            ));
            return LibraStatus::InternalError;
        }
    };

    let script_buf: *mut u8 = libc::malloc(script_bytes.len()).cast();
    script_buf.copy_from(script_bytes.as_ptr(), script_bytes.len());

    *ptr_buf = script_buf;
    *ptr_len = script_bytes.len();

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
    expiration_timestamp_secs: u64,
    chain_id: u8,
    metadata_bytes: *const u8,
    metadata_len: usize,
    metadata_signature_bytes: *const u8,
    metadata_signature_len: usize,
    buf: *mut *mut u8,
    len: *mut usize,
) -> LibraStatus {
    clear_error();

    if sender.is_null() {
        update_last_error("sender parameter must not be null.".to_string());
        return LibraStatus::InvalidArgument;
    }
    let sender_buf = slice::from_raw_parts(sender, AccountAddress::LENGTH);
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
    let receiver_buf = slice::from_raw_parts(receiver, AccountAddress::LENGTH);
    let receiver_address = match AccountAddress::try_from(receiver_buf) {
        Ok(result) => result,
        Err(e) => {
            update_last_error(format!("Invalid receiver address: {}", e.to_string()));
            return LibraStatus::InvalidArgument;
        }
    };

    let metadata = if metadata_bytes.is_null() {
        vec![]
    } else {
        slice::from_raw_parts(metadata_bytes, metadata_len).to_vec()
    };
    let metadata_signature = if metadata_signature_bytes.is_null() {
        vec![]
    } else {
        slice::from_raw_parts(metadata_signature_bytes, metadata_signature_len).to_vec()
    };

    let program = encode_peer_to_peer_with_metadata_script(
        lbr_type_tag(),
        receiver_address,
        num_coins,
        metadata,
        metadata_signature,
    );
    let payload = TransactionPayload::Script(program);
    let raw_txn = RawTransaction::new(
        sender_address,
        sequence,
        payload,
        max_gas_amount,
        gas_unit_price,
        LBR_NAME.to_owned(),
        expiration_timestamp_secs,
        ChainId::new(chain_id),
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

    let sender = signed_txn.sender().into();
    let sequence_number = signed_txn.sequence_number();
    let payload = signed_txn.payload();
    let max_gas_amount = signed_txn.max_gas_amount();
    let gas_unit_price = signed_txn.gas_unit_price();
    let expiration_timestamp_secs = signed_txn.expiration_timestamp_secs();
    let chain_id = signed_txn.chain_id().id();
    // TODO: this will not work with multisig transactions, where both the pubkey and signature
    // have different sizes than the ones expected here. We will either need LibraSignedTransaction
    // types for single and multisig authenticators or adapt the type to work  with both
    // authenticators
    let public_key_bytes = signed_txn.authenticator().public_key_bytes();
    let signature_bytes = signed_txn.authenticator().signature_bytes();
    let mut public_key = [0; LIBRA_PUBKEY_SIZE as usize];
    let mut signature = [0; LIBRA_SIGNATURE_SIZE as usize];

    let public_key_bytes = &public_key_bytes[..public_key.len()];
    public_key.copy_from_slice(public_key_bytes);
    let signature_bytes = &signature_bytes[..signature.len()];
    signature.copy_from_slice(signature_bytes);

    let mut txn_payload = None;

    if let TransactionPayload::Script(script) = payload {
        match get_transaction_name(script.code()).as_str() {
            "mint_transaction" => {
                let args = script.args();
                if let [TransactionArgument::Address(addr), TransactionArgument::U8Vector(auth_key_prefix), TransactionArgument::U64(amount)] =
                    &args[..]
                {
                    let mut auth_key_prefix_buffer =
                        [0u8; AuthenticationKey::LENGTH - AccountAddress::LENGTH];
                    auth_key_prefix_buffer.copy_from_slice(auth_key_prefix.as_slice());

                    txn_payload = Some(LibraTransactionPayload {
                        txn_type: TransactionType::PeerToPeer,
                        args: LibraP2PTransferTransactionArgument {
                            value: *amount,
                            address: addr.into(),
                            metadata_bytes: vec![].as_ptr(),
                            metadata_len: 0,
                            metadata_signature_bytes: vec![].as_ptr(),
                            metadata_signature_len: 0,
                        },
                    });
                } else {
                    update_last_error("Fail to decode transaction payload".to_string());
                    return LibraStatus::InternalError;
                }
            }
            "peer_to_peer_with_metadata_transaction" => {
                let args = script.args();
                if let [TransactionArgument::Address(addr), TransactionArgument::U64(amount), TransactionArgument::U8Vector(metadata), TransactionArgument::U8Vector(metadata_signature)] =
                    &args[..]
                {
                    txn_payload = Some(LibraTransactionPayload {
                        txn_type: TransactionType::PeerToPeer,
                        args: LibraP2PTransferTransactionArgument {
                            value: *amount,
                            address: addr.into(),
                            metadata_bytes: (*Box::into_raw(metadata.clone().into_boxed_slice()))
                                .as_ptr(),
                            metadata_len: metadata.len(),
                            metadata_signature_bytes: (*Box::into_raw(
                                metadata_signature.clone().into_boxed_slice(),
                            ))
                            .as_ptr(),
                            metadata_signature_len: metadata_signature.len(),
                        },
                    });
                } else {
                    update_last_error("Fail to decode transaction payload".to_string());
                    return LibraStatus::InternalError;
                }
            }
            _ => {
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
            expiration_timestamp_secs,
            chain_id,
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
                expiration_timestamp_secs,
                chain_id,
            };
            *out = LibraSignedTransaction {
                raw_txn: raw_txn_other,
                public_key,
                signature,
            };
            return LibraStatus::Ok;
        }
    };

    *out = LibraSignedTransaction {
        raw_txn,
        public_key,
        signature,
    };

    LibraStatus::Ok
}

#[cfg(test)]
mod test {
    use super::*;
    use lcs::from_bytes;
    use libra_crypto::{PrivateKey, SigningKey, Uniform};
    use libra_types::{
        account_config::COIN1_NAME,
        transaction::{SignedTransaction, TransactionArgument},
    };
    use move_core_types::language_storage::TypeTag;
    use std::ffi::CStr;

    /// Generate a Signed Transaction and deserialize
    #[test]
    fn test_lcs_signed_transaction() {
        // generate key pair
        let private_key = Ed25519PrivateKey::generate_for_testing();
        let public_key = private_key.public_key();
        let private_key_bytes = private_key.to_bytes();

        // create transfer parameters
        let sender_address = account_address::from_public_key(&public_key);
        let receiver_address = AccountAddress::random();
        let sequence = 0;
        let amount = 100_000_000;
        let gas_unit_price = 123;
        let max_gas_amount = 1000;
        let expiration_timestamp_secs = 0;
        let metadata = vec![1, 2, 3];
        let metadata_signature = [0x1; 64].to_vec();
        let coin_ident = std::ffi::CString::new(LBR_NAME).expect("Invalid ident");

        let mut script_buf: *mut u8 = std::ptr::null_mut();
        let script_buf_ptr = &mut script_buf;
        let mut script_len: usize = 0;

        let script_result = unsafe {
            libra_TransactionP2PScript_from(
                receiver_address.as_ref().as_ptr(),
                coin_ident.as_ptr(),
                amount,
                metadata.as_ptr(),
                metadata.len(),
                metadata_signature.as_ptr(),
                metadata_signature.len(),
                script_buf_ptr,
                &mut script_len,
            )
        };
        assert_eq!(script_result, LibraStatus::Ok);

        let script_bytes: &[u8] = unsafe { slice::from_raw_parts(script_buf, script_len) };
        let _deserialized_script: Script =
            from_bytes(script_bytes).expect("LCS deserialization failed for Script");

        let mut buf: *mut u8 = std::ptr::null_mut();
        let buf_ptr = &mut buf;
        let mut len: usize = 0;

        let result = unsafe {
            libra_SignedTransactionBytes_from(
                private_key_bytes.as_ptr(),
                sequence,
                max_gas_amount,
                gas_unit_price,
                coin_ident.as_ptr(),
                expiration_timestamp_secs,
                ChainId::test().id(),
                script_bytes.as_ptr(),
                script_len,
                buf_ptr,
                &mut len,
            )
        };

        assert_eq!(result, LibraStatus::Ok);

        let signed_txn_bytes_buf: &[u8] = unsafe { slice::from_raw_parts(buf, len) };
        let deserialized_signed_txn: SignedTransaction =
            from_bytes(signed_txn_bytes_buf).expect("LCS deserialization failed");

        if let TransactionPayload::Script(program) = deserialized_signed_txn.payload() {
            match program.args()[1] {
                TransactionArgument::U64(val) => assert_eq!(val, amount),
                _ => unreachable!(),
            }
            match &program.args()[2] {
                TransactionArgument::U8Vector(val) => assert_eq!(val, &metadata),
                _ => unreachable!(),
            }
            match &program.args()[3] {
                TransactionArgument::U8Vector(val) => assert_eq!(val, &metadata_signature),
                _ => unreachable!(),
            }
        }
        assert_eq!(deserialized_signed_txn.sender(), sender_address);
        assert_eq!(deserialized_signed_txn.sequence_number(), 0);
        assert_eq!(deserialized_signed_txn.gas_unit_price(), gas_unit_price);
        assert_eq!(
            deserialized_signed_txn.authenticator().public_key_bytes(),
            public_key.to_bytes()
        );
        assert!(deserialized_signed_txn.check_signature().is_ok());

        // Test signature is stable
        let mut buf2: *mut u8 = std::ptr::null_mut();
        let buf_ptr2 = &mut buf2;
        let mut len2: usize = 0;
        let coin_idnet = std::ffi::CString::new(LBR_NAME).expect("Invalid ident");
        let result2 = unsafe {
            libra_SignedTransactionBytes_from(
                private_key_bytes.as_ptr(),
                sequence,
                max_gas_amount,
                gas_unit_price,
                coin_idnet.as_ptr(),
                expiration_timestamp_secs,
                ChainId::test().id(),
                script_bytes.as_ptr(),
                script_len,
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

        // Test creating add currency to account transaction
        let coin_ident_2 = std::ffi::CString::new(COIN1_NAME).expect("Invalid ident");

        let mut script_buf: *mut u8 = std::ptr::null_mut();
        let script_buf_ptr = &mut script_buf;
        let mut script_len: usize = 0;

        let script_result = unsafe {
            libra_TransactionAddCurrencyScript_from(
                coin_ident_2.as_ptr(),
                script_buf_ptr,
                &mut script_len,
            )
        };
        assert_eq!(script_result, LibraStatus::Ok);

        let script_bytes: &[u8] = unsafe { slice::from_raw_parts(script_buf, script_len) };
        let _deserialized_script: Script =
            from_bytes(script_bytes).expect("LCS deserialization failed for Script");

        let mut buf: *mut u8 = std::ptr::null_mut();
        let buf_ptr = &mut buf;
        let mut len: usize = 0;

        let result = unsafe {
            libra_SignedTransactionBytes_from(
                private_key_bytes.as_ptr(),
                sequence,
                max_gas_amount,
                gas_unit_price,
                coin_ident_2.as_ptr(),
                expiration_timestamp_secs,
                ChainId::test().id(),
                script_bytes.as_ptr(),
                script_len,
                buf_ptr,
                &mut len,
            )
        };

        assert_eq!(result, LibraStatus::Ok);

        let signed_txn_bytes_buf: &[u8] = unsafe { slice::from_raw_parts(buf, len) };
        let deserialized_signed_txn: SignedTransaction =
            from_bytes(signed_txn_bytes_buf).expect("LCS deserialization failed");

        let identifier = from_currency_code_string(
            coin_ident_2
                .as_c_str()
                .to_string_lossy()
                .into_owned()
                .as_str(),
        )
        .unwrap();

        if let TransactionPayload::Script(program) = deserialized_signed_txn.payload() {
            match program.ty_args()[0].clone() {
                TypeTag::Struct(struct_tag) => assert_eq!(struct_tag.module, identifier),
                _ => unreachable!(),
            }
        }
        assert_eq!(deserialized_signed_txn.sender(), sender_address);
        assert_eq!(deserialized_signed_txn.sequence_number(), 0);
        assert_eq!(deserialized_signed_txn.gas_unit_price(), gas_unit_price);
        assert_eq!(
            deserialized_signed_txn.authenticator().public_key_bytes(),
            public_key.to_bytes()
        );
        assert!(deserialized_signed_txn.check_signature().is_ok());

        unsafe {
            libra_free_bytes_buffer(script_buf);
            libra_free_bytes_buffer(buf);
            libra_free_bytes_buffer(buf2);
        };
    }

    /// Generate a P2P Transaction Script and deserialize
    #[test]
    fn test_lcs_p2p_transaction_script() {
        // create transfer parameters
        let receiver_address = AccountAddress::random();
        let amount = 100_000_000;
        let metadata = vec![1, 2, 3];
        let metadata_signature = [0x1; 64].to_vec();
        let coin_ident = std::ffi::CString::new(LBR_NAME).expect("Invalid ident");

        let mut script_buf: *mut u8 = std::ptr::null_mut();
        let script_buf_ptr = &mut script_buf;
        let mut script_len: usize = 0;

        let script_result = unsafe {
            libra_TransactionP2PScript_from(
                receiver_address.as_ref().as_ptr(),
                coin_ident.as_ptr(),
                amount,
                metadata.as_ptr(),
                metadata.len(),
                metadata_signature.as_ptr(),
                metadata_signature.len(),
                script_buf_ptr,
                &mut script_len,
            )
        };

        assert_eq!(script_result, LibraStatus::Ok);

        let script_bytes: &[u8] = unsafe { slice::from_raw_parts(script_buf, script_len) };
        let deserialized_script: Script =
            from_bytes(script_bytes).expect("LCS deserialization failed for Script");

        if let TransactionArgument::Address(val) = deserialized_script.args()[0] {
            assert_eq!(val, receiver_address);
        } else {
            unreachable!()
        }
        if let TransactionArgument::U64(val) = deserialized_script.args()[1] {
            assert_eq!(val, amount);
        } else {
            unreachable!()
        }
        if let TransactionArgument::U8Vector(val) = deserialized_script.args()[2].clone() {
            assert_eq!(val, metadata)
        } else {
            unreachable!()
        }
        if let TransactionArgument::U8Vector(val) = deserialized_script.args()[3].clone() {
            assert_eq!(val, metadata_signature)
        } else {
            unreachable!()
        }

        unsafe {
            libra_free_bytes_buffer(script_buf);
        };
    }

    /// Generate a add currency to account script and deserialize
    #[test]
    fn test_lcs_add_currency_to_account_transaction_script() {
        let coin_ident = std::ffi::CString::new(LBR_NAME).expect("Invalid ident");

        let mut script_buf: *mut u8 = std::ptr::null_mut();
        let script_buf_ptr = &mut script_buf;
        let mut script_len: usize = 0;

        let script_result = unsafe {
            libra_TransactionAddCurrencyScript_from(
                coin_ident.as_ptr(),
                script_buf_ptr,
                &mut script_len,
            )
        };

        assert_eq!(script_result, LibraStatus::Ok);

        let script_bytes: &[u8] = unsafe { slice::from_raw_parts(script_buf, script_len) };
        let deserialized_script: Script =
            from_bytes(script_bytes).expect("LCS deserialization failed for Script");
        let identifier = from_currency_code_string(
            coin_ident
                .as_c_str()
                .to_string_lossy()
                .into_owned()
                .as_str(),
        )
        .unwrap();

        if let TypeTag::Struct(struct_tag) = deserialized_script.ty_args()[0].clone() {
            assert_eq!(identifier, struct_tag.module);
        } else {
            unreachable!()
        }

        unsafe {
            libra_free_bytes_buffer(script_buf);
        };
    }

    /// Generate a RotateDualAttestationInfo Script and deserialize
    #[test]
    fn test_lcs_rotate_dual_attestationInfo_transaction_script() {
        let new_url = b"new_name".to_vec();
        let private_key = Ed25519PrivateKey::generate_for_testing();
        let new_compliance_public_key = private_key.public_key();

        let mut script_buf: *mut u8 = std::ptr::null_mut();
        let script_buf_ptr = &mut script_buf;
        let mut script_len: usize = 0;

        let script_result = unsafe {
            libra_TransactionRotateDualAttestationInfoScript_from(
                new_url.as_ptr(),
                new_url.len(),
                new_compliance_public_key.to_bytes().as_ptr(),
                script_buf_ptr,
                &mut script_len,
            )
        };

        assert_eq!(script_result, LibraStatus::Ok);

        let script_bytes: &[u8] = unsafe { slice::from_raw_parts(script_buf, script_len) };
        let deserialized_script: Script = from_bytes(script_bytes)
            .expect("LCS deserialization failed for rotate base URL Script");

        if let TransactionArgument::U8Vector(val) = deserialized_script.args()[0].clone() {
            assert_eq!(val, new_url);
        } else {
            unreachable!()
        }
        if let TransactionArgument::U8Vector(val) = deserialized_script.args()[1].clone() {
            assert_eq!(val, new_compliance_public_key.to_bytes().to_vec());
        } else {
            unreachable!()
        }

        unsafe {
            libra_free_bytes_buffer(script_buf);
        };
    }

    /// Generate a Raw Transaction, sign it, then deserialize
    #[test]
    fn test_libra_raw_transaction_bytes_from() {
        let private_key = Ed25519PrivateKey::generate_for_testing();
        let public_key = private_key.public_key();

        // create transfer parameters
        let sender_address = account_address::from_public_key(&public_key);
        let receiver_address = AccountAddress::random();
        let sequence = 0;
        let amount = 100_000_000;
        let gas_unit_price = 123;
        let max_gas_amount = 1000;
        let expiration_timestamp_secs = 0;

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
                expiration_timestamp_secs,
                ChainId::test().id(),
                vec![].as_ptr(),
                0,
                vec![].as_ptr(),
                0,
                &mut buf_ptr,
                &mut len,
            )
        };

        // deserialize raw txn and sign
        let raw_txn_bytes: &[u8] = unsafe { slice::from_raw_parts(buf_ptr, len) };
        let deserialized_raw_txn: RawTransaction =
            from_bytes(raw_txn_bytes).expect("LCS deserialization failed for raw transaction");
        let signature = private_key.sign(&deserialized_raw_txn);

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
        let deserialized_signed_txn: SignedTransaction = from_bytes(signed_txn_bytes)
            .expect("LCS deserialization failed for signed transaction");

        // test values equal
        if let TransactionPayload::Script(program) = deserialized_signed_txn.payload() {
            if let TransactionArgument::U64(val) = program.args()[1] {
                assert_eq!(val, amount);
            }
        }
        assert_eq!(deserialized_signed_txn.sender(), sender_address);
        assert_eq!(deserialized_signed_txn.sequence_number(), 0);
        assert_eq!(deserialized_signed_txn.gas_unit_price(), gas_unit_price);
        assert_eq!(
            deserialized_signed_txn.authenticator().public_key_bytes(),
            public_key.to_bytes()
        );
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
        let public_key = Ed25519PrivateKey::generate_for_testing().public_key();
        let sender = AccountAddress::random();
        let receiver = AccountAddress::random();
        let sequence_number = 1;
        let amount = 10_000_000;
        let max_gas_amount = 10;
        let gas_unit_price = 1;
        let expiration_timestamp_secs = 5;
        let metadata = vec![1, 2, 3];
        let metadata_signature = [0x1; 64].to_vec();
        let signature = Ed25519Signature::try_from(&[1u8; Ed25519Signature::LENGTH][..]).unwrap();

        let program = encode_peer_to_peer_with_metadata_script(
            lbr_type_tag(),
            receiver,
            amount,
            metadata.clone(),
            metadata_signature.clone(),
        );
        let signed_txn = SignedTransaction::new(
            RawTransaction::new_script(
                sender,
                sequence_number,
                program,
                max_gas_amount,
                gas_unit_price,
                LBR_NAME.to_owned(),
                expiration_timestamp_secs,
                ChainId::test(),
            ),
            public_key.clone(),
            signature.clone(),
        );
        let txn_bytes = lcs::to_bytes(&signed_txn).expect("Unable to serialize SignedTransaction");

        let mut libra_signed_txn = LibraSignedTransaction::default();
        let result = unsafe {
            libra_LibraSignedTransaction_from(
                txn_bytes.as_ptr(),
                txn_bytes.len() - 1, // pass in wrong length so that SignedTransaction cannot deserialize
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
                txn_bytes.as_ptr(),
                txn_bytes.len(),
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
            assert_eq!(amount, libra_signed_txn.raw_txn.payload.args.value);
            let txn_metadata = unsafe {
                slice::from_raw_parts(
                    libra_signed_txn.raw_txn.payload.args.metadata_bytes,
                    libra_signed_txn.raw_txn.payload.args.metadata_len,
                )
            };
            assert_eq!(metadata, txn_metadata);
            let txn_metadata_signature = unsafe {
                slice::from_raw_parts(
                    libra_signed_txn
                        .raw_txn
                        .payload
                        .args
                        .metadata_signature_bytes,
                    libra_signed_txn.raw_txn.payload.args.metadata_signature_len,
                )
            };
            assert_eq!(metadata_signature, txn_metadata_signature);
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
            expiration_timestamp_secs,
            libra_signed_txn.raw_txn.expiration_timestamp_secs
        );
    }
}
