// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{
    de::Deserializer,
    ser::{SerializeSeq, Serializer},
    Deserialize,
};

pub fn serialize<S>(data: &[Vec<u8>], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = serializer.serialize_seq(Some(data.len()))?;
    for e in data {
        seq.serialize_element(serde_bytes::Bytes::new(e.as_slice()))?;
    }
    seq.end()
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Vec<u8>>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(<Vec<serde_bytes::ByteBuf>>::deserialize(deserializer)?
        .into_iter()
        .map(serde_bytes::ByteBuf::into_vec)
        .collect())
}
