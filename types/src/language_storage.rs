// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    access_path::AccessPath, account_address::AccountAddress,
    proto::language_storage::ModuleId as ProtoModuleId,
};
use canonical_serialization::{
    CanonicalDeserialize, CanonicalDeserializer, CanonicalSerialize, CanonicalSerializer,
    SimpleSerializer,
};
use crypto::hash::{AccessPathHasher, CryptoHash, CryptoHasher, HashValue};
use failure::Result;
#[cfg(any(test, feature = "testing"))]
use proptest_derive::Arbitrary;
use proto_conv::{FromProto, IntoProto};
use serde::{Deserialize, Serialize};
use std::string::String;

#[derive(Serialize, Deserialize, Debug, PartialEq, Hash, Eq, Clone, PartialOrd, Ord)]
pub struct StructTag {
    pub address: AccountAddress,
    pub module: String,
    pub name: String,
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
#[derive(
    Serialize, Deserialize, Debug, PartialEq, Hash, Eq, Clone, PartialOrd, Ord, FromProto, IntoProto,
)]
#[cfg_attr(any(test, feature = "testing"), derive(Arbitrary))]
#[ProtoType(ProtoModuleId)]
#[cfg_attr(any(test, feature = "testing"), proptest(no_params))]
pub struct ModuleId {
    address: AccountAddress,
    name: String,
}

impl ModuleId {
    pub fn new(address: AccountAddress, name: String) -> Self {
        ModuleId { address, name }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn address(&self) -> &AccountAddress {
        &self.address
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
            .encode_variable_length_bytes(self.name.as_bytes())?;
        Ok(())
    }
}

impl CanonicalDeserialize for ModuleId {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self> {
        let address = deserializer.decode_struct::<AccountAddress>()?;
        let name = String::from_utf8(deserializer.decode_variable_length_bytes()?)?;

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
            .encode_variable_length_bytes(self.module.as_bytes())?
            .encode_variable_length_bytes(self.name.as_bytes())?
            .encode_vec(&self.type_params)?;
        Ok(())
    }
}

impl CanonicalDeserialize for StructTag {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self> {
        let address = deserializer.decode_struct::<AccountAddress>()?;
        let module = String::from_utf8(deserializer.decode_variable_length_bytes()?)?;
        let name = String::from_utf8(deserializer.decode_variable_length_bytes()?)?;
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
