// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    error::Error,
    secure_backend::{
        OptionalSharedBackend, OptionalValidatorBackend, SharedBackend, StorageLocation,
        StorageLocation::{LocalStorage, RemoteStorage},
        ValidatorBackend,
    },
};
use executor::db_bootstrapper;
use libra_global_constants::WAYPOINT;
use libra_secure_storage::{KVStorage, Storage, Value};
use libra_temppath::TempPath;
use libra_types::waypoint::Waypoint;
use libra_vm::LibraVM;
use libradb::LibraDB;
use std::{convert::TryInto, str::FromStr};
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

        if let Some(validator_backend) = self.validator_backend.validator_backend {
            let mut validator_storage: Storage = validator_backend.try_into()?;
            InsertWaypoint::insert_waypoint_to_backend(
                &waypoint,
                &mut validator_storage,
                StorageLocation::RemoteStorage,
            )?;
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

        let waypoint_string = if let Some(waypoint_string) = self.waypoint {
            waypoint_string
        } else if let Some(shared_backend) = self.shared_backend.shared_backend {
            let shared_storage = shared_backend.create_storage(RemoteStorage)?;
            shared_storage
                .get(WAYPOINT)
                .and_then(|v| v.value.string())
                .map_err(|e| Error::RemoteStorageReadError(WAYPOINT, e.to_string()))?
        } else {
            return Err(Error::CommandArgumentError(
                "please provide either --waypoint or --remote".to_string(),
            ));
        };

        let waypoint = Waypoint::from_str(&waypoint_string)
            .map_err(|e| Error::UnexpectedError(e.to_string()))?;

        let mut validator_storage = self
            .validator_backend
            .validator_backend
            .create_storage(LocalStorage)?;
        Self::insert_waypoint_to_backend(&waypoint, &mut validator_storage, LocalStorage)?;
        Ok(waypoint)
    }

    fn insert_waypoint_to_backend(
        waypoint: &Waypoint,
        storage: &mut Storage,
        backend_location: StorageLocation,
    ) -> Result<(), Error> {
        storage
            .set(
                libra_global_constants::WAYPOINT,
                Value::String(waypoint.to_string()),
            )
            .map_err(|e| match backend_location {
                LocalStorage => {
                    Error::LocalStorageWriteError(libra_global_constants::WAYPOINT, e.to_string())
                }
                RemoteStorage => {
                    Error::RemoteStorageWriteError(libra_global_constants::WAYPOINT, e.to_string())
                }
            })?;
        Ok(())
    }
}
