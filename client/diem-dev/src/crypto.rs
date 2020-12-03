// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    error::*,
    interface::{DiemAccountKey, DiemStatus},
};
use diem_crypto::{
    ed25519::{Ed25519PrivateKey, ED25519_PRIVATE_KEY_LENGTH},
    PrivateKey,
};
use diem_types::account_address;
use std::{convert::TryFrom, slice};

/// Takes in private key in bytes and return the associated public key and address
#[no_mangle]
pub unsafe extern "C" fn diem_DiemAccountKey_from(
    private_key_bytes: *const u8,
    out: *mut DiemAccountKey,
) -> DiemStatus {
    clear_error();
    if private_key_bytes.is_null() {
        update_last_error("private_key_bytes parameter must not be null.".to_string());
        return DiemStatus::InvalidArgument;
    }

    let private_key_buf: &[u8] =
        slice::from_raw_parts(private_key_bytes, ED25519_PRIVATE_KEY_LENGTH);

    let private_key = match Ed25519PrivateKey::try_from(private_key_buf) {
        Ok(result) => result,
        Err(e) => {
            update_last_error(format!("Invalid private key bytes: {}", e.to_string()));
            return DiemStatus::InvalidArgument;
        }
    };
    let public_key = private_key.public_key();
    let address = account_address::from_public_key(&public_key);

    *out = DiemAccountKey {
        address: address.into(),
        private_key: private_key.to_bytes(),
        public_key: public_key.to_bytes(),
    };

    DiemStatus::Ok
}

/// Generate a private key, then get DiemAccount
#[test]
fn test_diem_account_from() {
    use diem_crypto::Uniform;
    use diem_types::account_address::{self, AccountAddress};

    let private_key = Ed25519PrivateKey::generate_for_testing();
    let mut diem_account = DiemAccountKey::default();
    let result =
        unsafe { diem_DiemAccountKey_from(private_key.to_bytes().as_ptr(), &mut diem_account) };
    assert_eq!(result, DiemStatus::Ok);

    let public_key = private_key.public_key();
    let address = account_address::from_public_key(&public_key);

    assert_eq!(diem_account.public_key, public_key.to_bytes());
    assert_eq!(diem_account.private_key, private_key.to_bytes());
    assert_eq!(AccountAddress::new(diem_account.address), address);
}
