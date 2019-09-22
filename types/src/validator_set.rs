// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    access_path::{AccessPath, Accesses},
    account_config,
    identifier::{IdentStr, Identifier},
    language_storage::{ResourceKey, StructTag},
    validator_public_keys::ValidatorPublicKeys,
};
use canonical_serialization::{
    CanonicalDeserialize, CanonicalDeserializer, CanonicalSerialize, CanonicalSerializer,
    SimpleDeserializer,
};
use failure::prelude::*;
use lazy_static::lazy_static;
#[cfg(any(test, feature = "testing"))]
use proptest_derive::Arbitrary;
use proto_conv::{FromProto, IntoProto};
use serde::{Deserialize, Serialize};
use std::fmt;

lazy_static! {
    static ref VALIDATOR_SET_MODULE_NAME: Identifier = Identifier::new("ValidatorSet").unwrap();
    static ref VALIDATOR_SET_STRUCT_NAME: Identifier = Identifier::new("T").unwrap();
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(any(test, feature = "testing"), derive(Arbitrary))]
pub struct ValidatorSet {
    //    validators: ByteArray, // in Move, this is a Vector<ValidatorPublicKeys>
    validators: Vec<ValidatorPublicKeys>,
}

impl fmt::Display for ValidatorSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        write!(f, "[")?;
        for validator in &self.validators {
            write!(f, "{} ", validator)?;
        }
        write!(f, "]")
    }
}

impl ValidatorSet {
    pub fn new(validators: Vec<ValidatorPublicKeys>) -> Self {
        Self { validators }
    }

    pub fn payload(&self) -> &[ValidatorPublicKeys] {
        &self.validators
    }

    pub fn module_name() -> &'static IdentStr {
        &*VALIDATOR_SET_MODULE_NAME
    }

    pub fn struct_name() -> &'static IdentStr {
        &*VALIDATOR_SET_STRUCT_NAME
    }

    pub fn struct_tag() -> StructTag {
        StructTag {
            name: Self::struct_name().to_owned(),
            address: account_config::core_code_address(),
            module: Self::module_name().to_owned(),
            type_params: vec![],
        }
    }

    pub fn access_path() -> AccessPath {
        AccessPath::resource_access_path(
            &ResourceKey::new(account_config::validator_set_address(), Self::struct_tag()),
            &Accesses::empty(),
        )
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        SimpleDeserializer::deserialize(bytes)
    }
}

impl CanonicalSerialize for ValidatorSet {
    fn serialize(&self, mut serializer: &mut impl CanonicalSerializer) -> Result<()> {
        // TODO: We do not use encode_vec and decode_vec because the VM serializes these
        // differently. This will be fixed once collections are supported in the language.
        for validator_public_keys in &self.validators {
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
        Ok(ValidatorSet {
            validators: object
                .take_validator_public_keys()
                .into_iter()
                .map(ValidatorPublicKeys::from_proto)
                .collect::<Result<Vec<_>>>()?,
        })
    }
}

impl IntoProto for ValidatorSet {
    type ProtoType = crate::proto::validator_set::ValidatorSet;

    fn into_proto(self) -> Self::ProtoType {
        let mut out = Self::ProtoType::new();
        out.set_validator_public_keys(protobuf::RepeatedField::from_vec(
            self.validators
                .into_iter()
                .map(ValidatorPublicKeys::into_proto)
                .collect(),
        ));
        out
    }
}
