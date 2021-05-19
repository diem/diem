// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! A module for looking up the human-readable explanation of a Diem Move
//! transaction abort code.
//!
//! This module mostly exists as a temporary hack until we figure out a more
//! sustainable approach towards publishing the Diem move framework release to
//! crates.io.
//!
//! Note that the ~13 KiB error descriptions will be inlined into the final binary.

use move_core_types::{
    errmap::{ErrorContext, ErrorMapping},
    language_storage::ModuleId,
};
use once_cell::sync::Lazy;

static RELEASE_ERRMAP_BYTES: &[u8] = include_bytes!("../release_errmap/error_description.errmap");

static RELEASE_ERRMAP: Lazy<ErrorMapping> = Lazy::new(|| {
    bcs::from_bytes(&*RELEASE_ERRMAP_BYTES)
        .expect("Failed to deserialize static error descriptions")
});

/// Given the module ID and the abort code raised from that module, returns the
/// human-readable explanation of that abort if possible.
pub fn get_explanation(module_id: &ModuleId, abort_code: u64) -> Option<ErrorContext> {
    let errmap = &*RELEASE_ERRMAP;
    errmap.get_explanation(module_id, abort_code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use move_core_types::{account_address::AccountAddress, ident_str};

    #[test]
    fn test_get_explanation() {
        let module_id = ModuleId::new(AccountAddress::ZERO, ident_str!("TESTTEST").to_owned());
        // We don't care about the result, just that the errmap deserializes without panicking.
        let _ = get_explanation(&module_id, 1234);
    }
}
