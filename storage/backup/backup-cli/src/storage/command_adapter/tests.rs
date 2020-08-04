// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::storage::test_util::{
    arb_backups, arb_metadata_files, test_save_and_list_metadata_files_impl,
    test_write_and_read_impl,
};
use libra_temppath::TempPath;
use proptest::prelude::*;
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
                create_for_write = 'cd "$FOLDER" && cd "$BACKUP_HANDLE" && test ! -f $FILE_NAME && touch $FILE_NAME && echo `pwd`/$FILE_NAME && exec >&- && cat > $FILE_NAME'
                open_for_read = 'cat "$FILE_HANDLE"'
                save_metadata_line= 'cd "$FOLDER" && mkdir -p metadata && cd metadata && cat > $FILE_NAME'
                list_metadata_files = 'cd "$FOLDER" && (test -d metadata && cd metadata && ls -1 || exec) | while read f; do echo `pwd`/metadata/$f; done'
            "#, tmpdir.path().to_str().unwrap()),
    ).unwrap();

    Box::new(CommandAdapter::new(config))
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn test_write_and_read(
        backups in arb_backups()
    ) {
        let mut rt = Runtime::new().unwrap();
        let tmpdir = TempPath::new();

        rt.block_on(test_write_and_read_impl(get_store(&tmpdir), &tmpdir, backups));
    }

    #[test]
    fn test_save_list_metadata_files(
        input in arb_metadata_files(),
    ) {
        let tmpdir = TempPath::new();
        let mut rt = Runtime::new().unwrap();

        rt.block_on(test_save_and_list_metadata_files_impl(get_store(&tmpdir), input, &tmpdir.path().to_path_buf()));

    }
}
