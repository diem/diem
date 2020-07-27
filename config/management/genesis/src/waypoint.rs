// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use executor::db_bootstrapper;
use libra_global_constants::WAYPOINT;
use libra_management::{
    config::ConfigPath,
    error::Error,
    secure_backend::{SharedBackend, ValidatorBackend},
};
use libra_secure_storage::Value;
use libra_temppath::TempPath;
use libra_types::waypoint::Waypoint;
use libra_vm::LibraVM;
use libradb::LibraDB;
use storage_interface::DbReaderWriter;
use structopt::StructOpt;

/// Produces a waypoint from Genesis from the shared storage. It then computes the Waypoint and
/// optionally inserts it into another storage, typically the validator storage.
#[derive(Debug, StructOpt)]
pub struct CreateWaypoint {
    #[structopt(flatten)]
    config: ConfigPath,
    #[structopt(flatten)]
    shared_backend: SharedBackend,
}

impl CreateWaypoint {
    pub fn execute(self) -> Result<Waypoint, Error> {
        let genesis_helper = crate::genesis::Genesis {
            config: self.config,
            backend: self.shared_backend,
            path: None,
        };

        let genesis = genesis_helper.execute()?;

        let path = TempPath::new();
        let libradb =
            LibraDB::open(&path, false, None).map_err(|e| Error::UnexpectedError(e.to_string()))?;
        let db_rw = DbReaderWriter::new(libradb);

        db_bootstrapper::bootstrap_db_if_empty::<LibraVM>(&db_rw, &genesis)
            .map_err(|e| Error::UnexpectedError(e.to_string()))?
            .ok_or_else(|| Error::UnexpectedError("Unable to generate a waypoint".to_string()))
    }
}

#[derive(Debug, StructOpt)]
pub struct CreateAndInsertWaypoint {
    #[structopt(flatten)]
    config: ConfigPath,
    #[structopt(flatten)]
    shared_backend: SharedBackend,
    #[structopt(flatten)]
    validator_backend: ValidatorBackend,
}

impl CreateAndInsertWaypoint {
    pub fn execute(self) -> Result<Waypoint, Error> {
        let waypoint = CreateWaypoint {
            config: self.config.clone(),
            shared_backend: self.shared_backend,
        }
        .execute()?;

        let config = self
            .config
            .load()?
            .override_validator_backend(&self.validator_backend.validator_backend)?;
        let mut validator_storage = config.validator_backend();
        validator_storage.set(WAYPOINT, Value::String(waypoint.to_string()))?;
        Ok(waypoint)
    }
}
