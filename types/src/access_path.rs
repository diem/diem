// Copyright (c) The Diem Core Contributors
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

use crate::account_address::AccountAddress;
use diem_crypto::hash::HashValue;
use move_core_types::language_storage::{ModuleId, ResourceKey, StructTag, CODE_TAG, RESOURCE_TAG};
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, fmt};

#[derive(Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Ord, PartialOrd)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct AccessPath {
    pub address: AccountAddress,
    #[serde(with = "serde_bytes")]
    pub path: Vec<u8>,
}

#[derive(Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Ord, PartialOrd)]
pub enum Path {
    Code(ModuleId),
    Resource(StructTag),
}

impl AccessPath {
    pub fn new(address: AccountAddress, path: Vec<u8>) -> Self {
        AccessPath { address, path }
    }

    pub fn resource_access_vec(tag: StructTag) -> Vec<u8> {
        bcs::to_bytes(&Path::Resource(tag)).expect("Unexpected serialization error")
    }

    /// Convert Accesses into a byte offset which would be used by the storage layer to resolve
    /// where fields are stored.
    pub fn resource_access_path(key: ResourceKey) -> AccessPath {
        let path = AccessPath::resource_access_vec(key.type_);
        AccessPath {
            address: key.address,
            path,
        }
    }

    fn code_access_path_vec(key: ModuleId) -> Vec<u8> {
        bcs::to_bytes(&Path::Code(key)).expect("Unexpected serialization error")
    }

    pub fn code_access_path(key: ModuleId) -> AccessPath {
        let address = *key.address();
        let path = AccessPath::code_access_path_vec(key);
        AccessPath { address, path }
    }

    /// Extract the structured resource or module `Path` from `self`
    pub fn get_path(&self) -> Path {
        bcs::from_bytes::<Path>(&self.path).expect("Unexpected serialization error")
    }

    /// Extract a StructTag from `self`. Returns Some if this is a resource access
    /// path and None otherwise
    pub fn get_struct_tag(&self) -> Option<StructTag> {
        match self.get_path() {
            Path::Resource(s) => Some(s),
            Path::Code(_) => None,
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.path.len() < 1 + HashValue::LENGTH {
            write!(f, "{:?}", self)
        } else {
            write!(f, "AccessPath {{ address: {:x}, ", self.address)?;
            match self.path[0] {
                RESOURCE_TAG => write!(f, "type: Resource, ")?,
                CODE_TAG => write!(f, "type: Module, ")?,
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

impl From<&ModuleId> for AccessPath {
    fn from(id: &ModuleId) -> AccessPath {
        AccessPath {
            address: *id.address(),
            path: id.access_vector(),
        }
    }
}

impl TryFrom<&[u8]> for Path {
    type Error = bcs::Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        bcs::from_bytes::<Path>(bytes)
    }
}

impl TryFrom<&Vec<u8>> for Path {
    type Error = bcs::Error;

    fn try_from(bytes: &Vec<u8>) -> Result<Self, Self::Error> {
        bcs::from_bytes::<Path>(bytes)
    }
}
