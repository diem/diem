// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_core_types::language_storage::TypeTag;

/// Useful error message.
pub(crate) fn type_not_allowed(type_tag: &TypeTag) -> ! {
    panic!(
        "Transaction scripts cannot take arguments of type {}.",
        type_tag
    );
}

/// Clean up doc comments extracter by the Move prover.
pub(crate) fn prepare_doc_string(doc: &str) -> String {
    let mut doc = doc.replace("\n", " ").trim().to_string();
    loop {
        let doc2 = doc.replace("  ", " ");
        if doc == doc2 {
            return doc;
        }
        doc = doc2;
    }
}
