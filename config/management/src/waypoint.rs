// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{config::ConfigPath, error::Error, secure_backend::ValidatorBackend};
use diem_global_constants::{GENESIS_WAYPOINT, WAYPOINT};
use diem_types::waypoint::Waypoint;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct InsertWaypoint {
    #[structopt(flatten)]
    pub config: ConfigPath,
    #[structopt(flatten)]
    validator_backend: ValidatorBackend,
    #[structopt(long)]
    waypoint: Waypoint,
    #[structopt(long, help = "Also set the genesis waypoint")]
    set_genesis: bool,
}

impl InsertWaypoint {
    pub fn execute(self) -> Result<(), Error> {
        let config = self
            .config
            .load()?
            .override_validator_backend(&self.validator_backend.validator_backend)?;
        let mut validator_storage = config.validator_backend();
        validator_storage.set(WAYPOINT, self.waypoint)?;
        if self.set_genesis {
            validator_storage.set(GENESIS_WAYPOINT, self.waypoint)?;
        }
        Ok(())
    }
}
