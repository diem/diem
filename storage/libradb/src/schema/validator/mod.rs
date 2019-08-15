// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines physical storage schema for validator sets
//! Among all versions at some of them we change the validator set, at which
//! point we call it a new epoch. Each validator identified by their `public_key`
//! that's part of the epoch starting at `version` is stored in a row.
//! ```text
//! |<---------key-------->|<-value->|
//! | version | public_key |  null   |
//! ```

use crate::schema::{ensure_slice_len_eq, VALIDATOR_CF_NAME};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use core::convert::TryFrom;
use failure::prelude::*;
use nextgen_crypto::ed25519::{Ed25519PublicKey, ED25519_PUBLIC_KEY_LENGTH};
use schemadb::{
    define_schema,
    schema::{KeyCodec, ValueCodec},
};
use std::{io::Write, mem::size_of};
use types::transaction::Version;

define_schema!(ValidatorSchema, Key, (), VALIDATOR_CF_NAME);

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Key {
    /// version at which epoch starts
    pub(crate) version: Version,
    /// public_key of validator
    pub(crate) public_key: Ed25519PublicKey,
}

impl KeyCodec<ValidatorSchema> for Key {
    fn encode_key(&self) -> Result<Vec<u8>> {
        let public_key_serialized = self.public_key.to_bytes();
        let mut encoded_key =
            Vec::with_capacity(size_of::<Version>() + ED25519_PUBLIC_KEY_LENGTH * size_of::<u8>());
        encoded_key.write_u64::<BigEndian>(self.version)?;
        encoded_key.write_all(&public_key_serialized)?;
        Ok(encoded_key)
    }

    fn decode_key(data: &[u8]) -> Result<Self> {
        let version = (&data[..size_of::<u64>()]).read_u64::<BigEndian>()?;
        let public_key = Ed25519PublicKey::try_from(&data[size_of::<u64>()..])?;
        Ok(Key {
            version,
            public_key,
        })
    }
}

impl ValueCodec<ValidatorSchema> for () {
    fn encode_value(&self) -> Result<Vec<u8>> {
        Ok(vec![])
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        ensure_slice_len_eq(data, 0)?;
        Ok(())
    }
}

#[cfg(test)]
mod test;
