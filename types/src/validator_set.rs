// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    access_path::{AccessPath, Accesses},
    account_config::core_code_address,
    identifier::{IdentStr, Identifier},
    language_storage::StructTag,
    validator_public_keys::ValidatorPublicKeys,
};
use failure::prelude::*;
use lazy_static::lazy_static;
use libra_canonical_serialization::{
    CanonicalDeserialize, CanonicalDeserializer, CanonicalSerialize, CanonicalSerializer,
};
use libra_proto_conv::{FromProto, IntoProto};
#[cfg(any(test, feature = "testing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};
use std::fmt;

lazy_static! {
    static ref VALIDATOR_SET_MODULE_NAME: Identifier = Identifier::new("ValidatorSet").unwrap();
    static ref VALIDATOR_SET_STRUCT_NAME: Identifier = Identifier::new("T").unwrap();
}

pub fn validator_set_module_name() -> &'static IdentStr {
    &*VALIDATOR_SET_MODULE_NAME
}

pub fn validator_set_struct_name() -> &'static IdentStr {
    &*VALIDATOR_SET_STRUCT_NAME
}

pub fn validator_set_tag() -> StructTag {
    StructTag {
        name: validator_set_struct_name().to_owned(),
        address: core_code_address(),
        module: validator_set_module_name().to_owned(),
        type_params: vec![],
    }
}

pub(crate) fn validator_set_path() -> Vec<u8> {
    AccessPath::resource_access_vec(&validator_set_tag(), &Accesses::empty())
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(any(test, feature = "testing"), derive(Arbitrary))]
pub struct ValidatorSet(Vec<ValidatorPublicKeys>);

impl fmt::Display for ValidatorSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        write!(f, "[")?;
        for validator in &self.0 {
            write!(f, "{} ", validator)?;
        }
        write!(f, "]")
    }
}

impl ValidatorSet {
    /// Constructs a ValidatorSet resource.
    pub fn new(payload: Vec<ValidatorPublicKeys>) -> Self {
        ValidatorSet(payload)
    }

    pub fn payload(&self) -> &[ValidatorPublicKeys] {
        &self.0
    }
}

impl CanonicalSerialize for ValidatorSet {
    fn serialize(&self, mut serializer: &mut impl CanonicalSerializer) -> Result<()> {
        // TODO: We do not use encode_vec and decode_vec because the VM serializes these
        // differently. This will be fixed once collections are supported in the language.
        serializer = serializer.encode_u64(self.0.len() as u64)?;
        for validator_public_keys in &self.0 {
            serializer = serializer.encode_struct(validator_public_keys)?;
        }
        Ok(())
    }
}

impl CanonicalDeserialize for ValidatorSet {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self> {
        let size = deserializer.decode_u64()?;
        let mut payload = vec![];
        for _i in 0..size {
            payload.push(deserializer.decode_struct::<ValidatorPublicKeys>()?);
        }
        Ok(ValidatorSet::new(payload))
    }
}

impl FromProto for ValidatorSet {
    type ProtoType = crate::proto::validator_set::ValidatorSet;

    fn from_proto(mut object: Self::ProtoType) -> Result<Self> {
        Ok(ValidatorSet::new(
            object
                .take_validator_public_keys()
                .into_iter()
                .map(ValidatorPublicKeys::from_proto)
                .collect::<Result<Vec<_>>>()?,
        ))
    }
}

impl IntoProto for ValidatorSet {
    type ProtoType = crate::proto::validator_set::ValidatorSet;

    fn into_proto(self) -> Self::ProtoType {
        let mut out = Self::ProtoType::new();
        out.set_validator_public_keys(protobuf::RepeatedField::from_vec(
            self.0
                .into_iter()
                .map(ValidatorPublicKeys::into_proto)
                .collect(),
        ));
        out
    }
}
