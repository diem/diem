// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bytecode_verifier::DuplicationChecker;
use proptest::prelude::*;
use vm::file_format::*;

#[test]
fn duplicated_friend_decls() {
    let mut m = basic_test_module();
    m.friend_decls.push(FriendDeclaration {
        module: ModuleHandleIndex(0 as TableIndex),
    });
    m.friend_decls.push(FriendDeclaration {
        module: ModuleHandleIndex(0 as TableIndex),
    });
    DuplicationChecker::verify_module(&m.freeze().unwrap()).unwrap_err();
}

proptest! {
    #[test]
    fn valid_duplication(module in CompiledModule::valid_strategy(20)) {
        prop_assert!(DuplicationChecker::verify_module(&module).is_ok());
    }
}
