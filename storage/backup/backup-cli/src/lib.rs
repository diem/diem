// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod adapter;

use crate::adapter::{local_storage::LocalStorage, Adapter};
use anyhow::Result;
use byteorder::{LittleEndian, ReadBytesExt};
use futures::{executor::block_on_stream, stream, StreamExt};
use libra_crypto::HashValue;
use libra_types::{
    account_state_blob::AccountStateBlob, proof::SparseMerkleRangeProof, transaction::Version,
};
use libradb::LibraDB;
use std::{io::Read, path::Path};
use storage_client::{StorageRead, StorageReadServiceClient};

pub type FileHandle = String;

pub async fn backup_account_state(
    client: &StorageReadServiceClient,
    version: Version,
    adapter: &impl Adapter,
    max_chunk_size: usize,
) -> Result<Vec<(FileHandle, FileHandle)>> {
    let mut chunk = vec![];
    let mut ret = vec![];

    let mut account_stream = client.backup_account_state(version).await?;
    let mut prev_key = None;
    while let Some(resp) = account_stream.next().await.transpose()? {
        let key = resp.account_key;
        let blob = resp.account_state_blob;
        println!("Backing up key: {:x}", key);

        let mut bytes = key.to_vec();
        let blob: Vec<u8> = blob.into();
        assert!(blob.len() <= std::u32::MAX as usize);
        let blob_len = blob.len() as u32;
        bytes.extend_from_slice(&blob_len.to_le_bytes());
        bytes.extend(blob);
        assert!(bytes.len() <= max_chunk_size);

        if chunk.len() + bytes.len() > max_chunk_size {
            assert!(chunk.len() <= max_chunk_size);
            let account_state_file = adapter
                .write_new_file(stream::once(async move { chunk }))
                .await?;

            let prev_key = prev_key.expect("max_chunk_size should be larger than account size.");
            println!(
                "Reached max_chunk_size. Asking proof for key: {:?}",
                prev_key,
            );
            let proof_file = get_proof_and_write(client, adapter, prev_key, version).await?;
            ret.push((account_state_file, proof_file));
            chunk = vec![];
        }

        chunk.extend(bytes);
        prev_key = Some(key);
    }

    assert!(!chunk.is_empty());
    assert!(chunk.len() <= max_chunk_size);
    let account_state_file = adapter
        .write_new_file(stream::once(async move { chunk }))
        .await?;

    let prev_key = prev_key.expect("Should have at least one account.");
    println!("Asking proof for last key: {:x}", prev_key);
    let proof_file = get_proof_and_write(client, adapter, prev_key, version).await?;
    ret.push((account_state_file, proof_file));

    Ok(ret)
}

async fn get_proof_and_write(
    client: &StorageReadServiceClient,
    adapter: &impl Adapter,
    key: HashValue,
    version: Version,
) -> Result<FileHandle> {
    let proof = client.get_account_state_range_proof(key, version).await?;
    let proof_bytes: Vec<u8> = lcs::to_bytes(&proof)?;
    let file = adapter
        .write_new_file(stream::once(async move { proof_bytes }))
        .await?;
    Ok(file)
}

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

#[cfg(test)]
mod test {
    use crate::{
        adapter::local_storage::LocalStorage, backup_account_state, restore_account_state,
    };
    use libra_config::config::NodeConfig;
    use libra_proptest_helpers::ValueGenerator;
    use libra_temppath::TempPath;
    use libra_types::transaction::PRE_GENESIS_VERSION;
    use libradb::{test_helper::arb_blocks_to_commit, LibraDB};
    use std::sync::Arc;
    use storage_client::StorageReadServiceClient;
    use storage_interface::{DbReader, DbWriter};
    use storage_service::start_storage_service_with_db;

    fn tmp_db_empty() -> (TempPath, LibraDB) {
        let tmpdir = TempPath::new();
        let db = LibraDB::new_for_test(&tmpdir);

        (tmpdir, db)
    }

    fn tmp_db_with_random_content() -> (TempPath, LibraDB) {
        let (tmpdir, db) = tmp_db_empty();
        let mut cur_ver = 0;
        for (txns_to_commit, ledger_info_with_sigs) in
            ValueGenerator::new().generate(arb_blocks_to_commit())
        {
            db.save_transactions(
                &txns_to_commit,
                cur_ver, /* first_version */
                Some(&ledger_info_with_sigs),
            )
            .unwrap();
            cur_ver += txns_to_commit.len() as u64;
        }

        (tmpdir, db)
    }

    #[test]
    fn end_to_end() {
        let (_src_db_dir, src_db) = tmp_db_with_random_content();
        let src_db = Arc::new(src_db);
        let (latest_version, state_root_hash) = src_db.get_latest_state_root().unwrap();
        let tgt_db_dir = TempPath::new();
        let backup_dir = TempPath::new();
        backup_dir.create_as_dir().unwrap();
        let adaptor = LocalStorage::new(backup_dir.path().to_path_buf());

        let config = NodeConfig::random();
        let mut rt = start_storage_service_with_db(&config, Arc::clone(&src_db));
        let client = StorageReadServiceClient::new(&config.storage.address);
        let handles = rt
            .block_on(backup_account_state(
                &client,
                latest_version,
                &adaptor,
                1024 * 1024,
            ))
            .unwrap();

        restore_account_state(
            PRE_GENESIS_VERSION,
            state_root_hash,
            &tgt_db_dir,
            handles.into_iter().map(Ok),
        );
        let tgt_db = LibraDB::open(
            &tgt_db_dir,
            true, /* readonly */
            None, /* pruner */
        )
        .unwrap();
        assert_eq!(
            tgt_db
                .get_latest_tree_state()
                .unwrap()
                .account_state_root_hash,
            state_root_hash,
        );
    }
}
