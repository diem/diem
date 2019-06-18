// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Suppose we have the following data structure in a smart contract:
//!
//! struct B {
//!   Map<String, String> mymap;
//! }
//!
//! struct A {
//!   B b;
//!   int my_int;
//! }
//!
//! struct C {
//!   List<int> mylist;
//! }
//!
//! A a;
//! C c;
//!
//! and the data belongs to Alice. Then an access to `a.b.mymap` would be translated to an access
//! to an entry in key-value store whose key is `<Alice>/a/b/mymap`. In the same way, the access to
//! `c.mylist` would need to query `<Alice>/c/mylist`.
//!
//! So an account stores its data in a directory structure, for example:
//!   <Alice>/balance:   10
//!   <Alice>/a/b/mymap: {"Bob" => "abcd", "Carol" => "efgh"}
//!   <Alice>/a/myint:   20
//!   <Alice>/c/mylist:  [3, 5, 7, 9]
//!
//! If someone needs to query the map above and find out what value associated with "Bob" is,
//! `address` will be set to Alice and `path` will be set to "/a/b/mymap/Bob".
//!
//! On the other hand, if you want to query only <Alice>/a/*, `address` will be set to Alice and
//! `path` will be set to "/a" and use the `get_prefix()` method from statedb

// This is caused by deriving Arbitrary for AccessPath.
#![allow(clippy::unit_arg)]

use crate::{
    account_address::AccountAddress,
    account_config::{
        account_received_event_path, account_resource_path, account_sent_event_path,
        association_address,
    },
    language_storage::{CodeKey, ResourceKey, StructTag},
    validator_set::validator_set_path,
};
use canonical_serialization::{
    CanonicalDeserialize, CanonicalDeserializer, CanonicalSerialize, CanonicalSerializer,
};
use crypto::hash::{CryptoHash, HashValue};
use failure::prelude::*;
use hex;
use lazy_static::lazy_static;
use proptest_derive::Arbitrary;
use proto_conv::{FromProto, IntoProto};
use radix_trie::TrieKey;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Formatter},
    slice::Iter,
    str::{self, FromStr},
};

#[derive(Default, Serialize, Deserialize, Debug, PartialEq, Hash, Eq, Clone, Ord, PartialOrd)]
pub struct Field(String);

impl Field {
    pub fn new(s: &str) -> Field {
        Field(s.to_string())
    }

    pub fn name(&self) -> &String {
        &self.0
    }
}

impl From<String> for Field {
    fn from(s: String) -> Self {
        Field(s)
    }
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Eq, Hash, Serialize, Deserialize, Debug, Clone, PartialEq, Ord, PartialOrd)]
pub enum Access {
    Field(Field),
    Index(u64),
}

impl Access {
    pub fn new(s: &str) -> Self {
        Access::Field(Field::new(s))
    }
}

impl FromStr for Access {
    type Err = ::std::num::ParseIntError;

    fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
        if let Ok(idx) = s.parse::<u64>() {
            Ok(Access::Index(idx))
        } else {
            Ok(Access::Field(Field::new(s)))
        }
    }
}

impl fmt::Display for Access {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Access::Field(field) => write!(f, "\"{}\"", field),
            Access::Index(i) => write!(f, "{}", i),
        }
    }
}

/// Non-empty sequence of field accesses
#[derive(Eq, Hash, Serialize, Deserialize, Debug, Clone, PartialEq, Ord, PartialOrd)]
pub struct Accesses(Vec<Access>);

/// SEPARATOR is used as a delimiter between fields. It should not be a legal part of any identifier
/// in the language
const SEPARATOR: char = '/';

impl Accesses {
    pub fn empty() -> Self {
        Accesses(vec![])
    }

    pub fn new(field: Field) -> Self {
        Accesses(vec![Access::Field(field)])
    }

    /// Add a field to the end of the sequence
    pub fn add_field_to_back(&mut self, field: Field) {
        self.0.push(Access::Field(field))
    }

    /// Add an index to the end of the sequence
    pub fn add_index_to_back(&mut self, idx: u64) {
        self.0.push(Access::Index(idx))
    }

    pub fn append(&mut self, accesses: &mut Accesses) {
        self.0.append(&mut accesses.0)
    }

    /// Returns the first field in the sequence and reference to the remaining fields
    pub fn split_first(&self) -> (&Access, &[Access]) {
        self.0.split_first().unwrap()
    }

    /// Return the last access in the sequence
    pub fn last(&self) -> &Access {
        self.0.last().unwrap() // guaranteed not to fail because sequence is non-empty
    }

    pub fn iter(&self) -> Iter<'_, Access> {
        self.0.iter()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn as_separated_string(&self) -> String {
        let mut path = String::new();
        for access in self.0.iter() {
            match access {
                Access::Field(s) => {
                    let access_str = s.name().as_ref();
                    assert!(access_str != "");
                    path.push_str(access_str)
                }
                Access::Index(i) => path.push_str(i.to_string().as_ref()),
            };
            path.push(SEPARATOR);
        }
        path
    }

    pub fn take_nth(&self, new_len: usize) -> Accesses {
        assert!(self.0.len() >= new_len);
        Accesses(self.0.clone().into_iter().take(new_len).collect())
    }
}

impl<'a> IntoIterator for &'a Accesses {
    type Item = &'a Access;
    type IntoIter = Iter<'a, Access>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl From<Vec<Access>> for Accesses {
    fn from(accesses: Vec<Access>) -> Accesses {
        Accesses(accesses)
    }
}

impl From<Vec<u8>> for Accesses {
    fn from(mut raw_bytes: Vec<u8>) -> Accesses {
        let access_str = String::from_utf8(raw_bytes.split_off(HashValue::LENGTH + 1)).unwrap();
        let fields_str = access_str.split(SEPARATOR).collect::<Vec<&str>>();
        let mut accesses = vec![];
        for access_str in fields_str.into_iter() {
            if access_str != "" {
                accesses.push(Access::from_str(access_str).unwrap());
            }
        }
        Accesses::from(accesses)
    }
}

impl TrieKey for Accesses {
    fn encode_bytes(&self) -> Vec<u8> {
        self.as_separated_string().into_bytes()
    }
}

lazy_static! {
    /// The access path where the Validator Set resource is stored.
    pub static ref VALIDATOR_SET_ACCESS_PATH: AccessPath =
        AccessPath::new(association_address(), validator_set_path());
}

#[derive(
    Clone,
    Eq,
    PartialEq,
    Default,
    Hash,
    Serialize,
    Deserialize,
    Ord,
    PartialOrd,
    Arbitrary,
    FromProto,
    IntoProto,
)]
#[ProtoType(crate::proto::access_path::AccessPath)]
pub struct AccessPath {
    pub address: AccountAddress,
    pub path: Vec<u8>,
}

impl AccessPath {
    const CODE_TAG: u8 = 0;
    const RESOURCE_TAG: u8 = 1;

    pub fn new(address: AccountAddress, path: Vec<u8>) -> Self {
        AccessPath { address, path }
    }

    /// Given an address, returns the corresponding access path that stores the Account resource.
    pub fn new_for_account(address: AccountAddress) -> Self {
        Self::new(address, account_resource_path())
    }

    /// Create an AccessPath for a ContractEvent.
    /// That is an AccessPah that uniquely identifies a given event for a published resource.
    pub fn new_for_event(address: AccountAddress, root: &[u8], key: &[u8]) -> Self {
        let mut path: Vec<u8> = Vec::new();
        path.extend_from_slice(root);
        path.push(b'/');
        path.extend_from_slice(key);
        path.push(b'/');
        Self::new(address, path)
    }

    /// Create an AccessPath to the event for the sender account in a deposit operation.
    /// The sent counter in LibraAccount.T (LibraAccount.T.sent_events_count) is used to generate
    /// the AccessPath.
    /// That AccessPath can be used as a key into the event storage to retrieve all sent
    /// events for a given account.
    pub fn new_for_sent_event(address: AccountAddress) -> Self {
        Self::new(address, account_sent_event_path())
    }

    /// Create an AccessPath to the event for the target account (the receiver)
    /// in a deposit operation.
    /// The received counter in LibraAccount.T (LibraAccount.T.received_events_count) is used to
    /// generate the AccessPath.
    /// That AccessPath can be used as a key into the event storage to retrieve all received
    /// events for a given account.
    pub fn new_for_received_event(address: AccountAddress) -> Self {
        Self::new(address, account_received_event_path())
    }

    pub fn resource_access_vec(tag: &StructTag, accesses: &Accesses) -> Vec<u8> {
        let mut key = vec![];
        key.push(Self::RESOURCE_TAG);

        key.append(&mut tag.hash().to_vec());

        // We don't need accesses in production right now. Accesses are appended here just for
        // passing the old tests.
        key.append(&mut accesses.as_separated_string().into_bytes());
        key
    }

    /// Convert Accesses into a byte offset which would be used by the storage layer to resolve
    /// where fields are stored.
    pub fn resource_access_path(key: &ResourceKey, accesses: &Accesses) -> AccessPath {
        let path = AccessPath::resource_access_vec(&key.type_(), accesses);
        AccessPath {
            address: key.address().to_owned(),
            path,
        }
    }

    fn code_access_path_vec(key: &CodeKey) -> Vec<u8> {
        let mut root = vec![];
        root.push(Self::CODE_TAG);
        root.append(&mut key.hash().to_vec());
        root
    }

    pub fn code_access_path(key: &CodeKey) -> AccessPath {
        let path = AccessPath::code_access_path_vec(key);
        AccessPath {
            address: *key.address(),
            path,
        }
    }
}

impl fmt::Debug for AccessPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AccessPath {{ address: {:x}, path: {} }}",
            self.address,
            hex::encode(&self.path)
        )
    }
}

impl fmt::Display for AccessPath {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if self.path.len() < 1 + HashValue::LENGTH {
            write!(f, "{:?}", self)
        } else {
            write!(f, "AccessPath {{ address: {:x}, ", self.address)?;
            match self.path[0] {
                Self::RESOURCE_TAG => write!(f, "type: Resource, ")?,
                Self::CODE_TAG => write!(f, "type: Module, ")?,
                tag => write!(f, "type: {:?}, ", tag)?,
            };
            write!(
                f,
                "hash: {:?}, ",
                hex::encode(&self.path[1..=HashValue::LENGTH])
            )?;
            write!(
                f,
                "suffix: {:?} }} ",
                String::from_utf8_lossy(&self.path[1 + HashValue::LENGTH..])
            )
        }
    }
}

impl CanonicalSerialize for AccessPath {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer
            .encode_struct(&self.address)?
            .encode_variable_length_bytes(&self.path)?;
        Ok(())
    }
}

impl CanonicalDeserialize for AccessPath {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self> {
        let address = deserializer.decode_struct::<AccountAddress>()?;
        let path = deserializer.decode_variable_length_bytes()?;

        Ok(Self { address, path })
    }
}
