// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use libra_temppath::TempPath;
use proptest::{
    collection::{hash_map, vec},
    prelude::*,
};
use std::{collections::HashMap, pin::Pin};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    runtime::Runtime,
};

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

async fn test_write_and_read_impl(backups: HashMap<String, HashMap<String, Vec<u8>>>) {
    let tmpdir = TempPath::new();
    tmpdir.create_as_dir().unwrap();
    let store = LocalFs::new(tmpdir.path().to_path_buf());

    for (backup_name, files) in &backups {
        let backup_handle = store.create_backup(backup_name).await.unwrap();
        assert_eq!(&backup_handle, backup_name);
        for (name, content) in files {
            let (handle, file) = store.create_for_write(&backup_handle, name).await.unwrap();
            assert_eq!(handle, to_file_name(&tmpdir, backup_name, name));
            Pin::new(file).write_all(content).await.unwrap();
        }
    }

    for (backup_name, files) in &backups {
        for (name, content) in files {
            let handle = to_file_name(&tmpdir, backup_name, name);
            let mut file = LocalFs::open_for_read(&handle).await.unwrap();
            let mut buf = Vec::new();
            file.read_to_end(&mut buf).await.unwrap();
            assert_eq!(content, &buf);
        }
    }
}

fn arb_file_name() -> impl Strategy<Value = String> {
    r"[-A-Za-z0-9_.]{1, 50}".prop_filter("no . and ..", |s| s != "." && s != "..")
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn test_write_and_read(
        backups in hash_map(
            arb_file_name(), // backup_name
            hash_map(
                arb_file_name(), // file name
                vec(any::<u8>(), 1..1000), // file content
                1..10
            ),
            1..10
        )
    ) {
        let mut rt = Runtime::new().unwrap();
        rt.block_on(test_write_and_read_impl(backups));
    }
}
