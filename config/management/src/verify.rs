// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{error::Error, SingleBackend};
use libra_secure_storage::Storage;
use libra_types::waypoint::Waypoint;
use std::{convert::TryInto, fmt::Write, str::FromStr};
use structopt::StructOpt;

/// Prints the public information within a store
#[derive(Debug, StructOpt)]
pub struct Verify {
    #[structopt(flatten)]
    backend: SingleBackend,
}

impl Verify {
    pub fn execute(self) -> Result<String, Error> {
        let storage: Box<dyn Storage> = self.backend.backend.try_into()?;
        storage
            .available()
            .map_err(|e| Error::LocalStorageUnavailable(e.to_string()))?;
        let mut buffer = String::new();

        writeln!(buffer, "Data stored in SecureStorage:").unwrap();
        writeln!(buffer, "=================================================").unwrap();
        writeln!(buffer, "Keys").unwrap();
        writeln!(buffer, "=================================================").unwrap();

        write_key(
            storage.as_ref(),
            &mut buffer,
            libra_global_constants::CONSENSUS_KEY,
        );
        write_key(
            storage.as_ref(),
            &mut buffer,
            libra_global_constants::FULLNODE_NETWORK_KEY,
        );
        write_key(
            storage.as_ref(),
            &mut buffer,
            libra_global_constants::OWNER_KEY,
        );
        write_key(
            storage.as_ref(),
            &mut buffer,
            libra_global_constants::OPERATOR_KEY,
        );
        write_key(
            storage.as_ref(),
            &mut buffer,
            libra_global_constants::VALIDATOR_NETWORK_KEY,
        );

        writeln!(buffer, "=================================================").unwrap();
        writeln!(buffer, "Data").unwrap();
        writeln!(buffer, "=================================================").unwrap();

        write_u64(storage.as_ref(), &mut buffer, libra_global_constants::EPOCH);
        write_u64(
            storage.as_ref(),
            &mut buffer,
            libra_global_constants::LAST_VOTED_ROUND,
        );
        write_u64(
            storage.as_ref(),
            &mut buffer,
            libra_global_constants::PREFERRED_ROUND,
        );
        write_waypoint(
            storage.as_ref(),
            &mut buffer,
            libra_global_constants::WAYPOINT,
        );

        writeln!(buffer, "=================================================").unwrap();

        Ok(buffer)
    }
}

fn write_key(storage: &dyn Storage, buffer: &mut String, key: &str) {
    let value = storage
        .get_public_key(key)
        .map(|c| c.public_key.to_string())
        .unwrap_or_else(|e| format!("{:?}", e));
    writeln!(buffer, "{} - {}", key, value).unwrap();
}

fn write_u64(storage: &dyn Storage, buffer: &mut String, key: &str) {
    let value = storage
        .get(key)
        .and_then(|c| c.value.u64())
        .map(|c| c.to_string())
        .unwrap_or_else(|e| format!("{:?}", e));
    writeln!(buffer, "{} - {}", key, value).unwrap();
}

fn write_waypoint(storage: &dyn Storage, buffer: &mut String, key: &str) {
    let value = storage
        .get(key)
        .and_then(|c| c.value.string())
        .map(|value| {
            if value.is_empty() {
                "empty".into()
            } else {
                Waypoint::from_str(&value)
                    .map(|c| c.to_string())
                    .unwrap_or_else(|_| "Invalid waypoint".into())
            }
        })
        .unwrap_or_else(|e| format!("{:?}", e));

    writeln!(buffer, "{} - {}", key, value).unwrap();
}
