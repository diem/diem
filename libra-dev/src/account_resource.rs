// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::data::{CDevAccountResource, CEventHandle};
use libra_types::{
    account_config::get_account_resource_or_default,
    account_state_blob::AccountStateBlob,
    byte_array::ByteArray,
    event::{EventHandle, EVENT_KEY_LENGTH},
};
use std::slice;

#[derive(Debug)]
pub struct DevAccountResource {
    balance: u64,
    sequence_number: u64,
    authentication_key: ByteArray,
    delegated_key_rotation_capability: bool,
    delegated_withdrawal_capability: bool,
    sent_events: EventHandle,
    received_events: EventHandle,
}

impl DevAccountResource {
    pub fn new(account_state_blob: AccountStateBlob) -> Self {
        let account_resource = get_account_resource_or_default(&Some(account_state_blob)).unwrap();

        Self {
            balance: account_resource.balance(),
            sequence_number: account_resource.sequence_number(),
            authentication_key: account_resource.authentication_key().clone(),
            delegated_key_rotation_capability: account_resource.delegated_key_rotation_capability(),
            delegated_withdrawal_capability: account_resource.delegated_withdrawal_capability(),
            sent_events: account_resource.sent_events().clone(),
            received_events: account_resource.received_events().clone(),
        }
    }

    /// Return the authentication_key field for the given AccountResource
    pub fn authentication_key(&self) -> &ByteArray {
        &self.authentication_key
    }

    /// Return the balance field for the given AccountResource
    pub fn balance(&self) -> u64 {
        self.balance
    }

    /// Return the sequence_number field for the given AccountResource
    pub fn sequence_number(&self) -> u64 {
        self.sequence_number
    }

    /// Return the delegated_key_rotation_capability field for the given AccountResource
    pub fn delegated_key_rotation_capability(&self) -> bool {
        self.delegated_key_rotation_capability
    }

    /// Return the delegated_withdrawal_capability field for the given AccountResource
    pub fn delegated_withdrawal_capability(&self) -> bool {
        self.delegated_withdrawal_capability
    }

    /// Return the sent_events field for the given AccountResource
    pub fn sent_events(&self) -> &EventHandle {
        &self.sent_events
    }

    /// Return the received_events field for the given AccountResource
    pub fn received_events(&self) -> &EventHandle {
        &self.received_events
    }
}

#[no_mangle]
pub unsafe extern "C" fn account_resource_free(account_resource: *mut CDevAccountResource) {
    let _: Box<DevAccountResource> = Box::from_raw(account_resource.cast());
}

#[no_mangle]
pub unsafe extern "C" fn account_resource_from_lcs(
    buf: *const u8,
    len: usize,
) -> CDevAccountResource {
    println!("Hello World from rust!");
    let buf: &[u8] = slice::from_raw_parts(buf, len);
    println!("buf: {:?}", buf);

    let account_state_blob: AccountStateBlob = AccountStateBlob::from(buf.to_vec());
    let account_resource = DevAccountResource::new(account_state_blob);
    println!("account_resource: {:?}", account_resource);

    let authentication_key = account_resource.authentication_key.into_inner();
    let mut key_slice = authentication_key.into_boxed_slice();
    let authentication_key_ptr = key_slice.as_mut_ptr();
    std::mem::forget(key_slice);

    let mut sent_key_copy = [0u8; EVENT_KEY_LENGTH];
    sent_key_copy.copy_from_slice(account_resource.sent_events.key().as_bytes());

    let sent_events = CEventHandle {
        count: account_resource.sent_events.count(),
        key: sent_key_copy,
    };

    let mut received_key_copy = [0u8; EVENT_KEY_LENGTH];
    received_key_copy.copy_from_slice(account_resource.received_events.key().as_bytes());

    let received_events = CEventHandle {
        count: account_resource.received_events.count(),
        key: received_key_copy,
    };

    return CDevAccountResource {
        balance: account_resource.balance,
        sequence: account_resource.sequence_number,
        authentication_key: authentication_key_ptr,
        delegated_key_rotation_capability: account_resource.delegated_key_rotation_capability,
        delegated_withdrawal_capability: account_resource.delegated_withdrawal_capability,
        sent_events,
        received_events,
    };
}
