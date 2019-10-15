// Copyright (c) The SG Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::access_path::AccessPath;
use crate::account_address::AccountAddress;
use crate::write_set::WriteSet;
use canonical_serialization::SimpleSerializer;
use failure::{Error, Result};
use std::collections::{BTreeMap, HashMap};
use std::convert::TryFrom;

impl TryFrom<&WriteSet> for HashMap<AccountAddress, BTreeMap<Vec<u8>, Vec<u8>>> {
    type Error = Error;

    fn try_from(ws: &WriteSet) -> Result<Self> {
        let mut account_state = HashMap::new();

        for (ap, wp) in ws.iter() {
            let AccessPath { address, path } = ap;
            let write_op_blob = SimpleSerializer::<Vec<u8>>::serialize(wp)?;
            match account_state.entry(*address) {
                ::std::collections::hash_map::Entry::Vacant(e) => {
                    let mut state = BTreeMap::new();
                    state.insert(path.clone(), write_op_blob);
                    e.insert(state);
                }
                std::collections::hash_map::Entry::Occupied(mut e) => {
                    e.get_mut().insert(path.clone(), write_op_blob);
                }
            }
        }
        Ok(account_state)
    }
}
