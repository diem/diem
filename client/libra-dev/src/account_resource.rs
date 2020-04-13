// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::{
    data::{LibraAccountResource, LibraEventHandle, LibraStatus},
    error::*,
};
use libra_crypto::ed25519;
use libra_types::{account_state::AccountState, account_state_blob::AccountStateBlob};
use std::{convert::TryFrom, slice};

pub fn libra_LibraAccountResource_from_safe(
    blob: AccountStateBlob,
) -> Result<LibraAccountResource, LibraStatus> {
    clear_error();
    if let Ok(account_state) = AccountState::try_from(&blob) {
        if let Ok(Some(account_resource)) = account_state.get_account_resource() {
            if let Ok(Some(balance_resource)) = account_state.get_balance_resource() {
                let mut authentication_key = [0u8; ed25519::PUBLIC_KEY_LENGTH];
                authentication_key.copy_from_slice(account_resource.authentication_key());

                let sent_events = LibraEventHandle {
                    count: account_resource.sent_events().count(),
                    key: account_resource.sent_events().key().into(),
                };
                let received_events = LibraEventHandle {
                    count: account_resource.received_events().count(),
                    key: account_resource.received_events().key().into(),
                };

                return Ok(LibraAccountResource {
                    balance: balance_resource.coin(),
                    sequence: account_resource.sequence_number(),
                    delegated_key_rotation_capability: account_resource
                        .delegated_key_rotation_capability(),
                    delegated_withdrawal_capability: account_resource
                        .delegated_withdrawal_capability(),
                    sent_events,
                    received_events,
                    authentication_key,
                });
            }
        }
    }
    update_last_error(format!(
        "Error deserializing account state blob: {:?}",
        blob
    ));
    Err(LibraStatus::InvalidArgument)
}

#[no_mangle]
pub unsafe extern "C" fn libra_LibraAccountResource_from(
    buf: *const u8,
    len: usize,
    out: *mut LibraAccountResource,
) -> LibraStatus {
    clear_error();
    if buf.is_null() {
        update_last_error("buf parameter must not be null.".to_string());
        return LibraStatus::InvalidArgument;
    }
    let buf: &[u8] = slice::from_raw_parts(buf, len);
    let account_state_blob = AccountStateBlob::from(buf.to_vec());

    match libra_LibraAccountResource_from_safe(account_state_blob) {
        Ok(res) => *out = res,
        Err(err) => return err,
    }
    LibraStatus::Ok
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Generate an AccountBlob and verify we can parse it
    #[test]
    fn test_get_accountresource() {
        use libra_crypto::{ed25519, PrivateKeyExt, Uniform};
        use libra_types::{
            account_config::{AccountResource, BalanceResource},
            event::{EventHandle, EventKey},
            move_resource::MoveResource,
            transaction::authenticator::AuthenticationKey,
        };
        use std::collections::BTreeMap;

        let pubkey = ed25519::PrivateKey::generate_for_testing().public_key();

        // Figure out how to use Libra code to generate AccountStateBlob directly, not involving btreemap directly
        let mut map: BTreeMap<Vec<u8>, Vec<u8>> = BTreeMap::new();
        let auth_key = AuthenticationKey::ed25519(&pubkey);
        let addr = auth_key.derived_address();
        let ar = AccountResource::new(
            123_456_789,
            auth_key.to_vec(),
            true,
            false,
            EventHandle::new(EventKey::new_from_address(&addr, 0), 777),
            EventHandle::new(EventKey::new_from_address(&addr, 0), 888),
            0,
        );
        let br = BalanceResource::new(100);

        // Fill in data
        map.insert(
            AccountResource::resource_path(),
            lcs::to_bytes(&ar).expect("Account resource lcs serialization was not successful"),
        );
        map.insert(
            BalanceResource::resource_path(),
            lcs::to_bytes(&br).expect("Balance resource lcs serialization was not successful"),
        );

        let account_state_blob = lcs::to_bytes(&map).expect("LCS serialization failed");

        let mut result = LibraAccountResource::default();
        assert_eq!(LibraStatus::Ok, unsafe {
            libra_LibraAccountResource_from(
                account_state_blob.as_ptr(),
                account_state_blob.len(),
                &mut result,
            )
        });

        assert_eq!(result.sequence, ar.sequence_number());
        assert_eq!(result.authentication_key, ar.authentication_key());
        assert_eq!(
            result.delegated_key_rotation_capability,
            ar.delegated_key_rotation_capability()
        );
        assert_eq!(
            result.delegated_withdrawal_capability,
            ar.delegated_withdrawal_capability()
        );
        assert_eq!(result.sent_events.count, ar.sent_events().count());
        assert_eq!(
            EventKey::new(result.sent_events.key),
            *ar.sent_events().key()
        );

        assert_eq!(result.received_events.count, ar.received_events().count());
        assert_eq!(
            EventKey::new(result.received_events.key),
            *ar.received_events().key()
        );
    }
}
