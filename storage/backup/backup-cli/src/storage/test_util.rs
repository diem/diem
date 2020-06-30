// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::storage::{BackupStorage, ShellSafeName};
use libra_temppath::TempPath;
use proptest::{
    collection::{hash_map, vec},
    prelude::*,
};
use std::collections::HashMap;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

fn to_file_name(tmpdir: &TempPath, backup_name: &str, file_name: &str) -> String {
    tmpdir
        .path()
        .to_path_buf()
        .join(backup_name)
        .join(file_name)
        .into_os_string()
        .into_string()
        .unwrap()
}

pub async fn test_write_and_read_impl(
    store: Box<dyn BackupStorage>,
    tmpdir: &TempPath,
    backups: HashMap<ShellSafeName, HashMap<ShellSafeName, Vec<u8>>>,
) {
    for (backup_name, files) in &backups {
        let backup_handle = store.create_backup(backup_name).await.unwrap();
        assert_eq!(backup_handle, backup_name.as_ref());
        for (name, content) in files {
            let (handle, mut file) = store.create_for_write(&backup_handle, name).await.unwrap();
            assert_eq!(handle, to_file_name(&tmpdir, backup_name, name));
            file.write_all(content).await.unwrap();
        }
    }

    for (backup_name, files) in &backups {
        for (name, content) in files {
            let handle = to_file_name(&tmpdir, backup_name, name);
            let mut file = store.open_for_read(&handle).await.unwrap();
            let mut buf = Vec::new();
            file.read_to_end(&mut buf).await.unwrap();
            assert_eq!(content, &buf);
        }
    }
}

pub fn arb_backups(
) -> impl Strategy<Value = HashMap<ShellSafeName, HashMap<ShellSafeName, Vec<u8>>>> {
    hash_map(
        any::<ShellSafeName>(), // backup_name
        hash_map(
            any::<ShellSafeName>(),    // file name
            vec(any::<u8>(), 1..1000), // file content
            1..10,
        ),
        1..10,
    )
}
