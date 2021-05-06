// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{Error, Result};
use diem_json_rpc_types::views::{AccountStateWithProofView, EventWithProofView};
use diem_types::{
    account_state::AccountState,
    account_state_blob::AccountStateBlob,
    contract_event::{ContractEvent, EventWithProof},
};
use move_core_types::{language_storage::TypeTag, move_resource::{MoveResource, MoveStructType}};
use serde::de::DeserializeOwned;
use std::convert::TryFrom;

/// Wrapper for a deserialized Move event and its containing `ContractEvent`
#[derive(Debug, Clone)]
pub struct Event<T: MoveStructType + DeserializeOwned> {
    /// The deserialized event type
    data: T,
    event: ContractEvent,
}

impl<T: MoveStructType + DeserializeOwned> Event<T> {
    pub fn data(&self) -> &T {
        &self.data
    }

    pub fn event(&self) -> &ContractEvent {
        &self.event
    }
}

/// Deserialize and return the Move events of type `T` in `events`
/// The type `T` must match the specified event types in `events`
pub fn get_events<T: MoveStructType + DeserializeOwned>(
    events: Vec<EventWithProofView>,
) -> Result<Vec<Event<T>>> {
    let events_with_proof: Vec<EventWithProof> = events
        .into_iter()
        .map(|e| {
            bcs::from_bytes::<EventWithProof>(e.event_with_proof.inner()).map_err(Error::decode)
        })
        .collect::<Result<Vec<EventWithProof>>>()?;
    let event_type_tag = TypeTag::Struct(T::struct_tag());
    events_with_proof
        .into_iter()
        .map(|e| {
            // Check that `T` matches the type specified in `e`
            if &event_type_tag != e.event.type_tag() {
                Err(Error::decode(format!(
                    "Type tag of events in stream {:?} does not match type tag of generic type T {:?}", event_type_tag, e.event.type_tag()),
                ))
            } else {
                bcs::from_bytes::<T>(e.event.event_data())
                    .map(|data| Event {
                        data,
                        event: e.event,
                    })
                    .map_err(Error::decode)
            }
        })
        .collect::<Result<Vec<Event<T>>>>()
}

fn get_account_state(
    account_state_with_proof: AccountStateWithProofView,
) -> Result<Option<AccountState>> {
    let account_opt = account_state_with_proof.blob;
    if let Some(account) = account_opt {
        let account_state_blob: AccountStateBlob =
            bcs::from_bytes(account.inner()).map_err(Error::decode)?;
        return Ok(Some(
            AccountState::try_from(&account_state_blob).map_err(Error::decode)?,
        ));
    }
    Ok(None)
}

/// Deserialize and return the Move value of type `T` in `account_state_with_proof`
/// Returns None if no resource of type `T` exists under `address`
pub fn get_resource<T: MoveResource>(
    account_state_with_proof: AccountStateWithProofView,
) -> Result<Option<T>> {
    if let Some(account_state) = get_account_state(account_state_with_proof)? {
        return account_state.get_resource::<T>().map_err(Error::decode);
    }
    Ok(None)
}
