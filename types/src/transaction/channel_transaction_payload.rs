use serde::{Deserialize, Serialize};

use canonical_serialization::{
    CanonicalDeserialize, CanonicalDeserializer, CanonicalSerialize, CanonicalSerializer,
    SimpleSerializer,
};
use crypto::{
    hash::{CryptoHash, CryptoHasher, TestOnlyHasher},
    HashValue,
};
use failure::prelude::*;

use crate::{account_address::AccountAddress, transaction::Script, write_set::WriteSet};

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ChannelWriteSetPayload {
    pub channel_sequence_number: u64,
    pub write_set: WriteSet,
    pub receiver: AccountAddress,
}

impl ChannelWriteSetPayload {
    pub fn new(
        channel_sequence_number: u64,
        write_set: WriteSet,
        receiver: AccountAddress,
    ) -> Self {
        Self {
            channel_sequence_number,
            write_set,
            receiver,
        }
    }

    pub fn write_set(&self) -> &WriteSet {
        &self.write_set
    }
}

impl CanonicalSerialize for ChannelWriteSetPayload {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer.encode_u64(self.channel_sequence_number)?;
        serializer.encode_struct(&self.write_set)?;
        serializer.encode_struct(&self.receiver)?;
        Ok(())
    }
}

impl CanonicalDeserialize for ChannelWriteSetPayload {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self>
    where
        Self: Sized,
    {
        let channel_sequence_number = deserializer.decode_u64()?;
        let write_set = deserializer.decode_struct()?;
        let receiver = deserializer.decode_struct()?;
        Ok(Self {
            channel_sequence_number,
            write_set,
            receiver,
        })
    }
}

impl CryptoHash for ChannelWriteSetPayload {
    //TODO use special hasher
    type Hasher = TestOnlyHasher;

    fn hash(&self) -> HashValue {
        let mut state = Self::Hasher::default();
        state.write(
            SimpleSerializer::<Vec<u8>>::serialize(self)
                .expect("Failed to serialize ChannelWriteSetPayload")
                .as_slice(),
        );
        state.finish()
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ChannelScriptPayload {
    pub channel_sequence_number: u64,
    pub write_set: WriteSet,
    pub receiver: AccountAddress,
    pub script: Script,
}

impl ChannelScriptPayload {
    pub fn new(
        channel_sequence_number: u64,
        write_set: WriteSet,
        receiver: AccountAddress,
        script: Script,
    ) -> Self {
        Self {
            channel_sequence_number,
            write_set,
            receiver,
            script,
        }
    }

    pub fn write_set(&self) -> &WriteSet {
        &self.write_set
    }

    pub fn script(&self) -> &Script {
        &self.script
    }
}

impl CanonicalSerialize for ChannelScriptPayload {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer.encode_u64(self.channel_sequence_number)?;
        serializer.encode_struct(&self.write_set)?;
        serializer.encode_struct(&self.receiver)?;
        serializer.encode_struct(&self.script)?;
        Ok(())
    }
}

impl CanonicalDeserialize for ChannelScriptPayload {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self>
    where
        Self: Sized,
    {
        let channel_sequence_number = deserializer.decode_u64()?;
        let write_set = deserializer.decode_struct()?;
        let receiver = deserializer.decode_struct()?;
        let script = deserializer.decode_struct()?;
        Ok(Self {
            channel_sequence_number,
            write_set,
            receiver,
            script,
        })
    }
}

impl CryptoHash for ChannelScriptPayload {
    //TODO use special hasher
    type Hasher = TestOnlyHasher;

    fn hash(&self) -> HashValue {
        let mut state = Self::Hasher::default();
        state.write(
            SimpleSerializer::<Vec<u8>>::serialize(self)
                .expect("Failed to serialize ChannelScriptPayload")
                .as_slice(),
        );
        state.finish()
    }
}
