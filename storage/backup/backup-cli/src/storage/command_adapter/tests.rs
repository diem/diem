// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::storage::test_util::{arb_backups, test_write_and_read_impl};
use libra_temppath::TempPath;
use proptest::prelude::*;
use tokio::runtime::Runtime;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn test_write_and_read(
        backups in arb_backups()
    ) {
        let mut rt = Runtime::new().unwrap();
        let tmpdir = TempPath::new();
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
            "#, tmpdir.path().to_str().unwrap()),
        ).unwrap();

        let store = CommandAdapter::new(config);
        rt.block_on(test_write_and_read_impl(Box::new(store), &tmpdir, backups));
    }
}
