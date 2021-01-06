// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::storage::test_util::{
    arb_backups, arb_metadata_files, test_save_and_list_metadata_files_impl,
    test_write_and_read_impl,
};
use diem_temppath::TempPath;
use proptest::prelude::*;
use tokio::runtime::Runtime;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn test_write_and_read(
        backups in arb_backups()
    ) {
        let tmpdir = TempPath::new();
        tmpdir.create_as_dir().unwrap();
        let store = LocalFs::new(tmpdir.path().to_path_buf());

        let rt = Runtime::new().unwrap();
        rt.block_on(test_write_and_read_impl(Box::new(store), backups));
    }

    #[test]
    fn test_save_list_metadata_files(
        input in arb_metadata_files(),
    ) {
        let tmpdir = TempPath::new();
        tmpdir.create_as_dir().unwrap();
        let store = LocalFs::new(tmpdir.path().to_path_buf());

        let rt = Runtime::new().unwrap();
        rt.block_on(test_save_and_list_metadata_files_impl(Box::new(store), input));
    }
}
