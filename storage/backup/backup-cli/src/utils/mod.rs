// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod backup_service_client;
pub mod read_record_bytes;
pub mod storage_ext;

#[cfg(test)]
pub mod test_utils;

use libra_types::transaction::Version;
use std::{mem::size_of, path::PathBuf};
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
    #[structopt(
        long = "target-version",
        default_value = "Version::max_value()",
        help = "Content newer than this version will not be recovered to DB."
    )]
    pub target_version: Version,
}

pub(crate) fn should_cut_chunk(chunk: &[u8], record: &[u8], max_chunk_size: usize) -> bool {
    !chunk.is_empty() && chunk.len() + record.len() + size_of::<u32>() > max_chunk_size
}
