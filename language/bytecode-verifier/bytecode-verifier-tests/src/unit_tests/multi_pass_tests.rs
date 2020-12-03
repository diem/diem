// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bytecode_verifier::{
    constants, instantiation_loops::InstantiationLoopChecker, DuplicationChecker,
    InstructionConsistency, RecursiveStructDefChecker, ResourceTransitiveChecker, SignatureChecker,
};
use proptest::prelude::*;
use vm::CompiledModule;

proptest! {
    #[test]
    fn check_verifier_passes(module in CompiledModule::valid_strategy(20)) {
        DuplicationChecker::verify_module(&module).expect("DuplicationChecker failure");
        SignatureChecker::verify_module(&module).expect("SignatureChecker failure");
        InstructionConsistency::verify_module(&module).expect("InstructionConsistency failure");
        constants::verify_module(&module).expect("constants failure");
        ResourceTransitiveChecker::verify_module(&module).expect("ResourceTransitiveChecker failure");
        RecursiveStructDefChecker::verify_module(&module).expect("RecursiveStructDefChecker failure");
        InstantiationLoopChecker::verify_module(&module).expect("InstantiationLoopChecker failure");
    }
}
