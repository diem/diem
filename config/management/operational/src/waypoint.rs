// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_global_constants::WAYPOINT;
use libra_management::{error::Error, secure_backend::ValidatorBackend, storage::StorageWrapper};
use libra_secure_storage::Value;
use libra_types::waypoint::Waypoint;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct InsertWaypoint {
    #[structopt(flatten)]
    validator_backend: ValidatorBackend,
    #[structopt(long)]
    waypoint: Waypoint,
}

impl InsertWaypoint {
    pub fn execute(self) -> Result<(), Error> {
        let mut validator_storage = StorageWrapper::new(
            self.validator_backend.name(),
            &self.validator_backend.validator_backend,
        )?;
        validator_storage.set(WAYPOINT, Value::String(self.waypoint.to_string()))?;
        Ok(())
    }
}
