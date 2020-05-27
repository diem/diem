// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{error::Error, SingleBackend};
use executor::db_bootstrapper;
use libra_crypto::{ed25519::Ed25519PublicKey, x25519};
use libra_global_constants::{
    CONSENSUS_KEY, FULLNODE_NETWORK_KEY, VALIDATOR_NETWORK_KEY, WAYPOINT,
};
use libra_secure_storage::Storage;
use libra_temppath::TempPath;
use libra_types::{
    account_address::{self, AccountAddress},
    account_config,
    account_state::AccountState,
    on_chain_config::ValidatorSet,
    validator_config::ValidatorConfig,
    waypoint::Waypoint,
};
use libra_vm::LibraVM;
use libradb::LibraDB;
use std::{
    convert::{TryFrom, TryInto},
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
    backend: SingleBackend,
    /// If specified, compares the internal state to that of a
    /// provided genesis. Note, that a waypont might diverge from
    /// the provided genesis after execution has begun.
    #[structopt(long, verbatim_doc_comment)]
    genesis_path: Option<PathBuf>,
}

impl Verify {
    pub fn execute(self) -> Result<String, Error> {
        let storage: Box<dyn Storage> = self.backend.backend.try_into()?;
        storage
            .available()
            .map_err(|e| Error::LocalStorageUnavailable(e.to_string()))?;
        let mut buffer = String::new();

        writeln!(buffer, "Data stored in SecureStorage:").unwrap();
        write_break(&mut buffer);
        writeln!(buffer, "Keys").unwrap();
        write_break(&mut buffer);

        write_ed25519_key(storage.as_ref(), &mut buffer, CONSENSUS_KEY);
        write_x25519_key(storage.as_ref(), &mut buffer, FULLNODE_NETWORK_KEY);
        write_ed25519_key(
            storage.as_ref(),
            &mut buffer,
            libra_global_constants::OWNER_KEY,
        );
        write_ed25519_key(
            storage.as_ref(),
            &mut buffer,
            libra_global_constants::OPERATOR_KEY,
        );
        write_ed25519_key(storage.as_ref(), &mut buffer, VALIDATOR_NETWORK_KEY);

        write_break(&mut buffer);
        writeln!(buffer, "Data").unwrap();
        write_break(&mut buffer);

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

        write_break(&mut buffer);

        if let Some(genesis_path) = self.genesis_path.as_ref() {
            compare_genesis(storage.as_ref(), &mut buffer, genesis_path)?;
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

fn write_ed25519_key(storage: &dyn Storage, buffer: &mut String, key: &'static str) {
    let value = ed25519_from_storage(key, storage).map_or_else(|e| e, |v| v.to_string());
    writeln!(buffer, "{} - {}", key, value).unwrap();
}

fn write_x25519_key(storage: &dyn Storage, buffer: &mut String, key: &'static str) {
    let value = ed25519_from_storage(key, storage).map_or_else(|e| e, |v| v.to_string());
    writeln!(buffer, "{} - {}", key, value).unwrap();
}

fn write_u64(storage: &dyn Storage, buffer: &mut String, key: &str) {
    let value = storage
        .get(key)
        .and_then(|c| c.value.u64())
        .map(|c| c.to_string())
        .unwrap_or_else(|e| e.to_string());
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
        .unwrap_or_else(|e| e.to_string());

    writeln!(buffer, "{} - {}", key, value).unwrap();
}

fn compare_genesis(
    storage: &dyn Storage,
    buffer: &mut String,
    genesis_path: &PathBuf,
) -> Result<(), Error> {
    let db_path = TempPath::new();
    let (db_rw, expected_waypoint) = compute_genesis(genesis_path, db_path.path())?;

    let actual_waypoint = storage
        .get(WAYPOINT)
        .and_then(|c| c.value.string())
        .map_err(|e| Error::LocalStorageReadError(WAYPOINT, e.to_string()))?;
    let actual_waypoint = Waypoint::from_str(&actual_waypoint).map_err(|e| {
        Error::UnexpectedError(format!("Unable to parse waypoint: {}", e.to_string()))
    })?;
    write_assert(
        buffer,
        libra_global_constants::WAYPOINT,
        actual_waypoint == expected_waypoint,
    );

    let validator_account = validator_account(storage)?;
    let validator_config = validator_config(validator_account, db_rw.reader.clone())?;

    let actual_consensus_key = ed25519_from_storage(CONSENSUS_KEY, storage)
        .map_err(|e| Error::LocalStorageReadError(CONSENSUS_KEY, e))?;
    let expected_consensus_key = validator_config.consensus_public_key;
    write_assert(
        buffer,
        CONSENSUS_KEY,
        actual_consensus_key == expected_consensus_key,
    );

    let actual_validator_key = x25519_from_storage(VALIDATOR_NETWORK_KEY, storage)
        .map_err(|e| Error::LocalStorageReadError(VALIDATOR_NETWORK_KEY, e))?;
    let expected_validator_key = validator_config.validator_network_identity_public_key;
    write_assert(
        buffer,
        VALIDATOR_NETWORK_KEY,
        actual_validator_key == expected_validator_key,
    );

    let actual_fullnode_key = x25519_from_storage(FULLNODE_NETWORK_KEY, storage)
        .map_err(|e| Error::LocalStorageReadError(FULLNODE_NETWORK_KEY, e))?;
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
        .ok_or_else(|| Error::UnexpectedError("Unable to find Validator account".into()))?;
    Ok(info.config().clone())
}

fn validator_account(storage: &dyn Storage) -> Result<AccountAddress, Error> {
    let key = storage
        .get_public_key(libra_global_constants::OPERATOR_KEY)
        .map_err(|e| {
            Error::LocalStorageReadError(libra_global_constants::OPERATOR_KEY, e.to_string())
        })?;
    Ok(account_address::from_public_key(&key.public_key))
}

fn ed25519_from_storage(
    key_name: &'static str,
    storage: &dyn Storage,
) -> Result<Ed25519PublicKey, String> {
    storage
        .get_public_key(key_name)
        .map(|v| v.public_key)
        .map_err(|e| e.to_string())
}

fn x25519_from_storage(
    key_name: &'static str,
    storage: &dyn Storage,
) -> Result<x25519::PublicKey, String> {
    let edkey = ed25519_from_storage(key_name, storage)?;
    x25519::PublicKey::from_ed25519_public_bytes(&edkey.to_bytes()).map_err(|e| e.to_string())
}
