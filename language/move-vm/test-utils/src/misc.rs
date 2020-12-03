// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{format_err, Result};
use move_vm_runtime::data_cache::TransactionEffects;

use crate::effects::{ChangeSet, Event};

pub fn convert_txn_effects_to_move_changeset_and_events(
    txn_effects: TransactionEffects,
) -> Result<(ChangeSet, Vec<Event>)> {
    let mut changeset = ChangeSet::new();

    for (addr, resources) in txn_effects.resources {
        for (struct_tag, val_opt) in resources {
            match val_opt {
                Some((layout, val)) => {
                    let blob = val
                        .simple_serialize(&layout)
                        .ok_or_else(|| format_err!("failed to serialize value"))?;
                    changeset.publish_resource(addr, struct_tag, blob)?;
                }
                None => {
                    changeset.unpublish_resource(addr, struct_tag)?;
                }
            }
        }
    }

    for (module_id, module_blob) in txn_effects.modules {
        changeset.publish_module(module_id, module_blob)?;
    }

    let events = txn_effects
        .events
        .into_iter()
        .map(|(guid, seq_num, ty_tag, layout, val)| {
            let blob = val
                .simple_serialize(&layout)
                .ok_or_else(|| format_err!("failed to serialize value"))?;
            Ok((guid, seq_num, ty_tag, blob))
        })
        .collect::<Result<Vec<_>>>()?;

    Ok((changeset, events))
}
