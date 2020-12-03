// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::storage::{
    command_adapter::config::Commands,
    test_util::{
        arb_backups, arb_metadata_files, test_save_and_list_metadata_files_impl,
        test_write_and_read_impl,
    },
};
use diem_temppath::TempPath;
use futures::Future;
use proptest::prelude::*;
use std::str::FromStr;
use tokio::runtime::Runtime;

fn get_store(tmpdir: &TempPath) -> Box<dyn BackupStorage> {
    tmpdir.create_as_dir().unwrap();
    let config = CommandAdapterConfig::load_from_str(
        &format!(r#"
                [[env_vars]]
                key = "FOLDER"
                value = "{}"

                [commands]
                create_backup = 'cd "$FOLDER" && mkdir $BACKUP_NAME && echo $BACKUP_NAME'
                create_for_write = 'cd "$FOLDER" && cd "$BACKUP_HANDLE" && test ! -f $FILE_NAME && touch $FILE_NAME && echo $BACKUP_HANDLE/$FILE_NAME && exec >&- && cat > $FILE_NAME'
                open_for_read = 'cat "$FOLDER/$FILE_HANDLE"'
                save_metadata_line= 'cd "$FOLDER" && mkdir -p metadata && cd metadata && cat > $FILE_NAME'
                list_metadata_files = 'cd "$FOLDER" && (test -d metadata && cd metadata && ls -1 || exec) | while read f; do echo metadata/$f; done'
            "#, tmpdir.path().to_str().unwrap()),
    ).unwrap();

    Box::new(CommandAdapter::new(config))
}

fn block_on<F: Future<Output = ()>>(f: F) {
    Runtime::new().unwrap().block_on(f)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn test_write_and_read(
        backups in arb_backups()
    ) {
        let tmpdir = TempPath::new();
        block_on(test_write_and_read_impl(get_store(&tmpdir), backups));
    }

    #[test]
    fn test_save_list_metadata_files(
        input in arb_metadata_files(),
    ) {
        let tmpdir = TempPath::new();
        block_on(test_save_and_list_metadata_files_impl(get_store(&tmpdir), input));
    }
}

fn dummy_store(cmd: &str) -> CommandAdapter {
    CommandAdapter::new(CommandAdapterConfig {
        commands: Commands {
            create_backup: cmd.to_string(),
            create_for_write: cmd.to_string(),
            open_for_read: cmd.to_string(),
            save_metadata_line: cmd.to_string(),
            list_metadata_files: cmd.to_string(),
        },
        env_vars: Vec::new(),
    })
}

async fn assert_commands_error(cmd: &str) {
    let name = ShellSafeName::from_str("name").unwrap();

    let store = dummy_store(cmd);

    // create_backup
    assert!(store
        .create_backup(&ShellSafeName::from_str("backup_name").unwrap())
        .await
        .is_err());

    let handle = "handle";

    // open_for_read
    let mut buf = String::new();
    assert!(store
        .open_for_read(&handle)
        .await
        .unwrap()
        .read_to_string(&mut buf)
        .await
        .is_err());

    // create_for_write
    assert!(async {
        let (_, mut file) = store.create_for_write(&handle, &name).await?;
        file.write_all(&[0; 1024]).await?;
        file.shutdown().await?;
        Result::<()>::Ok(())
    }
    .await
    .is_err());

    // save_metadata_line
    assert!(store
        .save_metadata_line(&name, &TextLine::new("1234").unwrap())
        .await
        .is_err());

    // list_metadata_files
    assert!(store.list_metadata_files().await.is_err());
}

async fn assert_commands_okay(cmd: &str) {
    let handle = "handle";
    let name = ShellSafeName::from_str("name").unwrap();

    let store = dummy_store(cmd);

    // create_backup
    assert_eq!(&store.create_backup(&name).await.unwrap(), "okay");

    // open_for_read
    let mut buf = String::new();
    store
        .open_for_read(&handle)
        .await
        .unwrap()
        .read_to_string(&mut buf)
        .await
        .unwrap();
    assert_eq!(&buf, "okay\n");

    // create_for_write
    let (out_handle, mut file) = store.create_for_write(&handle, &name).await.unwrap();
    assert_eq!(out_handle, "okay");
    file.write_all(&[0; 1024]).await.unwrap();
    file.shutdown().await.unwrap();

    // save_metadata_line
    store
        .save_metadata_line(&name, &TextLine::new("1234").unwrap())
        .await
        .unwrap();

    // list_metadata_files
    assert_eq!(store.list_metadata_files().await.unwrap(), vec!["okay"])
}

#[test]
fn test_unprocessed_error() {
    block_on(assert_commands_error(
        "echo okay; exec 1>&-; false; cat > /dev/null",
    ));

    // error processed
    block_on(assert_commands_okay(
        "echo okay; exec 1>&-; false && true; cat > /dev/null",
    ));
}

#[test]
fn test_unset_env_var() {
    std::env::remove_var("MYVAR2343u2");
    block_on(assert_commands_error(
        "echo $MYVAR2343u2 > /dev/null; echo okay; exec 1>&-; cat > /dev/null",
    ));

    // variable set
    std::env::set_var("MYVAR2343u2", "hehe");
    block_on(assert_commands_okay(
        "echo $MYVAR2343u2 > /dev/null; echo okay; exec 1>&-; cat > /dev/null",
    ));
}

/// Make sure errors in pipe processing are not ignored.
#[test]
fn test_pipe_fail() {
    // pipe fail on stdout processing
    block_on(assert_commands_error(
        "echo okay | (cat; false) | cat; exec 1>&-; cat | (cat; true) | cat > /dev/null",
    ));
    // pipe fail on stdin processing
    block_on(assert_commands_error(
        "echo okay | (cat; true) | cat; exec 1>&-; cat | (cat; false) | cat > /dev/null",
    ));

    // no error in pipelines
    block_on(assert_commands_okay(
        "echo okay | (cat; true) | cat; exec 1>&-; cat | (cat; true) | cat > /dev/null",
    ));
}
