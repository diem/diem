// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    data::{LibraAccountKey, LibraStatus},
    error::*,
};
use libra_crypto::{ed25519, PrivateKeyExt};
use libra_types::account_address::AccountAddress;
use std::{convert::TryFrom, slice};

/// Takes in private key in bytes and return the associated public key and address
#[no_mangle]
pub unsafe extern "C" fn libra_LibraAccountKey_from(
    private_key_bytes: *const u8,
    out: *mut LibraAccountKey,
) -> LibraStatus {
    clear_error();
    if private_key_bytes.is_null() {
        update_last_error("private_key_bytes parameter must not be null.".to_string());
        return LibraStatus::InvalidArgument;
    }

    let private_key_buf: &[u8] =
        slice::from_raw_parts(private_key_bytes, ed25519::PrivateKey::LENGTH);

    let private_key = match ed25519::PrivateKey::try_from(private_key_buf) {
        Ok(result) => result,
        Err(e) => {
            update_last_error(format!("Invalid private key bytes: {}", e.to_string()));
            return LibraStatus::InvalidArgument;
        }
    };
    let public_key = private_key.public_key();
    let address = AccountAddress::from_public_key(&public_key);

    *out = LibraAccountKey {
        address: address.into(),
        private_key: private_key.to_bytes(),
        public_key: public_key.to_bytes(),
    };

    LibraStatus::Ok
}

/// Generate a private key, then get LibraAccount
#[test]
fn test_libra_account_from() {
    use libra_crypto::Uniform;

    let private_key = ed25519::PrivateKey::generate_for_testing();
    let mut libra_account = LibraAccountKey::default();
    let result =
        unsafe { libra_LibraAccountKey_from(private_key.to_bytes().as_ptr(), &mut libra_account) };
    assert_eq!(result, LibraStatus::Ok);

    let public_key = private_key.public_key();
    let address = AccountAddress::from_public_key(&public_key);

    assert_eq!(libra_account.public_key, public_key.to_bytes());
    assert_eq!(libra_account.private_key, private_key.to_bytes());
    assert_eq!(AccountAddress::new(libra_account.address), address);
}
