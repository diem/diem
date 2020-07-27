// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use executor::db_bootstrapper;
use libra_global_constants::{
    CONSENSUS_KEY, EPOCH, FULLNODE_NETWORK_KEY, LAST_VOTED_ROUND, OPERATOR_ACCOUNT, OPERATOR_KEY,
    OWNER_ACCOUNT, OWNER_KEY, PREFERRED_ROUND, VALIDATOR_NETWORK_KEY, WAYPOINT,
};
use libra_management::{
    config::ConfigPath, error::Error, secure_backend::ValidatorBackend,
    storage::StorageWrapper as Storage,
};
use libra_temppath::TempPath;
use libra_types::{
    account_address::AccountAddress, account_config, account_state::AccountState,
    on_chain_config::ValidatorSet, validator_config::ValidatorConfig, waypoint::Waypoint,
};
use libra_vm::LibraVM;
use libradb::LibraDB;
use std::{
    convert::TryFrom,
    fmt::Write,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};
use storage_interface::{DbReader, DbReaderWriter};
use structopt::StructOpt;

/// Prints the public information within a store
#[derive(Debug, StructOpt)]
pub struct Verify {
    #[structopt(flatten)]
    config: ConfigPath,
    #[structopt(flatten)]
    backend: ValidatorBackend,
    /// If specified, compares the internal state to that of a
    /// provided genesis. Note, that a waypont might diverge from
    /// the provided genesis after execution has begun.
    #[structopt(long, verbatim_doc_comment)]
    genesis_path: Option<PathBuf>,
}

impl Verify {
    pub fn execute(self) -> Result<String, Error> {
        let config = self
            .config
            .load()?
            .override_validator_backend(&self.backend.validator_backend)?;
        let validator_storage = config.validator_backend();
        let mut buffer = String::new();

        writeln!(buffer, "Data stored in SecureStorage:").unwrap();
        write_break(&mut buffer);
        writeln!(buffer, "Keys").unwrap();
        write_break(&mut buffer);

        write_ed25519_key(&validator_storage, &mut buffer, CONSENSUS_KEY);
        write_x25519_key(&validator_storage, &mut buffer, FULLNODE_NETWORK_KEY);
        write_ed25519_key(&validator_storage, &mut buffer, OWNER_KEY);
        write_ed25519_key(&validator_storage, &mut buffer, OPERATOR_KEY);
        write_ed25519_key(&validator_storage, &mut buffer, VALIDATOR_NETWORK_KEY);

        write_break(&mut buffer);
        writeln!(buffer, "Data").unwrap();
        write_break(&mut buffer);

        write_string(&validator_storage, &mut buffer, OPERATOR_ACCOUNT);
        write_string(&validator_storage, &mut buffer, OWNER_ACCOUNT);
        write_u64(&validator_storage, &mut buffer, EPOCH);
        write_u64(&validator_storage, &mut buffer, LAST_VOTED_ROUND);
        write_u64(&validator_storage, &mut buffer, PREFERRED_ROUND);
        write_waypoint(&validator_storage, &mut buffer, WAYPOINT);

        write_break(&mut buffer);

        if let Some(genesis_path) = self.genesis_path.as_ref() {
            compare_genesis(&validator_storage, &mut buffer, genesis_path)?;
        }

        Ok(buffer)
    }
}

fn write_assert(buffer: &mut String, name: &str, value: bool) {
    let value = if value { "match" } else { "MISMATCH" };
    writeln!(buffer, "{} - {}", name, value).unwrap();
}

fn write_break(buffer: &mut String) {
    writeln!(
        buffer,
        "====================================================================================",
    )
    .unwrap();
}

fn write_ed25519_key(storage: &Storage, buffer: &mut String, key: &'static str) {
    let value = storage
        .ed25519_public_from_private(key)
        .map(|v| v.to_string())
        .unwrap_or_else(|e| e.to_string());
    writeln!(buffer, "{} - {}", key, value).unwrap();
}

fn write_x25519_key(storage: &Storage, buffer: &mut String, key: &'static str) {
    let value = storage
        .x25519_public_from_private(key)
        .map(|v| v.to_string())
        .unwrap_or_else(|e| e.to_string());
    writeln!(buffer, "{} - {}", key, value).unwrap();
}

fn write_string(storage: &Storage, buffer: &mut String, key: &'static str) {
    let value = storage.string(key).unwrap_or_else(|e| e.to_string());
    writeln!(buffer, "{} - {}", key, value).unwrap();
}

fn write_u64(storage: &Storage, buffer: &mut String, key: &'static str) {
    let value = storage
        .u64(key)
        .map(|v| v.to_string())
        .unwrap_or_else(|e| e.to_string());
    writeln!(buffer, "{} - {}", key, value).unwrap();
}

fn write_waypoint(storage: &Storage, buffer: &mut String, key: &'static str) {
    let value = storage
        .string(key)
        .map(|value| {
            if value.is_empty() {
                "empty".into()
            } else {
                Waypoint::from_str(&value)
                    .map(|c| c.to_string())
                    .unwrap_or_else(|_| "Invalid waypoint".into())
            }
        })
        .unwrap_or_else(|e| e.to_string());

    writeln!(buffer, "{} - {}", key, value).unwrap();
}

fn compare_genesis(
    storage: &Storage,
    buffer: &mut String,
    genesis_path: &PathBuf,
) -> Result<(), Error> {
    // Compute genesis and waypoint and compare to given waypoint
    let db_path = TempPath::new();
    let (db_rw, expected_waypoint) = compute_genesis(genesis_path, db_path.path())?;

    let actual_waypoint = storage.waypoint(WAYPOINT)?;
    write_assert(buffer, WAYPOINT, actual_waypoint == expected_waypoint);

    // Fetch on-chain validator config and compare on-chain keys to local keys
    let validator_account = storage.account_address(OWNER_ACCOUNT)?;
    let validator_config = validator_config(validator_account, db_rw.reader.clone())?;

    let actual_consensus_key = storage.ed25519_public_from_private(CONSENSUS_KEY)?;
    let expected_consensus_key = validator_config.consensus_public_key;
    write_assert(
        buffer,
        CONSENSUS_KEY,
        actual_consensus_key == expected_consensus_key,
    );

    let actual_validator_key = storage.x25519_public_from_private(VALIDATOR_NETWORK_KEY)?;
    let expected_validator_key = validator_config.validator_network_identity_public_key;
    write_assert(
        buffer,
        VALIDATOR_NETWORK_KEY,
        actual_validator_key == expected_validator_key,
    );

    let actual_fullnode_key = storage.x25519_public_from_private(FULLNODE_NETWORK_KEY)?;
    let expected_fullnode_key = validator_config.full_node_network_identity_public_key;
    write_assert(
        buffer,
        FULLNODE_NETWORK_KEY,
        actual_fullnode_key == expected_fullnode_key,
    );

    Ok(())
}

/// Compute the ledger given a genesis writeset transaction and return access to that ledger and
/// the waypoint for that state.
fn compute_genesis(
    genesis_path: &PathBuf,
    db_path: &Path,
) -> Result<(DbReaderWriter, Waypoint), Error> {
    let libradb =
        LibraDB::open(db_path, false, None).map_err(|e| Error::UnexpectedError(e.to_string()))?;
    let db_rw = DbReaderWriter::new(libradb);

    let mut file = File::open(genesis_path)
        .map_err(|e| Error::UnexpectedError(format!("Unable to open genesis file: {}", e)))?;
    let mut buffer = vec![];
    file.read_to_end(&mut buffer)
        .map_err(|e| Error::UnexpectedError(format!("Unable to read genesis: {}", e)))?;
    let genesis = lcs::from_bytes(&buffer)
        .map_err(|e| Error::UnexpectedError(format!("Unable to parse genesis: {}", e)))?;

    let waypoint = db_bootstrapper::bootstrap_db_if_empty::<LibraVM>(&db_rw, &genesis)
        .map_err(|e| Error::UnexpectedError(e.to_string()))?
        .ok_or_else(|| Error::UnexpectedError("Unable to generate a waypoint".to_string()))?;

    Ok((db_rw, waypoint))
}

/// Read from the ledger the validator config from the validator set for the specified account
fn validator_config(
    validator_account: AccountAddress,
    reader: Arc<dyn DbReader>,
) -> Result<ValidatorConfig, Error> {
    let blob = reader
        .get_latest_account_state(account_config::validator_set_address())
        .map_err(|e| {
            Error::UnexpectedError(format!("ValidatorSet Account issue {}", e.to_string()))
        })?
        .ok_or_else(|| Error::UnexpectedError("ValidatorSet Account not found".into()))?;
    let account_state = AccountState::try_from(&blob)
        .map_err(|e| Error::UnexpectedError(format!("Failed to parse blob: {}", e)))?;
    let validator_set: ValidatorSet = account_state
        .get_validator_set()
        .map_err(|e| Error::UnexpectedError(format!("ValidatorSet issue {}", e.to_string())))?
        .ok_or_else(|| Error::UnexpectedError("ValidatorSet does not exist".into()))?;
    let info = validator_set
        .payload()
        .iter()
        .find(|vi| vi.account_address() == &validator_account)
        .ok_or_else(|| {
            Error::UnexpectedError(format!(
                "Unable to find Validator account {:?}",
                &validator_account
            ))
        })?;
    Ok(info.config().clone())
}
