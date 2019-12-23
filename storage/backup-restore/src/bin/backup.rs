// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use backup_restore::{adapter::local_storage::LocalStorage, backup_account_state};
use std::path::PathBuf;
use storage_client::{StorageRead, StorageReadServiceClient};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// Default 4M.
    #[structopt(long, default_value = "4194304")]
    state_chunk_size: usize,

    /// Where the backup is stored.
    #[structopt(long, parse(from_os_str))]
    local_dir: PathBuf,

    /// The port of the storage service.
    #[structopt(long)]
    node_port: u16,
}

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();

    let client = StorageReadServiceClient::new("localhost", opt.node_port);

    let (version, state_root_hash) = client
        .get_latest_state_root_async()
        .await
        .expect("Failed to get latest version and state root hash.");
    println!("Latest version: {}", version);
    println!("State root hash: {:x}", state_root_hash);

    let adapter = LocalStorage::new(opt.local_dir);
    let file_handles = backup_account_state(&client, version, &adapter, opt.state_chunk_size)
        .await
        .expect("Failed to backup account state.");

    for (account_state_file, proof_file) in file_handles {
        println!("{}", account_state_file);
        println!("{}", proof_file);
    }
}
