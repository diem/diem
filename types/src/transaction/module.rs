// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use canonical_serialization::{
    CanonicalDeserialize, CanonicalDeserializer, CanonicalSerialize, CanonicalSerializer,
};
use failure::prelude::*;
use proto_conv::{FromProto, IntoProto};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Module {
    code: Vec<u8>,
}

impl Module {
    pub fn new(code: Vec<u8>) -> Module {
        Module { code }
    }

    pub fn code(&self) -> &[u8] {
        &self.code
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.code
    }
}

impl fmt::Debug for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // XXX note that "code" will eventually be encoded bytecode and will no longer be a
        // UTF8-ish string -- at that point the from_utf8_lossy will stop making sense.
        f.debug_struct("Module")
            .field("code", &String::from_utf8_lossy(&self.code))
            .finish()
    }
}

impl FromProto for Module {
    type ProtoType = crate::proto::transaction::Module;

    fn from_proto(proto_module: Self::ProtoType) -> Result<Self> {
        Ok(Module::new(proto_module.get_code().to_vec()))
    }
}

impl IntoProto for Module {
    type ProtoType = crate::proto::transaction::Module;

    fn into_proto(self) -> Self::ProtoType {
        let mut proto_module = Self::ProtoType::new();
        proto_module.set_code(self.code);
        proto_module
    }
}

impl CanonicalSerialize for Module {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer.encode_vec(&self.code)?;
        Ok(())
    }
}

impl CanonicalDeserialize for Module {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self> {
        let code: Vec<u8> = deserializer.decode_vec()?;
        Ok(Module::new(code))
    }
}
