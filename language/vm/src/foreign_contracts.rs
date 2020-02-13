// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This file contains models of the vm crate's dependencies for use with MIRAI.

pub mod types {
    pub mod transaction {
        pub const MAX_TRANSACTION_SIZE_IN_BYTES: usize = 4096;
    }
    pub mod byte_array {
        pub struct ByteArray(Vec<u8>);
        impl ByteArray {
            pub fn as_bytes(_self: &ByteArray) -> &[u8] {
                &_self.0
            }
            pub fn new(buf: Vec<u8>) -> Self {
                ByteArray(buf)
            }

            pub fn len(_self: &ByteArray) -> usize {
                _self.0.len()
            }

            pub fn is_empty(_self: &ByteArray) -> bool {
                _self.0.len() == 0
            }
        }
    }
}
