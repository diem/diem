// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! For each transaction the VM executes, the VM will output a `WriteSet` that contains each access
//! path it updates. For each access path, the VM can either give its new value or delete it.

use crate::access_path::AccessPath;
use canonical_serialization::{
    CanonicalDeserialize, CanonicalDeserializer, CanonicalSerialize, CanonicalSerializer,
};
use failure::prelude::*;
use proto_conv::{FromProto, IntoProto};
use serde::{Deserialize, Serialize};

#[derive(Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum WriteOp {
    Value(Vec<u8>),
    Deletion,
}

impl WriteOp {
    #[inline]
    pub fn is_value(&self) -> bool {
        match self {
            WriteOp::Value(_) => true,
            WriteOp::Deletion => false,
        }
    }

    #[inline]
    pub fn is_deletion(&self) -> bool {
        match self {
            WriteOp::Deletion => true,
            WriteOp::Value(_) => false,
        }
    }
}

impl std::fmt::Debug for WriteOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WriteOp::Value(value) => write!(f, "Value({})", String::from_utf8_lossy(value)),
            WriteOp::Deletion => write!(f, "Deletion"),
        }
    }
}

impl CanonicalSerialize for WriteOp {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        match self {
            WriteOp::Deletion => serializer.encode_u32(WriteOpType::Deletion as u32)?,
            WriteOp::Value(value) => {
                serializer.encode_u32(WriteOpType::Value as u32)?;
                serializer.encode_vec(value)?
            }
        };
        Ok(())
    }
}

impl CanonicalDeserialize for WriteOp {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self> {
        let decoded_write_op_type = deserializer.decode_u32()?;
        let write_op_type = WriteOpType::from_u32(decoded_write_op_type);
        match write_op_type {
            Some(WriteOpType::Deletion) => Ok(WriteOp::Deletion),
            Some(WriteOpType::Value) => Ok(WriteOp::Value(deserializer.decode_vec()?)),
            None => Err(format_err!(
                "ParseError: Unable to decode WriteOpType, found {}",
                decoded_write_op_type
            )),
        }
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
enum WriteOpType {
    Deletion = 0,
    Value = 1,
}

impl WriteOpType {
    fn from_u32(value: u32) -> Option<WriteOpType> {
        match value {
            0 => Some(WriteOpType::Deletion),
            1 => Some(WriteOpType::Value),
            _ => None,
        }
    }
}

/// `WriteSet` contains all access paths that one transaction modifies. Each of them is a `WriteOp`
/// where `Value(val)` means that serialized representation should be updated to `val`, and
/// `Deletion` means that we are going to delete this access path.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct WriteSet(WriteSetMut);

impl WriteSet {
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    pub fn iter<'a>(&'a self) -> ::std::slice::Iter<'a, (AccessPath, WriteOp)> {
        self.into_iter()
    }

    #[inline]
    pub fn into_mut(self) -> WriteSetMut {
        self.0
    }
}

/// A mutable version of `WriteSet`.
///
/// This is separate because it goes through validation before becoming an immutable `WriteSet`.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct WriteSetMut {
    write_set: Vec<(AccessPath, WriteOp)>,
}

impl WriteSetMut {
    pub fn new(write_set: Vec<(AccessPath, WriteOp)>) -> Self {
        Self { write_set }
    }

    pub fn push(&mut self, item: (AccessPath, WriteOp)) {
        self.write_set.push(item);
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.write_set.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.write_set.is_empty()
    }

    pub fn freeze(self) -> Result<WriteSet> {
        // TODO: add structural validation
        Ok(WriteSet(self))
    }
}

impl FromProto for WriteSet {
    type ProtoType = crate::proto::transaction::WriteSet;

    fn from_proto(mut write_set: Self::ProtoType) -> Result<Self> {
        use crate::proto::transaction::WriteOpType;

        let write_set = write_set
            .take_write_set()
            .into_iter()
            .map(|mut write_op| {
                // The protobuf WriteOp is equivalent to (AccessPath, WriteOp) in Rust, so
                // From/IntoProto can't be implemented for WriteOp and instead the conversion must
                // be done here.
                let access_path = AccessPath::from_proto(write_op.take_access_path())?;
                let write_op = match write_op.get_field_type() {
                    WriteOpType::Write => WriteOp::Value(write_op.take_value()),
                    WriteOpType::Delete => {
                        ensure!(
                            write_op.get_value().is_empty(),
                            "WriteOp with access path {:?} has WriteOpType::Delete with value",
                            access_path,
                        );
                        WriteOp::Deletion
                    }
                };
                Ok((access_path, write_op))
            })
            .collect::<Result<_>>()?;
        let write_set_mut = WriteSetMut::new(write_set);
        write_set_mut.freeze()
    }
}

impl IntoProto for WriteSet {
    type ProtoType = crate::proto::transaction::WriteSet;

    fn into_proto(self) -> Self::ProtoType {
        use crate::proto::transaction::{WriteOp as ProtoWriteOp, WriteOpType};

        let proto_write_ops = self
            .0
            .write_set
            .into_iter()
            .map(|(access_path, write_op)| {
                let mut proto_write_op = ProtoWriteOp::new();
                proto_write_op.set_access_path(access_path.into_proto());
                match write_op {
                    WriteOp::Value(value) => {
                        proto_write_op.set_value(value);
                        proto_write_op.set_field_type(WriteOpType::Write);
                    }
                    WriteOp::Deletion => {
                        // This should be a no-op but this code conveys the intent better.
                        proto_write_op.set_value(vec![]);
                        proto_write_op.set_field_type(WriteOpType::Delete);
                    }
                };
                proto_write_op
            })
            .collect();

        let mut proto_write_set = Self::ProtoType::new();
        proto_write_set.set_write_set(proto_write_ops);
        proto_write_set
    }
}

impl CanonicalSerialize for WriteSet {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer.encode_vec(&self.0.write_set)?;
        Ok(())
    }
}

impl CanonicalDeserialize for WriteSet {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self> {
        let write_set = deserializer.decode_vec::<(AccessPath, WriteOp)>()?;
        WriteSetMut::new(write_set).freeze()
    }
}

impl ::std::iter::FromIterator<(AccessPath, WriteOp)> for WriteSetMut {
    fn from_iter<I: IntoIterator<Item = (AccessPath, WriteOp)>>(iter: I) -> Self {
        let mut ws = WriteSetMut::default();
        for write in iter {
            ws.push((write.0, write.1));
        }
        ws
    }
}

impl<'a> IntoIterator for &'a WriteSet {
    type Item = &'a (AccessPath, WriteOp);
    type IntoIter = ::std::slice::Iter<'a, (AccessPath, WriteOp)>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.write_set.iter()
    }
}

impl ::std::iter::IntoIterator for WriteSet {
    type Item = (AccessPath, WriteOp);
    type IntoIter = ::std::vec::IntoIter<(AccessPath, WriteOp)>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.write_set.into_iter()
    }
}
