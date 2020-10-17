// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_core_types::language_storage::ModuleId;
use once_cell::sync::Lazy;

pub(crate) static PRECOMPILED_STD_LIB: Lazy<Vec<(ModuleId, Vec<u8>)>> = Lazy::new(|| {
    stdlib::build_move_stdlib()
        .into_iter()
        .map(|compiled_module| {
            let mut blob = vec![];
            compiled_module
                .serialize(&mut blob)
                .expect("failed to serialize module");
            (compiled_module.self_id(), blob)
        })
        .collect()
});
