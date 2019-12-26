use crate::access_path::{AccessPath, Accesses};
use crate::event::EventHandle;
use crate::{
    account_address::AccountAddress,
    account_config::core_code_address,
    identifier::{IdentStr, Identifier},
    language_storage::StructTag,
};
use anyhow::{bail, Result};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

lazy_static! {
    // Channel
    static ref CHANNEL_MODULE_NAME: Identifier = Identifier::new("LibraAccount").unwrap();
    static ref CHANNEL_STRUCT_NAME: Identifier = Identifier::new("Channel").unwrap();

    static ref CHANNEL_MIRROR_STRUCT_NAME: Identifier = Identifier::new("ChannelMirror").unwrap();

    static ref CHANNEL_PARTICIPANT_STRUCT_NAME: Identifier = Identifier::new("ChannelParticipantAccount").unwrap();

    static ref USER_CHANNELS_STRUCT_NAME: Identifier = Identifier::new("UserChannels").unwrap();

    static ref CHANNEL_GLOBAL_EVENTS_STRUCT_NAME: Identifier = Identifier::new("ChannelGlobalEvents").unwrap();
    static ref CHANNEL_EVENT_STRUCT_NAME: Identifier = Identifier::new("ChannelEvent").unwrap();
    static ref CHANNEL_LOCKED_BY_STRUCT_NAME: Identifier = Identifier::new("ChannelLockedBy").unwrap();
    static ref CHANNEL_CHALLENGE_BY_STRUCT_NAME: Identifier = Identifier::new("ChannelChallengedBy").unwrap();

    pub static ref CHANNEL_EVENT_PATH: Vec<u8> = {
        let mut path = channel_resource_path();
        path.extend_from_slice(b"/events_count/");
        path
    };

    pub static ref CHANNEL_GLOBAL_EVENT_PATH: Vec<u8> = {
        let mut path = channel_global_events_resource_path();
        path.extend_from_slice(b"/events_count/");
        path
    };
}

pub fn channel_module_name() -> &'static IdentStr {
    &*CHANNEL_MODULE_NAME
}

pub fn channel_struct_name() -> &'static IdentStr {
    &*CHANNEL_STRUCT_NAME
}

pub fn channel_global_events_struct_name() -> &'static IdentStr {
    &*CHANNEL_GLOBAL_EVENTS_STRUCT_NAME
}

pub fn channel_event_struct_name() -> &'static IdentStr {
    &*CHANNEL_EVENT_STRUCT_NAME
}
pub fn channel_locked_by_struct_name() -> &'static IdentStr {
    &*CHANNEL_LOCKED_BY_STRUCT_NAME
}
pub fn channel_challenge_by_struct_name() -> &'static IdentStr {
    &*CHANNEL_CHALLENGE_BY_STRUCT_NAME
}

pub fn channel_struct_tag() -> StructTag {
    StructTag {
        address: core_code_address(),
        module: channel_module_name().to_owned(),
        name: channel_struct_name().to_owned(),
        type_params: vec![],
    }
}

pub fn channel_event_struct_tag() -> StructTag {
    StructTag {
        address: core_code_address(),
        module: channel_module_name().to_owned(),
        name: channel_event_struct_name().to_owned(),
        type_params: vec![],
    }
}

pub fn channel_global_events_struct_tag() -> StructTag {
    StructTag {
        address: core_code_address(),
        module: channel_module_name().to_owned(),
        name: channel_global_events_struct_name().to_owned(),
        type_params: vec![],
    }
}
pub fn channel_locked_by_struct_tag() -> StructTag {
    StructTag {
        address: core_code_address(),
        module: channel_module_name().to_owned(),
        name: channel_locked_by_struct_name().to_owned(),
        type_params: vec![],
    }
}
pub fn channel_challenge_by_struct_tag() -> StructTag {
    StructTag {
        address: core_code_address(),
        module: channel_module_name().to_owned(),
        name: channel_challenge_by_struct_name().to_owned(),
        type_params: vec![],
    }
}

pub fn channel_resource_path() -> Vec<u8> {
    AccessPath::resource_access_vec(&channel_struct_tag(), &Accesses::empty())
}

pub fn channel_global_events_resource_path() -> Vec<u8> {
    AccessPath::resource_access_vec(&channel_global_events_struct_tag(), &Accesses::empty())
}

pub fn channel_mirror_struct_name() -> &'static IdentStr {
    &*CHANNEL_MIRROR_STRUCT_NAME
}

pub fn channel_mirror_struct_tag() -> StructTag {
    StructTag {
        address: core_code_address(),
        module: channel_module_name().to_owned(),
        name: channel_mirror_struct_name().to_owned(),
        type_params: vec![],
    }
}

pub fn channel_participant_struct_name() -> &'static IdentStr {
    &*CHANNEL_PARTICIPANT_STRUCT_NAME
}

pub fn channel_participant_struct_tag() -> StructTag {
    StructTag {
        address: core_code_address(),
        module: channel_module_name().to_owned(),
        name: channel_participant_struct_name().to_owned(),
        type_params: vec![],
    }
}

pub fn user_channels_struct_name() -> &'static IdentStr {
    &*USER_CHANNELS_STRUCT_NAME
}

pub fn user_channels_struct_tag() -> StructTag {
    StructTag {
        address: core_code_address(),
        module: channel_module_name().to_owned(),
        name: user_channels_struct_name().to_owned(),
        type_params: vec![],
    }
}

pub trait LibraResource {
    fn struct_tag() -> StructTag;
}

pub fn make_resource<'a, T>(value: &'a [u8]) -> Result<T>
where
    T: LibraResource + Deserialize<'a>,
{
    lcs::from_bytes(value).map_err(|e| Into::into(e))
}

impl LibraResource for ChannelResource {
    fn struct_tag() -> StructTag {
        channel_struct_tag()
    }
}

impl LibraResource for ChannelMirrorResource {
    fn struct_tag() -> StructTag {
        channel_mirror_struct_tag()
    }
}
impl LibraResource for ChannelParticipantAccountResource {
    fn struct_tag() -> StructTag {
        channel_participant_struct_tag()
    }
}
impl LibraResource for UserChannelsResource {
    fn struct_tag() -> StructTag {
        user_channels_struct_tag()
    }
}
impl LibraResource for ChannelEvent {
    fn struct_tag() -> StructTag {
        channel_event_struct_tag()
    }
}

/// A Rust representation of an Channel resource.
/// This is not how the Channel is represented in the VM but it's a convenient
/// representation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChannelResource {
    channel_sequence_number: u64,
    // 0 open, 1 locked, 2 closed.
    stage: u64,
    participants: Vec<AccountAddress>,
    events: EventHandle,
}

impl ChannelResource {
    pub fn new(
        channel_sequence_number: u64,
        participants: Vec<AccountAddress>,
        events: EventHandle,
    ) -> Self {
        Self {
            channel_sequence_number,
            stage: 0,
            participants,
            events,
        }
    }

    pub fn channel_sequence_number(&self) -> u64 {
        self.channel_sequence_number
    }

    pub fn closed(&self) -> bool {
        self.stage == 2
    }

    pub fn locked(&self) -> bool {
        self.stage == 1
    }

    pub fn participants(&self) -> &[AccountAddress] {
        self.participants.as_slice()
    }

    pub fn make_from(bytes: Vec<u8>) -> Result<Self> {
        lcs::from_bytes(bytes.as_slice()).map_err(|e| Into::into(e))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        lcs::to_bytes(self).unwrap()
    }

    pub fn get_event_handle_by_query_path(&self, query_path: &[u8]) -> Result<&EventHandle> {
        if *CHANNEL_EVENT_PATH == query_path {
            Ok(&self.events)
        } else {
            bail!("Unrecognized query path: {:?}", query_path);
        }
    }
}

// A resource to keep Channel global events handle.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChannelGlobalEventsResource {
    events: EventHandle,
}

impl ChannelGlobalEventsResource {
    pub fn new(events: EventHandle) -> Self {
        Self { events }
    }

    pub fn make_from(bytes: Vec<u8>) -> Result<Self> {
        lcs::from_bytes(bytes.as_slice()).map_err(|e| Into::into(e))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        lcs::to_bytes(self).unwrap()
    }

    pub fn get_event_handle_by_query_path(&self, query_path: &[u8]) -> Result<&EventHandle> {
        if *CHANNEL_GLOBAL_EVENT_PATH == query_path {
            Ok(&self.events)
        } else {
            bail!("Unrecognized query path: {:?}", query_path);
        }
    }
}

/// A Rust representation of an ChannelMirror resource.
/// ChannelMirror resource save on channel's shared resource space.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChannelMirrorResource {
    channel_sequence_number: u64,
}

impl ChannelMirrorResource {
    pub fn new(channel_sequence_number: u64) -> Self {
        Self {
            channel_sequence_number,
        }
    }

    pub fn channel_sequence_number(&self) -> u64 {
        self.channel_sequence_number
    }

    pub fn make_from(bytes: Vec<u8>) -> Result<Self> {
        lcs::from_bytes(bytes.as_slice()).map_err(|e| Into::into(e))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        lcs::to_bytes(self).unwrap()
    }
}

/// A Rust representation of an ChannelParticipantAccount resource.
/// ChannelParticipantAccount resource save on channel's participant resource space.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChannelParticipantAccountResource {
    balance: u64,
}

impl ChannelParticipantAccountResource {
    pub fn new(balance: u64) -> Self {
        Self { balance }
    }

    pub fn balance(&self) -> u64 {
        self.balance
    }

    pub fn make_from(bytes: Vec<u8>) -> Result<Self> {
        lcs::from_bytes(bytes.as_slice()).map_err(|e| Into::into(e))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        lcs::to_bytes(self).unwrap()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserChannelsResource {
    channels: Vec<AccountAddress>,
}

impl UserChannelsResource {
    pub fn new(channels: Vec<AccountAddress>) -> Self {
        Self { channels }
    }

    pub fn channels(&self) -> &[AccountAddress] {
        &self.channels
    }
}

impl Into<Vec<AccountAddress>> for UserChannelsResource {
    fn into(self) -> Vec<AccountAddress> {
        self.channels
    }
}

/// Generic struct that represents an Channel event.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ChannelEvent {
    channel_address: AccountAddress,
    stage: u64,
    balances: Vec<u64>,
}

impl ChannelEvent {
    pub fn make_from(bytes: &[u8]) -> Result<ChannelEvent> {
        lcs::from_bytes(bytes).map_err(Into::into)
    }

    pub fn channel_address(&self) -> AccountAddress {
        self.channel_address
    }

    pub fn stage(&self) -> u64 {
        self.stage
    }

    pub fn balances(&self) -> &[u64] {
        self.balances.as_slice()
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ChannelLockedBy {
    pub participant: AccountAddress,
    pub time_lock: u64,
}
impl LibraResource for ChannelLockedBy {
    fn struct_tag() -> StructTag {
        channel_locked_by_struct_tag()
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ChannelChallengeBy {
    pub participant: AccountAddress,
}
impl LibraResource for ChannelChallengeBy {
    fn struct_tag() -> StructTag {
        channel_challenge_by_struct_tag()
    }
}
