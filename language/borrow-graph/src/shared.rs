// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
use std::collections::{BTreeMap, BTreeSet};

pub fn remap_set<T: Copy + Ord>(set: &mut BTreeSet<T>, id_map: &BTreeMap<T, T>) {
    for (old, new) in id_map {
        if set.remove(&old) {
            set.insert(*new);
        }
    }
}
