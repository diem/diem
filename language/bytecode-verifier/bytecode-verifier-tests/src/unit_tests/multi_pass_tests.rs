// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bytecode_verifier::{
    ability_field_requirements, constants, instantiation_loops::InstantiationLoopChecker,
    DuplicationChecker, InstructionConsistency, RecursiveStructDefChecker, SignatureChecker,
};
use move_binary_format::CompiledModule;
use proptest::prelude::*;

proptest! {
    #[test]
    fn check_verifier_passes(module in CompiledModule::valid_strategy(20)) {
        DuplicationChecker::verify_module(&module).expect("DuplicationChecker failure");
        SignatureChecker::verify_module(&module).expect("SignatureChecker failure");
        InstructionConsistency::verify_module(&module).expect("InstructionConsistency failure");
        constants::verify_module(&module).expect("constants failure");
        ability_field_requirements::verify_module(&module).expect("ability_field_requirements failure");
        RecursiveStructDefChecker::verify_module(&module).expect("RecursiveStructDefChecker failure");
        InstantiationLoopChecker::verify_module(&module).expect("InstantiationLoopChecker failure");
    }
}
