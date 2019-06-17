// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Handle the customized and optimized serialization and deserialization of node types. We
//! customize ser/de for explicit specification and space optimization.

#[cfg(test)]
mod node_serde_test;

use crate::{
    nibble_path::NibblePath,
    node_type::{BranchNode, ExtensionNode, LeafNode},
};
use crypto::HashValue;
use serde::{
    de::{self, Deserialize, Deserializer, SeqAccess, Visitor},
    ser::{self, Serialize, SerializeTuple, Serializer},
};
use std::{collections::hash_map::HashMap, fmt, result::Result};

/// Customized BranchNode serialization/deserialization
///
/// A branch node will be serialized to 2 u16 bitmaps and a vector of hashes. The first bitmap
/// indicates which children exist by setting the bit at its corresponding index; the second bitmap
/// indicates which children are leaf nodes in the same way; finally a vector of hashes of children
/// follows in order of index starting at the beginning.  For example, if a branch node has 3
/// children, a leaf node, an extension node, and a branch node, with indices 0, 7, 12,
/// respectively. The serialization structure will be:
/// 1st field: `0b0001000010000001` (LSB denotes the child at index 0)
/// 2nd field: `0b0000000000000001` (LSB denotes the child at index 0)
/// 3rd field: vec![hash1, hash2, hash3]
impl Serialize for BranchNode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let (child_bitmap, leaf_bitmap) = self.generate_bitmaps();
        let mut hashes = Vec::with_capacity(self.num_children());
        for i in 0..16 {
            // If a child exists, append it to the vector.  Note: Don't have to fetch node
            // type info (leaf or not) since leaf_bitmap is born for this.
            if child_bitmap >> i & 1 != 0 {
                hashes.push(self.child(i as u8).ok_or_else(|| {
                    ser::Error::custom(format!(
                        "Invalid branch node: \
                         unable to get child {} for BranchNode serialization.",
                        i
                    ))
                })?);
            }
        }
        let mut tuple = serializer.serialize_tuple(3)?;
        tuple.serialize_element(&child_bitmap)?;
        tuple.serialize_element(&leaf_bitmap)?;
        tuple.serialize_element(&hashes)?;
        tuple.end()
    }
}

struct BranchNodeVisitor;

impl<'de> Visitor<'de> for BranchNodeVisitor {
    type Value = BranchNode;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "This visitor expects to receive two u16 bitmaps, \
             followed by a vector of HashValues(up to 16)"
        )
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let child_bitmap: u16 = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
        let leaf_bitmap: u16 = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(1, &self))?;
        if child_bitmap | leaf_bitmap != child_bitmap {
            Err(de::Error::custom(
                "Invalid branch node in deserialization: \
                 leaf_bitmap conflicts with child_bitmap",
            ))?;
        }
        let hashes: Vec<HashValue> = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(2, &self))?;

        if child_bitmap.count_ones() as usize != hashes.len() {
            Err(de::Error::custom(
                "Invalid branch node in deserialization: \
                 children number doesn't match child_bitmap",
            ))?;
        }
        let children = (0..16)
            .filter(|i| child_bitmap >> i & 1 == 1)
            .zip(hashes.into_iter())
            .map(|(i, d)| (i, (d, leaf_bitmap >> i & 1 == 1)))
            .collect::<HashMap<_, _>>();
        Ok(BranchNode::new(children))
    }
}

impl<'de> Deserialize<'de> for BranchNode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_tuple(3, BranchNodeVisitor)
    }
}

/// Customized ExtensionNode serialization/deserialization
///
/// An extension node will be serialized into a vector of bytes and a hash value.  The vector of
/// bytes represents the encoded_path of this extension node. If the encoded path has an even number
/// of nibbles, we append 0x01 at the end. Otherwise we set the last unused nibble to be 0x0, since
/// in the previous case the last nibble is always 0x1. After the byte vector, we store the
/// hash value of the only child node of the current extension node.
///
/// For example, if a branch node has an encoded_path of 'a13' with hash1 as its child node hash,
/// the serialization structure will be:
/// 1st field: vec![0xa1, 0x30]
/// 2nd field: hash1
impl Serialize for ExtensionNode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let nibble_path = self.nibble_path();
        if nibble_path.num_nibbles() == 0 {
            return Err(ser::Error::custom("Encoded nibble path bytes is empty"));
        }
        let mut nibble_bytes = Vec::with_capacity(nibble_path.num_nibbles() / 2 + 1);
        nibble_bytes.extend(self.nibble_path().bytes());
        if nibble_path.num_nibbles() % 2 == 1 {
            // Set the last nibble to 0x0 to be the flag when odd.
            *nibble_bytes
                .last_mut()
                .expect("Have verified encoded path bytes is not empty.") &= 0xf0;
        } else {
            // Append a new 1 byte to be the flag when even.
            nibble_bytes.push(1);
        }
        let mut tuple = serializer.serialize_tuple(2)?;
        tuple.serialize_element(&nibble_bytes)?;
        tuple.serialize_element(&self.child())?;
        tuple.end()
    }
}

struct ExtensionNodeVisitor;

impl<'de> Visitor<'de> for ExtensionNodeVisitor {
    type Value = ExtensionNode;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "This visitor expects to receive a vector of u8")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut nibble_bytes: Vec<u8> = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
        let child: HashValue = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(1, &self))?;
        match nibble_bytes
            .last()
            .ok_or_else(|| de::Error::custom("Serialization of extension node is empty."))?
            & 0x0f
        {
            1u8 => {
                nibble_bytes.pop();
                Ok(ExtensionNode::new(NibblePath::new(nibble_bytes), child))
            }
            0u8 => Ok(ExtensionNode::new(NibblePath::new_odd(nibble_bytes), child)),
            _ => Err(de::Error::custom(format!(
                "The last byte of the serialization of extension node is corrupt: {:?}",
                nibble_bytes
            ))),
        }
    }
}

impl<'de> Deserialize<'de> for ExtensionNode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_tuple(2, ExtensionNodeVisitor)
    }
}

/// Customized LeafNode serialization/deserialization
///
/// A leaf node will be serialized into two hash values.  They are the key to this leaf node and the
/// hash of the value blob under this account. For example, if a leaf node has hash1 as key with
/// hash2 as its value hash, the serialization structure will be:
/// 1st field: hash1
/// 2nd field: hash2
impl Serialize for LeafNode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut tuple = serializer.serialize_tuple(2)?;
        tuple.serialize_element(&self.key())?;
        tuple.serialize_element(&self.value_hash())?;
        tuple.end()
    }
}

struct LeafNodeVisitor;

impl<'de> Visitor<'de> for LeafNodeVisitor {
    type Value = LeafNode;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "This visitor expects to receive a vector of u8")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let key: HashValue = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
        let value_hash: HashValue = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(1, &self))?;
        Ok(LeafNode::new(key, value_hash))
    }
}

impl<'de> Deserialize<'de> for LeafNode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_tuple(2, LeafNodeVisitor)
    }
}
