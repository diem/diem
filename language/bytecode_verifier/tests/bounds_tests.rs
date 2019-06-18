// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use invalid_mutations::bounds::{
    ApplyCodeUnitBoundsContext, ApplyOutOfBoundsContext, CodeUnitBoundsMutation,
    OutOfBoundsMutation,
};
use proptest::{collection::vec, prelude::*};
use types::{account_address::AccountAddress, byte_array::ByteArray};
use vm::{
    check_bounds::BoundsChecker,
    errors::{VMStaticViolation, VerificationError},
    file_format::{CompiledModule, CompiledModuleMut},
    proptest_types::CompiledModuleStrategyGen,
    IndexKind,
};

proptest! {
    #[test]
    fn valid_bounds(_module in CompiledModule::valid_strategy(20)) {
        // valid_strategy will panic if there are any bounds check issues.
    }
}

/// Ensure that valid modules that don't have any members (e.g. function args, struct fields) pass
/// bounds checks.
///
/// There are some potentially tricky edge cases around ranges that are captured here.
#[test]
fn valid_bounds_no_members() {
    let mut gen = CompiledModuleStrategyGen::new(20);
    gen.member_count(0);
    proptest!(|(_module in gen.generate())| {
        // gen.generate() will panic if there are any bounds check issues.
    });
}

proptest! {
    #[test]
    fn invalid_out_of_bounds(
        module in CompiledModule::valid_strategy(20),
        oob_mutations in vec(OutOfBoundsMutation::strategy(), 0..40),
    ) {
        let (module, mut expected_violations) = {
            let oob_context = ApplyOutOfBoundsContext::new(module, oob_mutations);
            oob_context.apply()
        };
        expected_violations.sort();

        let bounds_checker = BoundsChecker::new(&module);
        let mut actual_violations = bounds_checker.verify();
        actual_violations.sort();
        prop_assert_eq!(expected_violations, actual_violations);
    }

    #[test]
    fn code_unit_out_of_bounds(
        module in CompiledModule::valid_strategy(20),
        mutations in vec(CodeUnitBoundsMutation::strategy(), 0..40),
    ) {
        let mut module = module.into_inner();
        let mut expected_violations = {
            let context = ApplyCodeUnitBoundsContext::new(&mut module, mutations);
            context.apply()
        };
        expected_violations.sort();

        let bounds_checker = BoundsChecker::new(&module);
        let mut actual_violations = bounds_checker.verify();
        actual_violations.sort();
        prop_assert_eq!(expected_violations, actual_violations);
    }

    #[test]
    fn no_module_handles(
        string_pool in vec(".*", 0..20),
        address_pool in vec(any::<AccountAddress>(), 0..20),
        byte_array_pool in vec(any::<ByteArray>(), 0..20),
    ) {
        // If there are no module handles, the only other things that can be stored are intrinsic
        // data.
        let mut module = CompiledModuleMut::default();
        module.string_pool = string_pool;
        module.address_pool = address_pool;
        module.byte_array_pool = byte_array_pool;

        let bounds_checker = BoundsChecker::new(&module);
        let actual_violations = bounds_checker.verify();
        prop_assert_eq!(
            actual_violations,
            vec![
                VerificationError {
                    kind: IndexKind::ModuleHandle,
                    idx: 0,
                    err: VMStaticViolation::NoModuleHandles,
                },
            ]
        );
    }
}

proptest! {
    // Generating arbitrary compiled modules is really slow, possibly because of
    // https://github.com/AltSysrq/proptest/issues/143.
    #![proptest_config(ProptestConfig::with_cases(16))]

    /// Make sure that garbage inputs don't crash the bounds checker.
    #[test]
    fn garbage_inputs(module in any_with::<CompiledModuleMut>(16)) {
        let _ = module.freeze();
    }
}
