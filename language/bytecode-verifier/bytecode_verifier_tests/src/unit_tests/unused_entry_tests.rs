// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bytecode_verifier::unused_entries::UnusedEntryChecker;
use proptest::prelude::*;
use vm::file_format::{CompiledModule, Signature};

proptest! {
    #[test]
    fn unused_signature(module in CompiledModule::valid_strategy(10)) {
        let mut module = module.into_inner();
        module.signatures.push(Signature(vec![]));
        let module = module.freeze().unwrap();
        let unused_entry_checker = UnusedEntryChecker::new(&module);
        prop_assert!(!unused_entry_checker.verify().is_empty());
    }
}
