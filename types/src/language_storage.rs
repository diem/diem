// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    access_path::AccessPath,
    account_address::AccountAddress,
    identifier::{IdentStr, Identifier},
};
use anyhow::{Error, Result};
use libra_crypto::hash::{CryptoHash, CryptoHasher, HashValue};
use libra_crypto_derive::CryptoHasher;
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};

#[derive(Serialize, Deserialize, Debug, PartialEq, Hash, Eq, Clone, PartialOrd, Ord)]
pub enum TypeTag {
    Bool,
    U8,
    U64,
    U128,
    ByteArray,
    Address,
    Vector(Box<TypeTag>),
    Struct(StructTag),
}

#[derive(
    Serialize, Deserialize, Debug, PartialEq, Hash, Eq, Clone, PartialOrd, Ord, CryptoHasher,
)]
pub struct StructTag {
    pub address: AccountAddress,
    pub module: Identifier,
    pub name: Identifier,
    pub type_params: Vec<TypeTag>,
}

/// Represents the intitial key into global storage where we first index by the address, and then
/// the struct tag
#[derive(Serialize, Deserialize, Debug, PartialEq, Hash, Eq, Clone, PartialOrd, Ord)]
pub struct ResourceKey {
    address: AccountAddress,
    type_: StructTag,
}

impl ResourceKey {
    pub fn address(&self) -> AccountAddress {
        self.address
    }

    pub fn type_(&self) -> &StructTag {
        &self.type_
    }
}

impl ResourceKey {
    pub fn new(address: AccountAddress, type_: StructTag) -> Self {
        ResourceKey { address, type_ }
    }
}

/// Represents the initial key into global storage where we first index by the address, and then
/// the struct tag
#[derive(
    Serialize, Deserialize, Debug, PartialEq, Hash, Eq, Clone, PartialOrd, Ord, CryptoHasher,
)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
#[cfg_attr(any(test, feature = "fuzzing"), proptest(no_params))]
pub struct ModuleId {
    address: AccountAddress,
    name: Identifier,
}

impl ModuleId {
    pub fn new(address: AccountAddress, name: Identifier) -> Self {
        ModuleId { address, name }
    }

    pub fn name(&self) -> &IdentStr {
        &self.name
    }

    pub fn address(&self) -> &AccountAddress {
        &self.address
    }
}

impl TryFrom<crate::proto::types::ModuleId> for ModuleId {
    type Error = Error;

    fn try_from(proto: crate::proto::types::ModuleId) -> Result<Self> {
        Ok(Self {
            address: proto.address.try_into()?,
            name: Identifier::new(proto.name)?,
        })
    }
}

impl From<ModuleId> for crate::proto::types::ModuleId {
    fn from(txn: ModuleId) -> Self {
        Self {
            address: txn.address.into(),
            name: txn.name.into_string(),
        }
    }
}

impl<'a> From<&'a ModuleId> for AccessPath {
    fn from(module_id: &'a ModuleId) -> Self {
        AccessPath::code_access_path(module_id)
    }
}

impl CryptoHash for ModuleId {
    type Hasher = ModuleIdHasher;

    fn hash(&self) -> HashValue {
        let mut state = Self::Hasher::default();
        state.write(&lcs::to_bytes(self).unwrap());
        state.finish()
    }
}

impl CryptoHash for StructTag {
    type Hasher = StructTagHasher;

    fn hash(&self) -> HashValue {
        let mut state = Self::Hasher::default();
        state.write(&lcs::to_bytes(self).unwrap());
        state.finish()
    }
}
