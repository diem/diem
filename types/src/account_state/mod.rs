// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_config::{AccountResource, ACCOUNT_RESOURCE_PATH};
use anyhow::{Error, Result};
use serde::export::Formatter;
use serde::{Deserialize, Serialize};
use std::collections::btree_map::BTreeMap;
use std::convert::TryFrom;
use std::fmt;
use std::ops::{Deref, DerefMut};

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
}

impl Deref for AccountState {
    type Target = BTreeMap<Vec<u8>, Vec<u8>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AccountState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl fmt::Debug for AccountState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let account_resource_str = self
            .get_account_resource()
            .map(|account_resource_opt| format!("{:#?}", account_resource_opt))
            .unwrap_or_else(|e| format!("parse error: {:#?}", e));

        write!(f, "AccountResource {{ {} }}", account_resource_str)
    }
}

impl TryFrom<&AccountResource> for AccountState {
    type Error = Error;

    fn try_from(account_resource: &AccountResource) -> Result<Self> {
        let mut btree_map: BTreeMap<Vec<u8>, Vec<u8>> = BTreeMap::new();
        btree_map.insert(
            ACCOUNT_RESOURCE_PATH.to_vec(),
            lcs::to_bytes(account_resource)?,
        );

        Ok(Self(btree_map))
    }
}
