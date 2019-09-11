use failure::prelude::*;
use crate::write_set::WriteSet;
use crate::account_address::AccountAddress;
use crate::transaction::Script;
use canonical_serialization::{CanonicalSerialize, CanonicalSerializer, CanonicalDeserializer, CanonicalDeserialize};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ChannelWriteSetPayload {
    pub channel_sequence_number: u64,
    pub write_set: WriteSet,
    pub receiver: AccountAddress,
}

impl ChannelWriteSetPayload {

    pub fn new(channel_sequence_number:u64, write_set: WriteSet, receiver: AccountAddress) -> Self{
        Self{
            channel_sequence_number,
            write_set,
            receiver,
        }
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

    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self> where
        Self: Sized {
        let channel_sequence_number = deserializer.decode_u64()?;
        let write_set = deserializer.decode_struct()?;
        let receiver = deserializer.decode_struct()?;
        Ok(Self{
            channel_sequence_number,
            write_set,
            receiver,
        })
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

    pub fn new(channel_sequence_number:u64, write_set: WriteSet, receiver: AccountAddress, script: Script) -> Self{
        Self{
            channel_sequence_number,
            write_set,
            receiver,
            script
        }
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

    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self> where
        Self: Sized {
        let channel_sequence_number = deserializer.decode_u64()?;
        let write_set = deserializer.decode_struct()?;
        let receiver = deserializer.decode_struct()?;
        let script = deserializer.decode_struct()?;
        Ok(Self{
            channel_sequence_number,
            write_set,
            receiver,
            script,
        })
    }
}
