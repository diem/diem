// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    adapter::{local_storage::LocalStorage, Adapter},
    FileHandle,
};
use anyhow::Result;
use byteorder::{LittleEndian, ReadBytesExt};
use futures::executor::block_on_stream;
use libra_crypto::HashValue;
use libra_types::{account_state_blob::AccountStateBlob, proof::SparseMerkleRangeProof};
use libradb::LibraDB;
use std::{io::Read, path::Path};

pub fn restore_account_state<P, I>(version: u64, root_hash: HashValue, db_dir: P, iter: I)
where
    P: AsRef<Path> + Clone,
    I: Iterator<Item = Result<(FileHandle, FileHandle)>>,
{
    let libradb =
        LibraDB::open(db_dir, false /* read_only */, None /* pruner */).expect("DB should open.");
    let chunk_and_proofs = iter.map(|res| {
        let (account_state_file, proof_file) = res.expect("Input iter yielded error.");
        let accounts = read_account_state_chunk(account_state_file)
            .expect("Failed to read account state file.");
        let proof = read_proof(proof_file).expect("Failed to read proof file.");

        (accounts, proof)
    });

    libradb
        .restore_account_state(chunk_and_proofs, version, root_hash)
        .expect("Failed to restore account state.");
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
