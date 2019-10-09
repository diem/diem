// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

const ARRAY_MAX_LENGTH: usize = i32::max_value() as usize;
type Endianness = byteorder::LittleEndian;

mod canonical_deserialize;
mod canonical_serialize;
mod simple_deserializer;
mod simple_serializer;
pub mod test_helper;

pub use canonical_deserialize::{CanonicalDeserialize, CanonicalDeserializer};
pub use canonical_serialize::{CanonicalSerialize, CanonicalSerializer};
pub use simple_deserializer::SimpleDeserializer;
pub use simple_serializer::SimpleSerializer;

#[cfg(test)]
mod canonical_serialization_test;

/// Commonly used function to ensure some length is <= ARRAY_MAX_LENGTH
#[macro_export]
macro_rules! ensure_max_length {
    ($len:expr) => {
        failure::ensure!(
            $len <= crate::ARRAY_MAX_LENGTH,
            "collection/array length exceeded the maximum length limit. \
             length: {}, Max length limit: {}",
            $len,
            crate::ARRAY_MAX_LENGTH,
        );
    };
}
