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

use crate::{
    account_address::AccountAddress,
    account_config::{
        account_resource_path, association_address, ACCOUNT_RECEIVED_EVENT_PATH,
        ACCOUNT_SENT_EVENT_PATH,
    },
    identifier::{IdentStr, Identifier},
    language_storage::{ModuleId, ResourceKey, StructTag},
    validator_set::validator_set_path,
};
use canonical_serialization::{CanonicalDeserialize, CanonicalDeserializer, CanonicalSerialize, CanonicalSerializer, SimpleDeserializer, SimpleSerializer};
use crypto::hash::{CryptoHash, HashValue};
use failure::prelude::*;
use hex;
use lazy_static::lazy_static;
#[cfg(any(test, feature = "testing"))]
use proptest_derive::Arbitrary;
use proto_conv::{FromProto, IntoProto};
use radix_trie::TrieKey;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Formatter},
    slice::Iter,
};
use crate::account_config::account_struct_tag;
use std::convert::TryFrom;
use std::ops::Index;
use crate::account_address::ADDRESS_LENGTH;

#[derive(Serialize, Deserialize, Debug, PartialEq, Hash, Eq, Clone, Ord, PartialOrd)]
pub struct Field(Identifier);

impl Field {
    pub fn new(name: Identifier) -> Field {
        Field(name)
    }

    pub fn name(&self) -> &IdentStr {
        &self.0
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
    pub fn new(name: Identifier) -> Self {
        Access::Field(Field::new(name))
    }

    pub fn new_with_index(idx: u64) -> Self {
        Access::Index(idx)
    }

    pub fn index(&self) -> Option<u64> {
        match self{
            Access::Index(val) => Some(*val),
            _ => None
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

    pub fn new_with_access(access:Vec<Access>) -> Self{
        Accesses(access)
    }

    pub fn new_with_index(idx: u64) -> Self{
        Accesses(vec![Access::Index(idx)])
    }

    pub fn from_separated_string(value: &str) -> Result<Self>{
        let result:Result<Vec<Access>> = value.split(SEPARATOR).into_iter().filter(|s|!s.is_empty()).map(|access|match access.parse::<u64>(){
            Ok(idx) => Ok(Access::Index(idx)),
            Err(_) => Ok(Access::Field(Field::new(Identifier::new(access)?)))
        }).collect();
        Ok(Accesses(result?))
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

    pub fn first(&self) -> &Access {
        self.0.first().unwrap()
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
                    let access_str = s.name().as_str();
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

    pub fn range(&self, from:usize, to:usize) -> Accesses {
        Accesses(self.0[from..to].to_vec())
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.as_separated_string().into_bytes()
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

impl TrieKey for Accesses {
    fn encode_bytes(&self) -> Vec<u8> {
        self.as_separated_string().into_bytes()
    }
}

impl Index<usize> for Accesses {

    type Output = Access;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

lazy_static! {
    /// The access path where the Validator Set resource is stored.
    pub static ref VALIDATOR_SET_ACCESS_PATH: AccessPath =
        AccessPath::new(association_address(), validator_set_path());
}


#[derive(Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Ord, PartialOrd, Debug)]
pub enum DataPath {
    Code { module_id: ModuleId },
    Resource { tag: StructTag },
    ChannelResource { participant: AccountAddress, tag: StructTag },
}

impl DataPath {
    //TODO get index by enum
    pub const CODE_TAG: u8 = 0;
    pub const RESOURCE_TAG: u8 = 1;
    pub const CHANNEL_RESOURCE_TAG: u8 =2;

    pub fn from(path: &[u8]) -> Result<Self> {
        match path[0]{
            DataPath::CODE_TAG => {
                Ok(DataPath::Code {module_id: SimpleDeserializer::deserialize(&path[1..])?})
            },
            DataPath::RESOURCE_TAG => {
                Ok(DataPath::Resource {tag: SimpleDeserializer::deserialize(&path[1..])?})
            },
            DataPath::CHANNEL_RESOURCE_TAG => {
                ensure!(
                    path.len() > ADDRESS_LENGTH +1,
                    "The path {:?} is of invalid length",
                    path
                );
                let address_bytes = &path[1..(1+ADDRESS_LENGTH)];
                let tag_bytes = &path[(1+ADDRESS_LENGTH)..];
                Ok(DataPath::ChannelResource { participant: AccountAddress::try_from(address_bytes)?, tag: SimpleDeserializer::deserialize(tag_bytes)?})
            }
            _ => bail!("invalid access path.")
        }
    }

    pub fn account_resource_data_path() -> Self {
        Self::onchain_resource_path(account_struct_tag())
    }

    pub fn code_data_path(module_id: ModuleId) -> Self {
        DataPath::Code { module_id }
    }

    pub fn onchain_resource_path(tag: StructTag) -> Self{
        DataPath::Resource {
            tag,
        }
    }

    pub fn channel_resource_path(participant: AccountAddress, tag: StructTag) -> Self{
        DataPath::ChannelResource {
            participant,
            tag,
        }
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.into()
    }

    pub fn is_code(&self) -> bool {
        match self {
            DataPath::Code {..} => true,
            _ => false,
        }
    }

    pub fn is_onchain_resource(&self) -> bool {
        match self {
            DataPath::Resource {..} => true,
            _ => false,
        }
    }

    pub fn is_channel_resource(&self) -> bool {
        match self {
            DataPath::ChannelResource {..} => true,
            _ => false,
        }
    }

    pub fn resource_tag(&self) -> Option<&StructTag> {
        match self{
            DataPath::Resource {tag} => Some(tag),
            DataPath::ChannelResource {participant:_,tag} => Some(tag),
            _ => None,
        }
    }

    pub fn participant(&self) -> Option<AccountAddress> {
        match self{
            DataPath::ChannelResource {participant,tag:_} => Some(*participant),
            _ => None
        }
    }
}

impl From<&DataPath> for Vec<u8> {
    fn from(path: &DataPath) -> Self {
        match path {
            DataPath::Code { module_id } => {
                let mut key = vec![];
                key.push(DataPath::CODE_TAG);
                key.append(&mut SimpleSerializer::serialize(module_id).unwrap());
                key
            },
            DataPath::Resource { tag } => {
                let mut key = vec![];
                key.push(DataPath::RESOURCE_TAG);
                key.append(&mut SimpleSerializer::serialize(tag).unwrap());
                key
            },
            DataPath::ChannelResource {participant, tag} => {
                let mut key = vec![];
                key.push(DataPath::CHANNEL_RESOURCE_TAG);
                key.append(&mut participant.to_vec());
                key.append(&mut SimpleSerializer::serialize(tag).unwrap());
                key
            }
        }
    }
}

impl From<DataPath> for Vec<u8> {
    fn from(path: DataPath) -> Self {
        Self::from(&path)
    }
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
    FromProto,
    IntoProto,
)]
#[cfg_attr(any(test, feature = "testing"), derive(Arbitrary))]
#[ProtoType(crate::proto::access_path::AccessPath)]
pub struct AccessPath {
    pub address: AccountAddress,
    pub path: Vec<u8>,
}

impl AccessPath {
    //const CODE_TAG: u8 = 0;
    //const RESOURCE_TAG: u8 = 1;

    pub fn new(address: AccountAddress, path: Vec<u8>) -> Self {
        AccessPath { address, path }
    }

    /// Given an address, returns the corresponding access path that stores the Account resource.
    pub fn new_for_account(address: AccountAddress) -> Self {
        Self::new_for_account_resource(address)
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
        Self::new(address, ACCOUNT_SENT_EVENT_PATH.to_vec())
    }

    /// Create an AccessPath to the event for the target account (the receiver)
    /// in a deposit operation.
    /// The received counter in LibraAccount.T (LibraAccount.T.received_events_count) is used to
    /// generate the AccessPath.
    /// That AccessPath can be used as a key into the event storage to retrieve all received
    /// events for a given account.
    pub fn new_for_received_event(address: AccountAddress) -> Self {
        Self::new(address, ACCOUNT_RECEIVED_EVENT_PATH.to_vec())
    }

    pub fn resource_access_vec(tag: &StructTag, accesses: &Accesses) -> Vec<u8> {

        let mut key:Vec<u8> = DataPath::onchain_resource_path(tag.clone()).into();

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

    pub fn new_for_data_path(address: AccountAddress, path: DataPath) -> Self {
        AccessPath { address, path: path.to_vec() }
    }

    pub fn new_for_account_resource(address: AccountAddress) -> Self {
        Self::new_for_data_path(address, DataPath::account_resource_data_path() )
    }

    pub fn code_access_path(key: &ModuleId) -> AccessPath {
        Self::new_for_data_path(*key.address(), DataPath::code_data_path(key.clone()))
    }

    pub fn onchain_resource_access_path(key: &ResourceKey) -> AccessPath {
        Self::new_for_data_path(key.address(), DataPath::onchain_resource_path(key.type_().clone()))
    }

    pub fn channel_resource_access_path(account: AccountAddress, participant: AccountAddress, tag: StructTag) -> AccessPath {
        Self::new_for_data_path(account, DataPath::channel_resource_path(participant, tag))
    }

    pub fn is_code(&self) -> bool {
        !self.path.is_empty() && self.path[0] == DataPath::CODE_TAG
    }

    pub fn is_onchain_resource(&self) -> bool {
        !self.path.is_empty() && self.path[0] == DataPath::RESOURCE_TAG
    }

    pub fn is_channel_resource(&self) -> bool {
        !self.path.is_empty() && self.path[0] == DataPath::CHANNEL_RESOURCE_TAG
    }

    pub fn data_path(&self) -> Option<DataPath> {
        if self.path.is_empty() {
            return None;
        }
        DataPath::from(self.path.as_slice()).ok()
    }

    pub fn resource_tag(&self) -> Option<StructTag>{
        if self.path.is_empty() {
            return None;
        }
        match DataPath::from(self.path.as_slice()).ok(){
            None => None,
            Some(data_path) => data_path.resource_tag().cloned()
        }
    }
}

impl fmt::Debug for AccessPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AccessPath {{ address: {:x}, path: {} data_path: {:?}}}",
            self.address,
            hex::encode(&self.path),
            self.data_path()
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
                DataPath::RESOURCE_TAG => write!(f, "type: OnChain Resource, ")?,
                DataPath::CHANNEL_RESOURCE_TAG => write!(f, "type: Channel Resource, ")?,
                DataPath::CODE_TAG => write!(f, "type: Module, ")?,
                tag => write!(f, "type: {:?}, ", tag)?,
            };
            write!(
                f,
                "hash: {:?}, ",
                hex::encode(&self.path[1..=HashValue::LENGTH])
            )?;
            write!(
                f,
                "data_path: {:?}",
                self.data_path()
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
            .encode_bytes(&self.path)?;
        Ok(())
    }
}

impl CanonicalDeserialize for AccessPath {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self> {
        let address = deserializer.decode_struct::<AccountAddress>()?;
        let path = deserializer.decode_bytes()?;

        Ok(Self { address, path })
    }
}
