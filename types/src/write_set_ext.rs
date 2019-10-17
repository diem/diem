// Copyright (c) The SG Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::access_path::AccessPath;
use crate::account_address::AccountAddress;
use crate::write_set::{WriteOp, WriteSet, WriteSetMut};
use canonical_serialization::{SimpleDeserializer, SimpleSerializer};
use failure::{Error, Result};
use std::collections::BTreeMap;
use std::convert::TryFrom;

impl TryFrom<&WriteSet> for BTreeMap<AccountAddress, BTreeMap<Vec<u8>, Vec<u8>>> {
    type Error = Error;

    fn try_from(ws: &WriteSet) -> Result<Self> {
        let mut account_state = BTreeMap::new();

        for (ap, wp) in ws.iter() {
            let AccessPath { address, path } = ap;
            let write_op_blob = SimpleSerializer::<Vec<u8>>::serialize(wp)?;
            match account_state.entry(*address) {
                ::std::collections::btree_map::Entry::Vacant(e) => {
                    let mut state = BTreeMap::new();
                    state.insert(path.clone(), write_op_blob);
                    e.insert(state);
                }
                std::collections::btree_map::Entry::Occupied(mut e) => {
                    e.get_mut().insert(path.clone(), write_op_blob);
                }
            }
        }
        Ok(account_state)
    }
}

impl TryFrom<&BTreeMap<AccountAddress, BTreeMap<Vec<u8>, Vec<u8>>>> for WriteSet {
    type Error = Error;

    fn try_from(value: &BTreeMap<AccountAddress, BTreeMap<Vec<u8>, Vec<u8>>>) -> Result<Self> {
        let mut ws = WriteSetMut::default();
        for (account, state) in value.into_iter() {
            for (data_path, data) in state.into_iter() {
                ws.push((
                    AccessPath::new(*account, data_path.clone()),
                    SimpleDeserializer::deserialize::<WriteOp>(data)?,
                ));
            }
        }
        ws.freeze()
    }
}
