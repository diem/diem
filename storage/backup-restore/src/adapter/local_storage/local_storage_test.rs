// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use futures::executor::{block_on, block_on_stream};
use proptest::{collection::vec, prelude::*};

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn test_local_storage(contents in vec(vec(any::<u8>(), 1..1000), 1..10)) {
        let tmpdir = tempfile::tempdir().unwrap();
        let adapter = LocalStorage::new(tmpdir.path().to_path_buf());

        let file_handles: Vec<_> = contents.iter().map(|content| {
            let iter = content.chunks(10).map(|c| c.to_vec());
            let stream = futures::stream::iter(iter);
            block_on(adapter.write_new_file(stream)).unwrap()
        }).collect();

        for (handle, expected_content) in itertools::zip_eq(file_handles, contents) {
            let mut actual_content = vec![];
            for res in block_on_stream(LocalStorage::read_file_content(&handle)) {
                let bytes = res.unwrap();
                actual_content.extend_from_slice(&bytes);
            }
            prop_assert_eq!(actual_content, expected_content);
        }
    }
}
