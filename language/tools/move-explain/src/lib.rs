// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use errmapgen::{ErrorContext, ErrorMapping};
use move_core_types::language_storage::ModuleId;

/// Given the module ID and the abort code raised from that module, returns the human-readable
/// explanation of that abort if possible.
pub fn get_explanation(module_id: &ModuleId, abort_code: u64) -> Option<ErrorContext> {
    let error_descriptions: ErrorMapping =
        bcs::from_bytes(compiled_stdlib::ERROR_DESCRIPTIONS).unwrap();
    error_descriptions.get_explanation(module_id, abort_code)
}
