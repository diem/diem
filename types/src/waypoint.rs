// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    epoch_change::Verifier,
    epoch_state::EpochState,
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    transaction::Version,
};
use anyhow::{ensure, format_err, Error, Result};
use diem_crypto::hash::{CryptoHash, HashValue};
use diem_crypto_derive::{BCSCryptoHash, CryptoHasher};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

// The delimiter between the version and the hash.
const WAYPOINT_DELIMITER: char = ':';

/// Waypoint keeps information about the LedgerInfo on a given version, which provides an
/// off-chain mechanism to verify the sync process right after the restart.
/// At high level, a trusted waypoint verifies the LedgerInfo for a certain epoch change.
/// For more information, please refer to the Waypoints documentation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Waypoint {
    /// The version of the reconfiguration transaction that is being approved by this waypoint.
    version: Version,
    /// The hash of the chosen fields of LedgerInfo.
    value: HashValue,
}

impl Waypoint {
    /// Generate a new waypoint given any LedgerInfo.
    pub fn new_any(ledger_info: &LedgerInfo) -> Self {
        let converter = Ledger2WaypointConverter::new(ledger_info);
        Self {
            version: ledger_info.version(),
            value: converter.hash(),
        }
    }

    /// Generates a new waypoint given the epoch change LedgerInfo.
    pub fn new_epoch_boundary(ledger_info: &LedgerInfo) -> Result<Self> {
        ensure!(ledger_info.ends_epoch(), "No validator set");
        Ok(Self::new_any(ledger_info))
    }

    pub fn version(&self) -> Version {
        self.version
    }

    pub fn value(&self) -> HashValue {
        self.value
    }

    /// Errors in case the given ledger info does not match the waypoint.
    pub fn verify(&self, ledger_info: &LedgerInfo) -> Result<()> {
        ensure!(
            ledger_info.version() == self.version(),
            "Waypoint version mismatch: waypoint version = {}, given version = {}",
            self.version(),
            ledger_info.version()
        );
        let converter = Ledger2WaypointConverter::new(ledger_info);
        ensure!(
            converter.hash() == self.value(),
            format!(
                "Waypoint value mismatch: waypoint value = {}, given value = {}",
                self.value().to_hex(),
                converter.hash().to_hex()
            )
        );
        Ok(())
    }
}

impl Verifier for Waypoint {
    fn verify(&self, ledger_info: &LedgerInfoWithSignatures) -> Result<()> {
        self.verify(ledger_info.ledger_info())
    }

    fn epoch_change_verification_required(&self, _epoch: u64) -> bool {
        true
    }

    fn is_ledger_info_stale(&self, ledger_info: &LedgerInfo) -> bool {
        ledger_info.version() < self.version()
    }
}

impl Display for Waypoint {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.version(),
            WAYPOINT_DELIMITER,
            self.value().to_hex()
        )
    }
}

impl FromStr for Waypoint {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut split = s.split(WAYPOINT_DELIMITER);
        let version = split
            .next()
            .ok_or_else(|| format_err!("Failed to parse waypoint string {}", s))?
            .parse::<Version>()?;
        let value = HashValue::from_hex(
            split
                .next()
                .ok_or_else(|| format_err!("Failed to parse waypoint string {}", s))?,
        )?;
        Ok(Self { version, value })
    }
}

/// Keeps the fields of LedgerInfo that are hashed for generating a waypoint.
/// Note that not all the fields of LedgerInfo are included: some consensus-related fields
/// might not be the same for all the participants.
#[derive(Deserialize, Serialize, CryptoHasher, BCSCryptoHash)]
struct Ledger2WaypointConverter {
    epoch: u64,
    root_hash: HashValue,
    version: Version,
    timestamp_usecs: u64,
    next_epoch_state: Option<EpochState>,
}

impl Ledger2WaypointConverter {
    pub fn new(ledger_info: &LedgerInfo) -> Self {
        Self {
            epoch: ledger_info.epoch(),
            root_hash: ledger_info.transaction_accumulator_hash(),
            version: ledger_info.version(),
            timestamp_usecs: ledger_info.timestamp_usecs(),
            next_epoch_state: ledger_info.next_epoch_state().cloned(),
        }
    }
}

impl<'de> Deserialize<'de> for Waypoint {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = <String>::deserialize(deserializer)?;
            Waypoint::from_str(&s).map_err(D::Error::custom)
        } else {
            // In order to preserve the Serde data model and help analysis tools,
            // make sure to wrap our value in a container with the same name
            // as the original type.
            #[derive(::serde::Deserialize)]
            #[serde(rename = "Waypoint")]
            struct Value(Version, HashValue);

            let value = Value::deserialize(deserializer)?;
            Ok(Waypoint {
                version: value.0,
                value: value.1,
            })
        }
    }
}

impl Serialize for Waypoint {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            self.to_string().serialize(serializer)
        } else {
            // See comment in deserialize.
            serializer.serialize_newtype_struct("Waypoint", &(self.version, self.value))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::block_info::BlockInfo;
    use std::str::FromStr;

    #[test]
    fn test_waypoint_parsing() {
        let waypoint = Waypoint {
            version: 123,
            value: HashValue::random(),
        };
        let waypoint_str = waypoint.to_string();
        let parsed_waypoint = Waypoint::from_str(&waypoint_str).unwrap();
        assert_eq!(waypoint, parsed_waypoint);
    }

    #[test]
    fn test_waypoint_li_verification() {
        let empty_li = LedgerInfo::new(BlockInfo::empty(), HashValue::zero());
        assert!(Waypoint::new_epoch_boundary(&empty_li).is_err()); // no validator set in empty LI
        let li = LedgerInfo::new(
            BlockInfo::new(
                1,
                10,
                HashValue::random(),
                HashValue::random(),
                123,
                1000,
                Some(EpochState::empty()),
            ),
            HashValue::zero(),
        );
        let waypoint = Waypoint::new_epoch_boundary(&li).unwrap();
        assert!(waypoint.verify(&li).is_ok());
    }
}
