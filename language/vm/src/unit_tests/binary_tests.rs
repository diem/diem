// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::file_format_common::*;
use proptest::prelude::*;

#[test]
fn binary_len() {
    let mut binary_data = BinaryData::new();
    for _ in 0..100 {
        binary_data.push(1).unwrap();
    }
    assert_eq!(binary_data.len(), 100);
}

proptest! {
    #[test]
    fn vec_to_binary(vec in any::<Vec<u8>>()) {
        let binary_data = BinaryData::from(vec.clone());
        let vec2 = binary_data.into_inner();
        assert_eq!(vec.len(), vec2.len());
    }
}

proptest! {
    #[test]
    fn binary_push(item in any::<u8>()) {
        let mut binary_data = BinaryData::new();
        binary_data.push(item).unwrap();
        assert_eq!(binary_data.into_inner()[0], item);
    }
}

proptest! {
    #[test]
    fn binary_extend(vec in any::<Vec<u8>>()) {
        let mut binary_data = BinaryData::new();
        binary_data.extend(&vec).unwrap();
        assert_eq!(binary_data.len(), vec.len());
        for (index, item) in vec.iter().enumerate() {
            assert_eq!(*item, binary_data.as_inner()[index]);
        }
    }
}
