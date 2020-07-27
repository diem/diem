// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_global_constants::WAYPOINT;
use libra_management::{config::ConfigPath, error::Error, secure_backend::ValidatorBackend};
use libra_secure_storage::Value;
use libra_types::waypoint::Waypoint;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct InsertWaypoint {
    #[structopt(flatten)]
    pub config: ConfigPath,
    #[structopt(flatten)]
    validator_backend: ValidatorBackend,
    #[structopt(long)]
    waypoint: Waypoint,
}

impl InsertWaypoint {
    pub fn execute(self) -> Result<(), Error> {
        let config = self
            .config
            .load()?
            .override_validator_backend(&self.validator_backend.validator_backend)?;
        let mut validator_storage = config.validator_backend();
        validator_storage.set(WAYPOINT, Value::String(self.waypoint.to_string()))?;
        Ok(())
    }
}
