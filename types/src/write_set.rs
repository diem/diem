// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! For each transaction the VM executes, the VM will output a `WriteSet` that contains each access
//! path it updates. For each access path, the VM can either give its new value or delete it.

use crate::access_path::AccessPath;
use crate::account_address::AccountAddress;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::mem;

#[derive(Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum WriteOp {
    Deletion,
    Value(Vec<u8>),
}

impl WriteOp {
    #[inline]
    pub fn is_deletion(&self) -> bool {
        match self {
            WriteOp::Deletion => true,
            WriteOp::Value(_) => false,
        }
    }

    pub fn merge_with(&mut self, other: WriteOp) {
        mem::replace(self, other);
    }
}

impl std::fmt::Debug for WriteOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WriteOp::Value(value) => write!(
                f,
                "Value({})",
                value
                    .iter()
                    .map(|byte| format!("{:02x}", byte))
                    .collect::<String>()
            ),
            WriteOp::Deletion => write!(f, "Deletion"),
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

    /// FIXME: delete it, it is flawed.
    /// Check whether the write set modifies the `participant_address`'s private channel resources.
    pub fn contains_channel_resource(&self, participant_address: &AccountAddress) -> bool {
        for (ap, _op) in self {
            if ap
                .data_path()
                .and_then(|data_path| data_path.participant())
                .and_then(|addr| Some(&addr == participant_address))
                .unwrap_or(false)
            {
                return true;
            }
        }
        return false;
    }

    pub fn contains_onchain_resource(&self) -> bool {
        for (access_path, _) in self.iter() {
            if access_path.is_onchain_resource() {
                return true;
            }
        }
        return false;
    }

    pub fn merge(first: &WriteSet, second: &WriteSet) -> Self {
        WriteSetMut::merge(&first.0, &second.0)
            .freeze()
            .expect("freeze should success.")
    }

    pub fn get(&self, access_path: &AccessPath) -> Option<&WriteOp> {
        for (ap, op) in self {
            if ap == access_path {
                return Some(op);
            }
        }
        None
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

    pub fn merge_with(&mut self, other: &WriteSetMut) {
        let new_set = Self::merge(self, other);
        mem::replace(self, new_set);
    }

    pub(crate) fn find_write_op_mut(&mut self, access_path: &AccessPath) -> Option<&mut WriteOp> {
        self.write_set
            .iter_mut()
            .find(|(ap, _)| ap == access_path)
            .map(|(_, op)| op)
    }

    pub fn merge(first: &WriteSetMut, second: &WriteSetMut) -> WriteSetMut {
        let mut write_set = first.clone();
        for (ap, second_op) in &second.write_set {
            match write_set.find_write_op_mut(ap) {
                Some(first_op) => {
                    first_op.merge_with(second_op.clone());
                }
                None => write_set.push((ap.clone(), second_op.clone())),
            }
        }
        write_set
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
