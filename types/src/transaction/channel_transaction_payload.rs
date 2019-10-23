use serde::{Deserialize, Serialize};

use canonical_serialization::{
    CanonicalDeserialize, CanonicalDeserializer, CanonicalSerialize, CanonicalSerializer,
    SimpleSerializer,
};
use crypto::{
    hash::{CryptoHash, CryptoHasher, TestOnlyHasher},
    HashValue, SigningKey,
};
use failure::prelude::*;

use crate::{account_address::AccountAddress, transaction::Script, write_set::WriteSet};
use crypto::ed25519::{Ed25519PrivateKey, Ed25519PublicKey, Ed25519Signature};

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum ChannelTransactionPayloadBody {
    WriteSet(ChannelWriteSetBody),
    Script(ChannelScriptBody),
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
enum ChannelTransactionPayloadBodyType {
    WriteSet = 0,
    Script = 1,
}

impl ChannelTransactionPayloadBodyType {
    fn from_u32(value: u32) -> Option<ChannelTransactionPayloadBodyType> {
        match value {
            0 => Some(ChannelTransactionPayloadBodyType::WriteSet),
            1 => Some(ChannelTransactionPayloadBodyType::Script),
            _ => None,
        }
    }
}

impl CanonicalSerialize for ChannelTransactionPayloadBody {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        match self {
            ChannelTransactionPayloadBody::WriteSet(write_set) => {
                serializer.encode_u32(ChannelTransactionPayloadBodyType::WriteSet as u32)?;
                serializer.encode_struct(write_set)?;
            }
            ChannelTransactionPayloadBody::Script(script) => {
                serializer.encode_u32(ChannelTransactionPayloadBodyType::Script as u32)?;
                serializer.encode_struct(script)?;
            }
        }
        Ok(())
    }
}

impl CanonicalDeserialize for ChannelTransactionPayloadBody {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self>
    where
        Self: Sized,
    {
        let decoded_body_type = deserializer.decode_u32()?;
        let body_type = ChannelTransactionPayloadBodyType::from_u32(decoded_body_type);
        match body_type {
            Some(ChannelTransactionPayloadBodyType::WriteSet) => Ok(
                ChannelTransactionPayloadBody::WriteSet(deserializer.decode_struct()?),
            ),
            Some(ChannelTransactionPayloadBodyType::Script) => Ok(
                ChannelTransactionPayloadBody::Script(deserializer.decode_struct()?),
            ),
            _ => Err(format_err!(
                "ParseError: Unable to decode ChannelTransactionPayloadBody, found {}",
                decoded_body_type
            )),
        }
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ChannelTransactionPayload {
    pub body: ChannelTransactionPayloadBody,
    pub receiver_public_key: Ed25519PublicKey,
    /// signature for body.
    pub receiver_signature: Ed25519Signature,
}

impl ChannelTransactionPayload {
    pub fn new(
        body: ChannelTransactionPayloadBody,
        receiver_public_key: Ed25519PublicKey,
        receiver_signature: Ed25519Signature,
    ) -> Self {
        Self {
            body,
            receiver_public_key,
            receiver_signature,
        }
    }

    pub fn new_with_write_set(
        body: ChannelWriteSetBody,
        receiver_public_key: Ed25519PublicKey,
        receiver_signature: Ed25519Signature,
    ) -> Self {
        Self {
            body: ChannelTransactionPayloadBody::WriteSet(body),
            receiver_public_key,
            receiver_signature,
        }
    }

    pub fn new_with_script(
        body: ChannelScriptBody,
        receiver_public_key: Ed25519PublicKey,
        receiver_signature: Ed25519Signature,
    ) -> Self {
        Self {
            body: ChannelTransactionPayloadBody::Script(body),
            receiver_public_key,
            receiver_signature,
        }
    }

    pub fn receiver(&self) -> AccountAddress {
        match self.body {
            ChannelTransactionPayloadBody::Script(ChannelScriptBody { receiver, .. })
            | ChannelTransactionPayloadBody::WriteSet(ChannelWriteSetBody { receiver, .. }) => {
                receiver
            }
        }
    }

    pub fn channel_sequence_number(&self) -> u64 {
        match self.body {
            ChannelTransactionPayloadBody::Script(ChannelScriptBody {
                channel_sequence_number,
                ..
            })
            | ChannelTransactionPayloadBody::WriteSet(ChannelWriteSetBody {
                channel_sequence_number,
                ..
            }) => channel_sequence_number,
        }
    }

    pub fn write_set(&self) -> &WriteSet {
        match &self.body {
            ChannelTransactionPayloadBody::Script(ChannelScriptBody { write_set, .. })
            | ChannelTransactionPayloadBody::WriteSet(ChannelWriteSetBody { write_set, .. }) => {
                write_set
            }
        }
    }
}

impl CanonicalSerialize for ChannelTransactionPayload {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer.encode_struct(&self.body)?;
        serializer.encode_struct(&self.receiver_public_key)?;
        serializer.encode_struct(&self.receiver_signature)?;
        Ok(())
    }
}

impl CanonicalDeserialize for ChannelTransactionPayload {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self>
    where
        Self: Sized,
    {
        let body = deserializer.decode_struct()?;
        let receiver_public_key = deserializer.decode_struct()?;
        let receiver_signature = deserializer.decode_struct()?;
        Ok(Self {
            body,
            receiver_public_key,
            receiver_signature,
        })
    }
}

impl CryptoHash for ChannelTransactionPayload {
    //TODO use special hasher
    type Hasher = TestOnlyHasher;

    fn hash(&self) -> HashValue {
        let mut state = Self::Hasher::default();
        state.write(
            SimpleSerializer::<Vec<u8>>::serialize(self)
                .expect("Failed to serialize ChannelTransactionPayload")
                .as_slice(),
        );
        state.finish()
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ChannelWriteSetBody {
    pub channel_sequence_number: u64,
    pub write_set: WriteSet,
    pub receiver: AccountAddress,
}

impl ChannelWriteSetBody {
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

    pub fn sign(
        self,
        private_key: &Ed25519PrivateKey,
        public_key: Ed25519PublicKey,
    ) -> ChannelTransactionPayload {
        let hash = self.hash();
        let signature = private_key.sign_message(&hash);
        ChannelTransactionPayload::new(
            ChannelTransactionPayloadBody::WriteSet(self),
            public_key,
            signature,
        )
    }
}

impl CanonicalSerialize for ChannelWriteSetBody {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer.encode_u64(self.channel_sequence_number)?;
        serializer.encode_struct(&self.write_set)?;
        serializer.encode_struct(&self.receiver)?;
        Ok(())
    }
}

impl CanonicalDeserialize for ChannelWriteSetBody {
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

impl CryptoHash for ChannelWriteSetBody {
    //TODO use special hasher
    type Hasher = TestOnlyHasher;

    fn hash(&self) -> HashValue {
        let mut state = Self::Hasher::default();
        state.write(
            SimpleSerializer::<Vec<u8>>::serialize(self)
                .expect("Failed to serialize ChannelWriteSetBody")
                .as_slice(),
        );
        state.finish()
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ChannelScriptBody {
    pub channel_sequence_number: u64,
    pub write_set: WriteSet,
    pub receiver: AccountAddress,
    pub script: Script,
}

impl ChannelScriptBody {
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

    pub fn sign(
        self,
        private_key: &Ed25519PrivateKey,
        public_key: Ed25519PublicKey,
    ) -> ChannelTransactionPayload {
        let hash = self.hash();
        let signature = private_key.sign_message(&hash);
        ChannelTransactionPayload::new(
            ChannelTransactionPayloadBody::Script(self),
            public_key,
            signature,
        )
    }
}

impl CanonicalSerialize for ChannelScriptBody {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer.encode_u64(self.channel_sequence_number)?;
        serializer.encode_struct(&self.write_set)?;
        serializer.encode_struct(&self.receiver)?;
        serializer.encode_struct(&self.script)?;
        Ok(())
    }
}

impl CanonicalDeserialize for ChannelScriptBody {
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

impl CryptoHash for ChannelScriptBody {
    //TODO use special hasher
    type Hasher = TestOnlyHasher;

    fn hash(&self) -> HashValue {
        let mut state = Self::Hasher::default();
        state.write(
            SimpleSerializer::<Vec<u8>>::serialize(self)
                .expect("Failed to serialize ChannelScriptBody")
                .as_slice(),
        );
        state.finish()
    }
}
