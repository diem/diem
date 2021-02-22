// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_crypto::{ed25519::Ed25519PublicKey, x25519};

use diem_management::{config::ConfigPath, error::Error, secure_backend::ValidatorBackend};
use diem_types::{account_address::AccountAddress, waypoint::Waypoint};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct PrintAccount {
    #[structopt(flatten)]
    config: ConfigPath,
    /// The account name in storage
    #[structopt(long)]
    account_name: String,
    #[structopt(flatten)]
    validator_backend: ValidatorBackend,
}

impl PrintAccount {
    pub fn execute(self) -> Result<AccountAddress, Error> {
        let config = self
            .config
            .load()?
            .override_validator_backend(&self.validator_backend.validator_backend)?;

        let storage = config.validator_backend();
        let account_name = Box::leak(self.account_name.into_boxed_str());
        storage.account_address(account_name)
    }
}

#[derive(Debug, StructOpt)]
pub struct PrintKey {
    #[structopt(flatten)]
    config: ConfigPath,
    /// The key name in storage
    #[structopt(long)]
    key_name: String,
    #[structopt(flatten)]
    validator_backend: ValidatorBackend,
}

impl PrintKey {
    pub fn execute(self) -> Result<Ed25519PublicKey, Error> {
        let config = self
            .config
            .load()?
            .override_validator_backend(&self.validator_backend.validator_backend)?;

        let storage = config.validator_backend();
        let key_name = Box::leak(self.key_name.into_boxed_str());
        storage.ed25519_public_from_private(key_name)
    }
}

#[derive(Debug, StructOpt)]
pub struct PrintXKey {
    #[structopt(flatten)]
    config: ConfigPath,
    /// The key name in storage
    #[structopt(long)]
    key_name: String,
    #[structopt(flatten)]
    validator_backend: ValidatorBackend,
}

impl PrintXKey {
    pub fn execute(self) -> Result<x25519::PublicKey, Error> {
        let config = self
            .config
            .load()?
            .override_validator_backend(&self.validator_backend.validator_backend)?;

        let storage = config.validator_backend();
        let key_name = Box::leak(self.key_name.into_boxed_str());
        storage.x25519_public_from_private(key_name)
    }
}

#[derive(Debug, StructOpt)]
pub struct PrintWaypoint {
    #[structopt(flatten)]
    config: ConfigPath,
    /// The waypoint name in storage
    #[structopt(long)]
    waypoint_name: String,
    #[structopt(flatten)]
    validator_backend: ValidatorBackend,
}

impl PrintWaypoint {
    pub fn execute(self) -> Result<Waypoint, Error> {
        let config = self
            .config
            .load()?
            .override_validator_backend(&self.validator_backend.validator_backend)?;

        let storage = config.validator_backend();
        let waypoint_name = Box::leak(self.waypoint_name.into_boxed_str());
        storage.waypoint(waypoint_name)
    }
}
