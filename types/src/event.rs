// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_address::AccountAddress;
use hex::FromHex;
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
#[cfg(any(test, feature = "fuzzing"))]
use rand::{rngs::OsRng, RngCore};
use serde::{de, ser, Deserialize, Serialize};
use std::{
    convert::{TryFrom, TryInto},
    fmt,
    str::FromStr,
};

/// A struct that represents a globally unique id for an Event stream that a user can listen to.
/// By design, the lower part of EventKey is the same as account address.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct EventKey([u8; EventKey::LENGTH]);

impl EventKey {
    /// Construct a new EventKey from a byte array slice.
    pub fn new(key: [u8; Self::LENGTH]) -> Self {
        EventKey(key)
    }

    /// The number of bytes in an EventKey.
    pub const LENGTH: usize = AccountAddress::LENGTH + 8;

    /// Get the byte representation of the event key.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Convert event key into a byte array.
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    /// Get the account address part in this event key
    pub fn get_creator_address(&self) -> AccountAddress {
        AccountAddress::try_from(&self.0[EventKey::LENGTH - AccountAddress::LENGTH..])
            .expect("get_creator_address failed")
    }

    /// If this is the `ith` EventKey` created by `get_creator_address()`, return `i`
    pub fn get_creation_number(&self) -> u64 {
        u64::from_le_bytes(self.0[0..8].try_into().unwrap())
    }

    #[cfg(any(test, feature = "fuzzing"))]
    /// Create a random event key for testing
    pub fn random() -> Self {
        let mut rng = OsRng;
        let salt = rng.next_u64();
        EventKey::new_from_address(&AccountAddress::random(), salt)
    }

    /// Create a unique handle by using an AccountAddress and a counter.
    pub fn new_from_address(addr: &AccountAddress, salt: u64) -> Self {
        let mut output_bytes = [0; Self::LENGTH];
        let (lhs, rhs) = output_bytes.split_at_mut(8);
        lhs.copy_from_slice(&salt.to_le_bytes());
        rhs.copy_from_slice(addr.as_ref());
        EventKey(output_bytes)
    }

    pub fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, EventKeyParseError> {
        <[u8; Self::LENGTH]>::from_hex(hex)
            .map_err(|_| EventKeyParseError)
            .map(Self)
    }

    pub fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, EventKeyParseError> {
        <[u8; Self::LENGTH]>::try_from(bytes.as_ref())
            .map_err(|_| EventKeyParseError)
            .map(Self)
    }
}

impl FromStr for EventKey {
    type Err = EventKeyParseError;

    fn from_str(s: &str) -> Result<Self, EventKeyParseError> {
        EventKey::from_hex(s)
    }
}

impl From<EventKey> for [u8; EventKey::LENGTH] {
    fn from(event_key: EventKey) -> Self {
        event_key.0
    }
}

impl From<&EventKey> for [u8; EventKey::LENGTH] {
    fn from(event_key: &EventKey) -> Self {
        event_key.0
    }
}

impl ser::Serialize for EventKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        if serializer.is_human_readable() {
            self.to_string().serialize(serializer)
        } else {
            // In order to preserve the Serde data model and help analysis tools,
            // make sure to wrap our value in a container with the same name
            // as the original type.
            serializer.serialize_newtype_struct("EventKey", serde_bytes::Bytes::new(&self.0))
        }
    }
}

impl<'de> de::Deserialize<'de> for EventKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        use serde::de::Error;

        if deserializer.is_human_readable() {
            let s = <String>::deserialize(deserializer)?;
            EventKey::from_hex(s).map_err(D::Error::custom)
        } else {
            // See comment in serialize.
            #[derive(::serde::Deserialize)]
            #[serde(rename = "EventKey")]
            struct Value<'a>(&'a [u8]);

            let value = Value::deserialize(deserializer)?;
            Self::try_from(value.0).map_err(D::Error::custom)
        }
    }
}

impl fmt::LowerHex for EventKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "0x")?;
        }

        for byte in &self.0 {
            write!(f, "{:02x}", byte)?;
        }

        Ok(())
    }
}

impl fmt::Display for EventKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:x}", self)
    }
}

impl fmt::Debug for EventKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        write!(f, "EventKey({:x})", self)
    }
}

impl TryFrom<&[u8]> for EventKey {
    type Error = EventKeyParseError;

    /// Tries to convert the provided byte array into Event Key.
    fn try_from(bytes: &[u8]) -> Result<EventKey, EventKeyParseError> {
        Self::from_bytes(bytes)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct EventKeyParseError;

impl fmt::Display for EventKeyParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        write!(f, "unable to parse EventKey")
    }
}

impl std::error::Error for EventKeyParseError {}

/// A Rust representation of an Event Handle Resource.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventHandle {
    /// Number of events in the event stream.
    count: u64,
    /// The associated globally unique key that is used as the key to the EventStore.
    key: EventKey,
}

impl EventHandle {
    /// Constructs a new Event Handle
    pub fn new(key: EventKey, count: u64) -> Self {
        EventHandle { count, key }
    }

    /// Return the key to where this event is stored in EventStore.
    pub fn key(&self) -> &EventKey {
        &self.key
    }

    /// Return the counter for the handle
    pub fn count(&self) -> u64 {
        self.count
    }

    #[cfg(any(test, feature = "fuzzing"))]
    pub fn count_mut(&mut self) -> &mut u64 {
        &mut self.count
    }

    #[cfg(any(test, feature = "fuzzing"))]
    /// Create a random event handle for testing
    pub fn random_handle(count: u64) -> Self {
        Self {
            key: EventKey::random(),
            count,
        }
    }

    #[cfg(any(test, feature = "fuzzing"))]
    /// Derive a unique handle by using an AccountAddress and a counter.
    pub fn new_from_address(addr: &AccountAddress, salt: u64) -> Self {
        Self {
            key: EventKey::new_from_address(addr, salt),
            count: 0,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::EventKey;

    #[test]
    fn test_display_impls() {
        let hex = "1000000000000000ca843279e3427144cead5e4d5999a3d0";

        let key = EventKey::from_hex(hex).unwrap();

        assert_eq!(format!("{}", key), hex);
        assert_eq!(format!("{:x}", key), hex);

        assert_eq!(format!("{:#x}", key), format!("0x{}", hex));
    }

    #[test]
    fn test_invalid_length() {
        let bytes = vec![1; 123];
        EventKey::from_bytes(bytes).unwrap_err();
    }

    #[test]
    fn test_deserialize_from_json_value() {
        let key = EventKey::random();
        let json_value = serde_json::to_value(key).unwrap();
        let key2: EventKey = serde_json::from_value(json_value).unwrap();
        assert_eq!(key, key2);
    }

    #[test]
    fn test_serde_json() {
        let hex = "1000000000000000ca843279e3427144cead5e4d5999a3d0";
        let json_hex = "\"1000000000000000ca843279e3427144cead5e4d5999a3d0\"";

        let key = EventKey::from_hex(hex).unwrap();

        let json = serde_json::to_string(&key).unwrap();
        let json_key: EventKey = serde_json::from_str(json_hex).unwrap();

        assert_eq!(json, json_hex);
        assert_eq!(key, json_key);
    }
}
