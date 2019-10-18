use crate::account_address::AccountAddress;
use failure::prelude::*;
use hex;
use libra_crypto::HashValue;
#[cfg(any(test, feature = "testing"))]
use proptest_derive::Arbitrary;
use serde::{de, ser, Deserialize, Serialize};
use std::{convert::TryFrom, fmt};

/// Size of an event key.
pub const EVENT_KEY_LENGTH: usize = 32;

/// A struct that represents a globally unique id for an Event stream that a user can listen to.
#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Default, Clone, Copy)]
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

// TODO(#1307)
impl ser::Serialize for EventKey {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_bytes(&self.0)
    }
}

impl<'de> de::Deserialize<'de> for EventKey {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct EventKeyVisitor;

        impl<'de> de::Visitor<'de> for EventKeyVisitor {
            type Value = EventKey;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("EventKey in bytes")
            }

            fn visit_bytes<E>(self, value: &[u8]) -> std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                EventKey::try_from(value).map_err(E::custom)
            }
        }

        deserializer.deserialize_bytes(EventKeyVisitor)
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
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventHandle {
    /// Number of events in the event stream.
    count: u64,
    /// The associated globally unique key that is used as the key to the EventStore.
    key: EventKey,
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
