// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    error::Error,
    secure_backend::{
        StorageLocation,
        StorageLocation::{LocalStorage, RemoteStorage},
    },
    SecureBackends, SingleBackend,
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

/// Produces a waypoint from Genesis by either building it from a remote share or a local file. It
/// then computes the Waypoint and optionally inserts it into another storage.
#[derive(Debug, StructOpt)]
pub struct CreateWaypoint {
    #[structopt(flatten)]
    secure_backends: SecureBackends,
}

impl CreateWaypoint {
    pub fn execute(self) -> Result<Waypoint, Error> {
        let backend = self.secure_backends.local;
        let genesis_helper = crate::genesis::Genesis {
            backend: SingleBackend { backend },
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

        if let Some(remote) = self.secure_backends.remote {
            let mut remote_storage: Storage = remote.try_into()?;
            InsertWaypoint::insert_waypoint_to_backend(
                &waypoint,
                &mut remote_storage,
                StorageLocation::RemoteStorage,
            )?;
        }
        Ok(waypoint)
    }
}

#[derive(Debug, StructOpt)]
pub struct InsertWaypoint {
    #[structopt(flatten)]
    secure_backends: SecureBackends,

    #[structopt(long)]
    waypoint: Option<String>,
}

impl InsertWaypoint {
    pub fn execute(self) -> Result<Waypoint, Error> {
        if self.waypoint.is_some() && self.secure_backends.remote.is_some() {
            return Err(Error::CommandArgumentError(
                "only one of --waypoint and --remote can be provided".to_string(),
            ));
        }

        let waypoint_string = if let Some(waypoint_string) = self.waypoint {
            waypoint_string
        } else if let Some(remote_backend) = self.secure_backends.remote {
            let remote_storage = remote_backend.create_storage(RemoteStorage)?;
            remote_storage
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

        let mut local_storage = self.secure_backends.local.create_storage(LocalStorage)?;
        Self::insert_waypoint_to_backend(&waypoint, &mut local_storage, LocalStorage)?;
        Ok(waypoint)
    }

    fn insert_waypoint_to_backend(
        waypoint: &Waypoint,
        backend_storage: &mut Storage,
        backend_location: StorageLocation,
    ) -> Result<(), Error> {
        backend_storage
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
