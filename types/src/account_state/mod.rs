// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_config::{
        AccountResource, BalanceResource, ACCOUNT_RECEIVED_EVENT_PATH, ACCOUNT_RESOURCE_PATH,
        ACCOUNT_SENT_EVENT_PATH, BALANCE_RESOURCE_PATH,
    },
    discovery_set::{
        DiscoverySetResource, DISCOVERY_SET_CHANGE_EVENT_PATH, DISCOVERY_SET_RESOURCE_PATH,
    },
    event::EventHandle,
};
use anyhow::{bail, Error, Result};
use serde::{export::Formatter, Deserialize, Serialize};
use std::{collections::btree_map::BTreeMap, convert::TryFrom, fmt};

#[derive(Default, Deserialize, Serialize)]
pub struct AccountState(BTreeMap<Vec<u8>, Vec<u8>>);

impl AccountState {
    pub fn get_account_resource(&self) -> Result<Option<AccountResource>> {
        self.0
            .get(&*ACCOUNT_RESOURCE_PATH)
            .map(|bytes| lcs::from_bytes(bytes))
            .transpose()
            .map_err(Into::into)
    }

    pub fn get_balance_resource(&self) -> Result<Option<BalanceResource>> {
        self.0
            .get(&*BALANCE_RESOURCE_PATH)
            .map(|bytes| lcs::from_bytes(bytes))
            .transpose()
            .map_err(Into::into)
    }

    pub fn get_discovery_set_resource(&self) -> Result<Option<DiscoverySetResource>> {
        self.0
            .get(&*DISCOVERY_SET_RESOURCE_PATH)
            .map(|bytes| lcs::from_bytes(bytes))
            .transpose()
            .map_err(Into::into)
    }

    pub fn get_event_handle_by_query_path(&self, query_path: &[u8]) -> Result<Option<EventHandle>> {
        let event_handle = if *ACCOUNT_RECEIVED_EVENT_PATH == query_path {
            self.get_account_resource()?
                .map(|account_resource| account_resource.received_events().clone())
        } else if *ACCOUNT_SENT_EVENT_PATH == query_path {
            self.get_account_resource()?
                .map(|account_resource| account_resource.sent_events().clone())
        } else if *DISCOVERY_SET_CHANGE_EVENT_PATH == query_path {
            self.get_discovery_set_resource()?
                .map(|discovery_set_resource| discovery_set_resource.change_events().clone())
        } else {
            bail!("Unrecognized query path: {:?}", query_path);
        };

        Ok(event_handle)
    }

    pub fn get(&self, key: &[u8]) -> Option<&Vec<u8>> {
        self.0.get(key)
    }

    pub fn insert(&mut self, key: Vec<u8>, value: Vec<u8>) -> Option<Vec<u8>> {
        self.0.insert(key, value)
    }

    pub fn remove(&mut self, key: &[u8]) -> Option<Vec<u8>> {
        self.0.remove(key)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Debug for AccountState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let account_resource_str = self
            .get_account_resource()
            .map(|account_resource_opt| format!("{:#?}", account_resource_opt))
            .unwrap_or_else(|e| format!("parse error: {:#?}", e));
        // TODO: add support for other types of resources

        write!(f, "AccountResource {{ {} }}", account_resource_str)
    }
}

impl TryFrom<(&AccountResource, &BalanceResource)> for AccountState {
    type Error = Error;

    fn try_from(
        (account_resource, balance_resource): (&AccountResource, &BalanceResource),
    ) -> Result<Self> {
        let mut btree_map: BTreeMap<Vec<u8>, Vec<u8>> = BTreeMap::new();
        btree_map.insert(
            ACCOUNT_RESOURCE_PATH.to_vec(),
            lcs::to_bytes(account_resource)?,
        );
        btree_map.insert(
            BALANCE_RESOURCE_PATH.to_vec(),
            lcs::to_bytes(balance_resource)?,
        );

        Ok(Self(btree_map))
    }
}
