use crate::account_address::AccountAddress;
use canonical_serialization::{
    CanonicalDeserialize, CanonicalDeserializer, CanonicalSerialize, CanonicalSerializer,
};
use crypto::HashValue;
use failure::prelude::*;
use hex;
#[cfg(any(test, feature = "testing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, fmt};

/// Size of an event key.
pub const EVENT_KEY_LENGTH: usize = 32;

/// A struct that represents a globally unique id for an Event stream that a user can listen to.
#[derive(
    Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Default, Clone, Serialize, Deserialize, Copy,
)]
#[cfg_attr(any(test, feature = "testing"), derive(Arbitrary))]
pub struct EventKey([u8; EVENT_KEY_LENGTH]);

impl EventKey {
    /// Construct a new EventKey from a byte array slice.
    pub fn new(key: [u8; EVENT_KEY_LENGTH]) -> Self {
        EventKey(key)
    }

    /// Get the byte representation of the event key.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Convert event key into a byte array.
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    #[cfg(any(test, feature = "testing"))]
    /// Create a random event key for testing
    pub fn random() -> Self {
        EventKey::try_from(HashValue::random().to_vec().as_slice()).unwrap()
    }

    /// Create a unique handle by using an AccountAddress and a counter.
    pub fn new_from_address(addr: &AccountAddress, salt: u64) -> Self {
        let mut output_bytes = salt.to_be_bytes().to_vec();
        output_bytes.append(&mut addr.to_vec());
        EventKey(*HashValue::from_sha3_256(&output_bytes).as_ref())
    }
}

impl TryFrom<&[u8]> for EventKey {
    type Error = failure::Error;

    /// Tries to convert the provided byte array into Event Key.
    fn try_from(bytes: &[u8]) -> Result<EventKey> {
        ensure!(
            bytes.len() == EVENT_KEY_LENGTH,
            "The Address {:?} is of invalid length",
            bytes
        );
        let mut addr = [0u8; EVENT_KEY_LENGTH];
        addr.copy_from_slice(bytes);
        Ok(EventKey(addr))
    }
}

/// A Rust representation of an Event Handle Resource.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct EventHandle {
    /// The associated globally unique key that is used as the key to the EventStore.
    key: EventKey,
    /// Number of events in the event stream.
    count: u64,
}

impl EventHandle {
    /// Constructs a new Event Handle
    pub fn new(key: EventKey, count: u64) -> Self {
        EventHandle { key, count }
    }

    /// Return the key to where this event is stored in EventStore.
    pub fn key(&self) -> &EventKey {
        &self.key
    }
    /// Return the counter for the handle
    pub fn count(&self) -> u64 {
        self.count
    }

    #[cfg(any(test, feature = "testing"))]
    pub fn count_mut(&mut self) -> &mut u64 {
        &mut self.count
    }

    #[cfg(any(test, feature = "testing"))]
    /// Create a random event handle for testing
    pub fn random_handle(count: u64) -> Self {
        Self {
            key: EventKey::random(),
            count,
        }
    }

    #[cfg(any(test, feature = "testing"))]
    /// Derive a unique handle by using an AccountAddress and a counter.
    pub fn new_from_address(addr: &AccountAddress, salt: u64) -> Self {
        Self {
            key: EventKey::new_from_address(addr, salt),
            count: 0,
        }
    }
}

impl fmt::LowerHex for EventKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(&self.0))
    }
}

impl fmt::Display for EventKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        // Forward to the LowerHex impl with a "0x" prepended (the # flag).
        write!(f, "{:#x}", self)
    }
}

impl CanonicalSerialize for EventKey {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        // We cannot use encode_raw_bytes as this structure will represent how Move Value of type
        // EventKey is serialized into. And since Move doesn't have fix length bytearray, values
        // can't be encoded in the fix length fasion.
        serializer.encode_bytes(&self.0)?;
        Ok(())
    }
}

impl CanonicalDeserialize for EventKey {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self> {
        let bytes = deserializer.decode_bytes()?;
        Self::try_from(bytes.as_slice())
    }
}

impl CanonicalSerialize for EventHandle {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer
            .encode_u64(self.count)?
            .encode_struct(&self.key)?;
        Ok(())
    }
}

impl CanonicalDeserialize for EventHandle {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self> {
        let count = deserializer.decode_u64()?;
        let key = deserializer.decode_struct()?;
        Ok(EventHandle { count, key })
    }
}
