// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{ledger_info::LedgerInfo, transaction::Version, validator_set::ValidatorSet};
use anyhow::{ensure, format_err, Error, Result};
use libra_crypto::hash::{CryptoHash, CryptoHasher, HashValue};
use libra_crypto_derive::CryptoHasher;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

// The delimiter between the version and the hash.
const WAYPOINT_DELIMITER: char = ':';

/// Waypoint keeps information about the LedgerInfo on a given reconfiguration, which provides an
/// off-chain mechanism to verify the sync process right after the restart.
/// At high level, a trusted waypoint verifies the LedgerInfo for a certain epoch change.
/// For more information, please refer to the Waypoints documentation.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Waypoint {
    /// The version of the reconfiguration transaction that is being approved by this waypoint.
    version: Version,
    /// The hash of the chosen fields of LedgerInfo (including the next validator set).
    value: HashValue,
}

impl Waypoint {
    /// Generates a new waypoint given the LedgerInfo.
    /// Errors in case the given LedgerInfo does not include the validator set.
    pub fn new(ledger_info: &LedgerInfo) -> Result<Self> {
        let converter = Ledger2WaypointConverter::new(ledger_info)?;
        Ok(Self {
            version: ledger_info.version(),
            value: converter.hash(),
        })
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
        let converter = Ledger2WaypointConverter::new(ledger_info)?;
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
#[derive(Deserialize, Serialize, CryptoHasher)]
struct Ledger2WaypointConverter {
    epoch: u64,
    root_hash: HashValue,
    version: Version,
    timestamp_usecs: u64,
    next_validator_set: ValidatorSet,
}

impl Ledger2WaypointConverter {
    pub fn new(ledger_info: &LedgerInfo) -> Result<Self> {
        Ok(Self {
            epoch: ledger_info.epoch(),
            root_hash: ledger_info.transaction_accumulator_hash(),
            version: ledger_info.version(),
            timestamp_usecs: ledger_info.timestamp_usecs(),
            next_validator_set: ledger_info
                .next_validator_set()
                .ok_or_else(|| format_err!("Cannot create a waypoint without validator set"))?
                .clone(),
        })
    }
}

impl CryptoHash for Ledger2WaypointConverter {
    type Hasher = Ledger2WaypointConverterHasher;

    fn hash(&self) -> HashValue {
        let bytes = lcs::to_bytes(self).expect("Ledger2WaypointConverter serialization failed");
        let mut state = Self::Hasher::default();
        state.write(bytes.as_ref());
        state.finish()
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
        assert!(Waypoint::new(&empty_li).is_err()); // no validator set in empty LI
        let li = LedgerInfo::new(
            BlockInfo::new(
                1,
                10,
                HashValue::random(),
                HashValue::random(),
                123,
                1000,
                Some(ValidatorSet::new(vec![])),
            ),
            HashValue::zero(),
        );
        let waypoint = Waypoint::new(&li).unwrap();
        assert!(waypoint.verify(&li).is_ok());
    }
}
