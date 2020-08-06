// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    storage::{BackupStorage, ShellSafeName, TextLine},
    utils::PathToString,
};
use anyhow::Result;
use itertools::Itertools;
use proptest::{
    collection::{hash_map, vec},
    prelude::*,
};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::Stdio,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    time::{delay_for, Duration},
};

fn to_file_name(backup_name: &str, file_name: &str) -> String {
    Path::new(backup_name)
        .join(file_name)
        .path_to_string()
        .unwrap()
}

pub async fn test_write_and_read_impl(
    store: Box<dyn BackupStorage>,
    backups: HashMap<ShellSafeName, HashMap<ShellSafeName, Vec<u8>>>,
) {
    for (backup_name, files) in &backups {
        let backup_handle = store.create_backup(backup_name).await.unwrap();
        assert_eq!(backup_handle, backup_name.as_ref());
        for (name, content) in files {
            let (handle, mut file) = store.create_for_write(&backup_handle, name).await.unwrap();
            assert_eq!(handle, to_file_name(backup_name, name));
            file.write_all(content).await.unwrap();
        }
    }

    for (backup_name, files) in &backups {
        for (name, content) in files {
            let handle = to_file_name(backup_name, name);
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

pub async fn test_save_and_list_metadata_files_impl(
    store: Box<dyn BackupStorage>,
    input: Vec<(ShellSafeName, TextLine)>,
    path: &PathBuf,
) {
    for (name, content) in &input {
        store.save_metadata_line(name, &content).await.unwrap();
    }

    // It takes a little time for the ls command to reflect newly created entries.
    // it's not a problem in real world.
    wait_for_dentries(path, input.len()).await;

    let mut read_back = Vec::new();
    for file_handle in store.list_metadata_files().await.unwrap() {
        let mut buf = String::new();
        store
            .open_for_read(&file_handle)
            .await
            .unwrap()
            .read_to_string(&mut buf)
            .await
            .unwrap();
        read_back.extend(
            buf.lines()
                .map(TextLine::new)
                .collect::<Result<Vec<_>>>()
                .unwrap()
                .into_iter(),
        )
    }
    read_back.sort();

    let expected = input
        .into_iter()
        .map(|(_name, content)| content)
        .sorted()
        .collect::<Vec<_>>();

    assert_eq!(read_back, expected)
}

pub fn arb_metadata_files() -> impl Strategy<Value = Vec<(ShellSafeName, TextLine)>> {
    vec(any::<(ShellSafeName, TextLine)>(), 0..10)
}

async fn wait_for_dentries(path: &PathBuf, num_of_files: usize) {
    // sync
    tokio::process::Command::new("sync")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap()
        .await
        .unwrap();

    // try every 10ms, for 10 seconds at most
    for n in 1..=1000usize {
        let output = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(&format!(
                "ls -1 {} || exec",
                path.join("metadata")
                    .into_os_string()
                    .into_string()
                    .unwrap()
            ))
            .stdin(Stdio::null())
            .output()
            .await
            .unwrap();
        let got_files = String::from_utf8(output.stdout).unwrap().lines().count();
        if got_files >= num_of_files {
            return;
        } else {
            println!(
                "Got {} files on {}-th try, expecting {}.",
                got_files, n, num_of_files
            );
        }
        delay_for(Duration::from_millis(10)).await;
    }

    panic!("ls result never contained {} entries", num_of_files);
}
