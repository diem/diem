// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines physical storage schema for account data blob pointed by leaves of state
//! Merkle tree.
//! Account state blob is identified by its hash.
//! ```text
//! |<----key--->|<-------value------->|
//! |    hash    |  account state blob |
//! ```

use crate::schema::ACCOUNT_STATE_CF_NAME;
use crypto::HashValue;
use failure::prelude::*;
use schemadb::{
    define_schema,
    schema::{KeyCodec, ValueCodec},
};
use types::account_state_blob::AccountStateBlob;

define_schema!(
    AccountStateSchema,
    HashValue,
    AccountStateBlob,
    ACCOUNT_STATE_CF_NAME
);

impl KeyCodec<AccountStateSchema> for HashValue {
    fn encode_key(&self) -> Result<Vec<u8>> {
        Ok(self.to_vec())
    }

    fn decode_key(data: &[u8]) -> Result<Self> {
        Ok(HashValue::from_slice(data)?)
    }
}

impl ValueCodec<AccountStateSchema> for AccountStateBlob {
    fn encode_value(&self) -> Result<Vec<u8>> {
        Ok(self.clone().into())
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Ok(data.to_vec().into())
    }
}

#[cfg(test)]
mod test;
