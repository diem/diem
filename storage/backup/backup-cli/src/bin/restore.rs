// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use backup_cli::{
    adapter::{local_storage::LocalStorage, Adapter},
    FileHandle,
};
use byteorder::{LittleEndian, ReadBytesExt};
use futures::executor::block_on_stream;
use itertools::Itertools;
use libra_crypto::HashValue;
use libra_types::{account_state_blob::AccountStateBlob, proof::SparseMerkleRangeProof};
use libradb::LibraDB;
use std::{
    io::{BufRead, Read},
    path::PathBuf,
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(long, parse(from_os_str))]
    db_dir: PathBuf,
}

fn main() {
    let opt = Opt::from_args();

    let libradb = LibraDB::open(
        &opt.db_dir,
        false, /* read_only */
        None,  /* pruner */
    )
    .expect("DB should open.");

    let stdin = std::io::stdin();
    let mut iter = stdin.lock().lines();

    println!("Input version:");
    let version = iter
        .next()
        .expect("Must provide version.")
        .expect("Failed to read from stdin.")
        .parse::<u64>()
        .expect("Version must be valid u64.");
    println!("Version: {}", version);

    println!("Input state root hash:");
    let root_hash_hex = iter
        .next()
        .expect("Must provide state root hash.")
        .expect("Failed to read from stdin.");
    let root_hash = HashValue::from_slice(
        &hex::decode(&root_hash_hex).expect("State root hash must be valid hex."),
    )
    .expect("Invalid root hash.");
    println!("State root hash: {:x}", root_hash);

    let chunk_and_proofs = iter.tuples().map(|(file1, file2)| {
        let account_state_file = file1.expect("Failed to read from stdin.");
        let accounts = read_account_state_chunk(account_state_file)
            .expect("Failed to read account state file.");

        let proof_file = file2.expect("Failed to read from stdin");
        let proof = read_proof(proof_file).expect("Failed to read proof file.");

        (accounts, proof)
    });

    libradb
        .restore_account_state(chunk_and_proofs, version, root_hash)
        .expect("Failed to restore account state.");

    println!("Finished restoring account state.");
}

fn read_account_state_chunk(file: FileHandle) -> Result<Vec<(HashValue, AccountStateBlob)>> {
    let content = read_file(file)?;

    let mut chunk = vec![];
    let mut reader = std::io::Cursor::new(content);
    loop {
        let mut buf = [0u8; HashValue::LENGTH];
        if reader.read_exact(&mut buf).is_err() {
            break;
        }
        let key = HashValue::new(buf);

        let len = reader.read_u32::<LittleEndian>()?;
        let mut buf = vec![0u8; len as usize];
        reader.read_exact(&mut buf)?;
        let blob = AccountStateBlob::from(buf);

        chunk.push((key, blob));
    }

    Ok(chunk)
}

fn read_proof(file: FileHandle) -> Result<SparseMerkleRangeProof> {
    let content = read_file(file)?;
    let proof = lcs::from_bytes(&content)?;
    Ok(proof)
}

fn read_file(file: FileHandle) -> Result<Vec<u8>> {
    let mut content = vec![];
    for bytes_res in block_on_stream(LocalStorage::read_file_content(&file)) {
        content.extend(bytes_res?);
    }
    Ok(content)
}
