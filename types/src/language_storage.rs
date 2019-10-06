// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    access_path::AccessPath,
    account_address::AccountAddress,
    identifier::{IdentStr, Identifier},
};
use canonical_serialization::{
    CanonicalDeserialize, CanonicalDeserializer, CanonicalSerialize, CanonicalSerializer,
    SimpleSerializer,
};
use crypto::hash::{AccessPathHasher, CryptoHash, CryptoHasher, HashValue};
use failure::Result;
#[cfg(any(test, feature = "testing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};

#[derive(Serialize, Deserialize, Debug, PartialEq, Hash, Eq, Clone, PartialOrd, Ord)]
pub struct StructTag {
    pub address: AccountAddress,
    pub module: Identifier,
    pub name: Identifier,
    pub type_params: Vec<StructTag>,
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
#[derive(Serialize, Deserialize, Debug, PartialEq, Hash, Eq, Clone, PartialOrd, Ord)]
#[cfg_attr(any(test, feature = "testing"), derive(Arbitrary))]
#[cfg_attr(any(test, feature = "testing"), proptest(no_params))]
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
    type Error = failure::Error;

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

impl CanonicalSerialize for ModuleId {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer
            .encode_struct(&self.address)?
            .encode_struct(&self.name)?;
        Ok(())
    }
}

impl CanonicalDeserialize for ModuleId {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self> {
        let address = deserializer.decode_struct::<AccountAddress>()?;
        let name = deserializer.decode_struct::<Identifier>()?;

        Ok(Self { address, name })
    }
}

impl CryptoHash for ModuleId {
    type Hasher = AccessPathHasher;

    fn hash(&self) -> HashValue {
        let mut state = Self::Hasher::default();
        state.write(&SimpleSerializer::<Vec<u8>>::serialize(self).unwrap());
        state.finish()
    }
}

impl CanonicalSerialize for StructTag {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer
            .encode_struct(&self.address)?
            .encode_struct(&self.module)?
            .encode_struct(&self.name)?
            .encode_vec(&self.type_params)?;
        Ok(())
    }
}

impl CanonicalDeserialize for StructTag {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self> {
        let address = deserializer.decode_struct::<AccountAddress>()?;
        let module = deserializer.decode_struct::<Identifier>()?;
        let name = deserializer.decode_struct::<Identifier>()?;
        let type_params = deserializer.decode_vec::<StructTag>()?;
        Ok(Self {
            address,
            name,
            module,
            type_params,
        })
    }
}

impl CryptoHash for StructTag {
    type Hasher = AccessPathHasher;

    fn hash(&self) -> HashValue {
        let mut state = Self::Hasher::default();
        state.write(&SimpleSerializer::<Vec<u8>>::serialize(self).unwrap());
        state.finish()
    }
}
