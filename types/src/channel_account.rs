use crate::{
    access_path::{AccessPath, DataPath},
    account_address::AccountAddress,
    account_config::core_code_address,
    identifier::{IdentStr, Identifier},
    language_storage::StructTag,
};
use canonical_serialization::{
    CanonicalDeserialize, CanonicalDeserializer, CanonicalSerialize, CanonicalSerializer,
    SimpleDeserializer, SimpleSerializer,
};
use failure::prelude::*;
use lazy_static::lazy_static;

lazy_static! {
    // ChannelAccount
    static ref CHANNEL_ACCOUNT_MODULE_NAME: Identifier = Identifier::new("ChannelAccount").unwrap();
    static ref CHANNEL_ACCOUNT_STRUCT_NAME: Identifier = Identifier::new("T").unwrap();
}

pub fn channel_account_module_name() -> &'static IdentStr {
    &*CHANNEL_ACCOUNT_MODULE_NAME
}

pub fn channel_account_struct_name() -> &'static IdentStr {
    &*CHANNEL_ACCOUNT_STRUCT_NAME
}

pub fn channel_account_struct_tag() -> StructTag {
    StructTag {
        address: core_code_address(),
        module: channel_account_module_name().to_owned(),
        name: channel_account_struct_name().to_owned(),
        type_params: vec![],
    }
}

pub fn channel_account_resource_path(participant: AccountAddress) -> Vec<u8> {
    DataPath::channel_resource_path(participant, channel_account_struct_tag()).to_vec()
}

/// A Rust representation of an ChannelAccount resource.
/// This is not how the ChannelAccount is represented in the VM but it's a convenient
/// representation.
#[derive(Debug, Default)]
pub struct ChannelAccountResource {
    balance: u64,
    channel_sequence_number: u64,
    closed: bool,
    participant: AccountAddress,
}

impl ChannelAccountResource {
    pub fn new(
        balance: u64,
        channel_sequence_number: u64,
        closed: bool,
        participant: AccountAddress,
    ) -> Self {
        Self {
            balance,
            channel_sequence_number,
            closed,
            participant,
        }
    }

    pub fn make_from(bytes: Vec<u8>) -> Result<Self> {
        SimpleDeserializer::deserialize(bytes.as_slice())
    }

    pub fn balance(&self) -> u64 {
        self.balance
    }

    pub fn channel_sequence_number(&self) -> u64 {
        self.channel_sequence_number
    }

    pub fn closed(&self) -> bool {
        self.closed
    }

    pub fn participant(self) -> AccountAddress {
        self.participant
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        SimpleSerializer::serialize(self).unwrap()
    }
}

impl CanonicalSerialize for ChannelAccountResource {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer
            .encode_u64(self.balance)?
            .encode_u64(self.channel_sequence_number)?
            .encode_bool(self.closed)?
            .encode_struct(&self.participant)?;
        Ok(())
    }
}

impl CanonicalDeserialize for ChannelAccountResource {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self> {
        let balance = deserializer.decode_u64()?;
        let channel_sequence_number = deserializer.decode_u64()?;
        let closed = deserializer.decode_bool()?;
        let participant = deserializer.decode_struct()?;

        Ok(ChannelAccountResource {
            balance,
            channel_sequence_number,
            closed,
            participant,
        })
    }
}
