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

use crate::account_address::AccountAddress;
use libra_crypto::hash::HashValue;
use move_core_types::language_storage::{ModuleId, ResourceKey, StructTag, CODE_TAG, RESOURCE_TAG};
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Ord, PartialOrd)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct AccessPath {
    pub address: AccountAddress,
    pub path: Vec<u8>,
}

impl AccessPath {
    pub const CODE_TAG: u8 = 0;
    pub const RESOURCE_TAG: u8 = 1;

    pub fn new(address: AccountAddress, path: Vec<u8>) -> Self {
        AccessPath { address, path }
    }

    pub fn resource_access_vec(tag: &StructTag) -> Vec<u8> {
        tag.access_vector()
    }

    /// Convert Accesses into a byte offset which would be used by the storage layer to resolve
    /// where fields are stored.
    pub fn resource_access_path(key: &ResourceKey) -> AccessPath {
        let path = AccessPath::resource_access_vec(&key.type_());
        AccessPath {
            address: key.address().to_owned(),
            path,
        }
    }

    fn code_access_path_vec(key: &ModuleId) -> Vec<u8> {
        key.access_vector()
    }

    pub fn code_access_path(key: &ModuleId) -> AccessPath {
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
