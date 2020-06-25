// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod backup_service_client;
pub mod read_record_bytes;

#[cfg(test)]
pub mod test_utils;

use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
pub struct GlobalBackupOpt {
    #[structopt(long = "max-chunk-size", help = "Maximum chunk file size in bytes.")]
    pub max_chunk_size: usize,
}

#[derive(StructOpt)]
pub struct GlobalRestoreOpt {
    #[structopt(long = "target-db-dir", parse(from_os_str))]
    pub db_dir: PathBuf,
}
