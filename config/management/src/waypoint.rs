// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    error::Error,
    secure_backend::{
        OptionalSharedBackend, OptionalValidatorBackend, SharedBackend, ValidatorBackend,
    },
    storage::StorageWrapper,
};
use executor::db_bootstrapper;
use libra_global_constants::WAYPOINT;
use libra_secure_storage::Value;
use libra_temppath::TempPath;
use libra_types::waypoint::Waypoint;
use libra_vm::LibraVM;
use libradb::LibraDB;
use std::str::FromStr;
use storage_interface::DbReaderWriter;
use structopt::StructOpt;

/// Produces a waypoint from Genesis from the shared storage. It then computes the Waypoint and
/// optionally inserts it into another storage, typically the validator storage.
#[derive(Debug, StructOpt)]
pub struct CreateWaypoint {
    #[structopt(flatten)]
    shared_backend: SharedBackend,
    #[structopt(flatten)]
    validator_backend: OptionalValidatorBackend,
}

impl CreateWaypoint {
    pub fn execute(self) -> Result<Waypoint, Error> {
        let genesis_helper = crate::genesis::Genesis {
            backend: self.shared_backend,
            path: None,
        };

        let genesis = genesis_helper.execute()?;

        let path = TempPath::new();
        let libradb =
            LibraDB::open(&path, false, None).map_err(|e| Error::UnexpectedError(e.to_string()))?;
        let db_rw = DbReaderWriter::new(libradb);

        let waypoint = db_bootstrapper::bootstrap_db_if_empty::<LibraVM>(&db_rw, &genesis)
            .map_err(|e| Error::UnexpectedError(e.to_string()))?
            .ok_or_else(|| Error::UnexpectedError("Unable to generate a waypoint".to_string()))?;

        if let Some(validator_backend_config) = &self.validator_backend.validator_backend {
            let storage_name = self.validator_backend.name();
            let mut validator_storage =
                StorageWrapper::new(storage_name, &validator_backend_config)?;
            validator_storage.set(WAYPOINT, Value::String(waypoint.to_string()))?;
        }
        Ok(waypoint)
    }
}

#[derive(Debug, StructOpt)]
pub struct InsertWaypoint {
    #[structopt(flatten)]
    validator_backend: ValidatorBackend,
    #[structopt(flatten)]
    shared_backend: OptionalSharedBackend,
    #[structopt(long)]
    waypoint: Option<String>,
}

impl InsertWaypoint {
    pub fn execute(self) -> Result<Waypoint, Error> {
        if self.waypoint.is_some() && self.shared_backend.shared_backend.is_some() {
            return Err(Error::CommandArgumentError(
                "only one of --waypoint and --remote can be provided".to_string(),
            ));
        }

        // Retrieve waypoint from args or storage
        let waypoint_string = if let Some(waypoint_string) = self.waypoint {
            waypoint_string
        } else if let Some(shared_backend_config) = &self.shared_backend.shared_backend {
            let storage_name = self.shared_backend.name();
            let shared_storage = StorageWrapper::new(storage_name, shared_backend_config)?;
            shared_storage.string(WAYPOINT)?
        } else {
            return Err(Error::CommandArgumentError(
                "please provide either --waypoint or --remote".to_string(),
            ));
        };

        let waypoint = Waypoint::from_str(&waypoint_string)
            .map_err(|e| Error::UnexpectedError(e.to_string()))?;

        // Insert waypoint to backend
        let mut validator_storage = StorageWrapper::new(
            self.validator_backend.name(),
            &self.validator_backend.validator_backend,
        )?;
        validator_storage.set(WAYPOINT, Value::String(waypoint.to_string()))?;
        Ok(waypoint)
    }
}
