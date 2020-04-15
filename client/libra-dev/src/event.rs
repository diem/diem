// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    data::{LibraEvent, LibraEventType, LibraPaymentEvent, LibraStatus},
    error::*,
};
use libra_types::{
    account_address::AccountAddress,
    account_config::{ReceivedPaymentEvent, SentPaymentEvent},
    contract_event::ContractEvent,
    event::EventKey,
    language_storage::TypeTag,
};
use std::{convert::TryFrom, ffi::CString, ops::Deref, slice};

const MAX_BUFFER_LENGTH: usize = 255;

#[no_mangle]
pub unsafe extern "C" fn libra_LibraEvent_from(
    buf_key: *const u8,
    len_key: usize,
    buf_data: *const u8,
    len_data: usize,
    buf_type_tag: *const u8,
    len_type_tag: usize,
    out: *mut *mut LibraEvent,
) -> LibraStatus {
    clear_error();
    if buf_key.is_null() {
        update_last_error("buf_key parameter must not be null.".to_string());
        return LibraStatus::InvalidArgument;
    }
    if buf_data.is_null() {
        update_last_error("buf_data parameter must not be null.".to_string());
        return LibraStatus::InvalidArgument;
    }
    if buf_type_tag.is_null() {
        update_last_error("buf_type_tag parameter must not be null.".to_string());
        return LibraStatus::InvalidArgument;
    }

    let buffer_key: &[u8] = slice::from_raw_parts(buf_key, len_key);
    let buffer_data: &[u8] = slice::from_raw_parts(buf_data, len_data);
    let buffer_type_tag: &[u8] = slice::from_raw_parts(buf_type_tag, len_type_tag);

    let mut key = [0u8; EventKey::LENGTH];
    key.copy_from_slice(buffer_key);

    let (_salt, event_address) = buffer_key.split_at(8);
    let mut key_address = [0u8; AccountAddress::LENGTH];
    key_address.copy_from_slice(event_address);

    let type_tag: TypeTag = match lcs::from_bytes(buffer_type_tag) {
        Ok(result) => result,
        Err(e) => {
            update_last_error(format!("Error deserializing type tag: {}", e.to_string()));
            return LibraStatus::InvalidArgument;
        }
    };

    let mut module = [0u8; MAX_BUFFER_LENGTH];
    let mut name = [0u8; MAX_BUFFER_LENGTH];

    if let TypeTag::Struct(struct_tag) = type_tag.clone() {
        let module_tmp = match CString::new(struct_tag.module.into_string()) {
            Ok(res) => res,
            Err(e) => {
                update_last_error(format!(
                    "Error converting module to string: {}",
                    e.to_string()
                ));
                return LibraStatus::InvalidArgument;
            }
        };
        let module_bytes = module_tmp.as_bytes_with_nul();
        let module_len = module_bytes.len();
        module[..module_len].copy_from_slice(module_bytes);

        let name_tmp = match CString::new(struct_tag.name.into_string()) {
            Ok(res) => res,
            Err(e) => {
                update_last_error(format!(
                    "Error converting struct tag name to string: {}",
                    e.to_string()
                ));
                return LibraStatus::InvalidArgument;
            }
        };
        let name_bytes = name_tmp.as_bytes_with_nul();
        let name_len = name_bytes.len();
        name[..name_len].copy_from_slice(name_bytes);
    };

    let account_event = ContractEvent::new(EventKey::new(key), 0, type_tag, buffer_data.to_vec());

    let mut event_enum = LibraEventType::UndefinedEvent;
    let mut event_data: Option<LibraPaymentEvent> = None;

    match SentPaymentEvent::try_from(&account_event) {
        Ok(res) => {
            event_enum = LibraEventType::SentPaymentEvent;

            let metadata_len = (*res.metadata()).len();
            let metadata_ptr = if metadata_len > 0 {
                let metadata = res.metadata().clone().into_boxed_slice();
                (*Box::into_raw(metadata)).as_mut_ptr()
            } else {
                std::ptr::null_mut()
            };

            event_data = Some(LibraPaymentEvent {
                sender_address: key_address,
                receiver_address: res.receiver().into(),
                amount: res.amount(),
                metadata: metadata_ptr,
                metadata_len,
            });
        }
        Err(e) => update_last_error(format!(
            "Error deserializing SentPaymentEvent: {}",
            e.to_string()
        )),
    };

    match ReceivedPaymentEvent::try_from(&account_event) {
        Ok(res) => {
            event_enum = LibraEventType::ReceivedPaymentEvent;

            let metadata_len = (*res.metadata()).len();
            let metadata_ptr = if metadata_len > 0 {
                let metadata = res.metadata().clone().into_boxed_slice();
                (*Box::into_raw(metadata)).as_mut_ptr()
            } else {
                std::ptr::null_mut()
            };

            event_data = Some(LibraPaymentEvent {
                sender_address: res.sender().into(),
                receiver_address: key_address,
                amount: res.amount(),
                metadata: metadata_ptr,
                metadata_len,
            });
        }
        Err(e) => update_last_error(format!(
            "Error deserializing ReceivedPaymentEvent: {}",
            e.to_string()
        )),
    };

    let result = Box::new(LibraEvent {
        event_type: event_enum,
        module,
        name,
        payment_event_data: event_data.unwrap_or_default(),
    });

    *out = Box::into_raw(result);

    LibraStatus::Ok
}

#[no_mangle]
pub unsafe extern "C" fn libra_LibraEvent_free(ptr: *mut LibraEvent) {
    let _to_drop = Box::from_raw(ptr);

    let metadata = _to_drop.deref().payment_event_data.metadata;
    let metadata_len = _to_drop.deref().payment_event_data.metadata_len;
    if !metadata.is_null() {
        let _to_drop = Box::from_raw(slice::from_raw_parts_mut(metadata, metadata_len));
    }
}

#[test]
fn test_libra_LibraEvent_from() {
    use libra_crypto::{ed25519, TPrivateKey, Uniform};
    use libra_types::{
        account_address::AccountAddress,
        account_config::SentPaymentEvent,
        contract_event::ContractEvent,
        event::{EventHandle, EventKey},
        language_storage::{StructTag, TypeTag::Struct},
    };
    use move_core_types::identifier::Identifier;
    use std::ffi::CStr;

    let public_key = ed25519::SigningKey::generate_for_testing().public_key();
    let sender_address = AccountAddress::from_public_key(&public_key);
    let sent_event_handle = EventHandle::new(EventKey::new_from_address(&sender_address, 0), 0);
    let sequence_number = sent_event_handle.count();
    let event_key = sent_event_handle.key();
    let module = "LibraAccount";
    let name = "SentPaymentEvent";

    let type_tag = Struct(StructTag {
        address: AccountAddress::new([0; AccountAddress::LENGTH]),
        module: Identifier::new(module).unwrap(),
        name: Identifier::new(name).unwrap(),
        type_params: [].to_vec(),
    });
    let amount = 50_000_000;
    let receiver_address = AccountAddress::random();
    let event_data = SentPaymentEvent::new(amount, receiver_address, vec![]);
    let event_data_bytes = lcs::to_bytes(&event_data).unwrap();

    let event = ContractEvent::new(*event_key, sequence_number, type_tag, event_data_bytes);

    let proto_txn = libra_types::proto::types::Event::from(event);

    let mut libra_event: *mut LibraEvent = std::ptr::null_mut();
    let result = unsafe {
        libra_LibraEvent_from(
            proto_txn.key.as_ptr(),
            proto_txn.key.len(),
            proto_txn.event_data.as_ptr(),
            proto_txn.event_data.len(),
            proto_txn.type_tag.as_ptr(),
            proto_txn.type_tag.len(),
            &mut libra_event,
        )
    };
    assert_eq!(result, LibraStatus::Ok);

    unsafe {
        let module_c_string: &CStr = CStr::from_ptr((*libra_event).module.as_ptr() as *const i8);
        let module_str_slice: &str = module_c_string.to_str().unwrap();
        assert_eq!(module_str_slice, module);

        let name_c_string: &CStr = CStr::from_ptr((*libra_event).name.as_ptr() as *const i8);
        let name_str_slice: &str = name_c_string.to_str().unwrap();
        assert_eq!(name_str_slice, name);

        assert_eq!((*libra_event).event_type, LibraEventType::SentPaymentEvent);

        assert_eq!(
            sender_address,
            AccountAddress::new((*libra_event).payment_event_data.sender_address)
        );
        assert_eq!(
            receiver_address,
            AccountAddress::new((*libra_event).payment_event_data.receiver_address),
        );
        assert_eq!(amount, (*libra_event).payment_event_data.amount);
    }

    unsafe {
        libra_LibraEvent_free(libra_event);
    }
}
