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
        let tmpdir = TempPath::new();
        tmpdir.create_as_dir().unwrap();
        let store = LocalFs::new(tmpdir.path().to_path_buf());

        let mut rt = Runtime::new().unwrap();
        rt.block_on(test_write_and_read_impl(Box::new(store), &tmpdir, backups));
    }
}
