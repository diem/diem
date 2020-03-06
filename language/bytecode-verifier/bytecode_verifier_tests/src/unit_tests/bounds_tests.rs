// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use invalid_mutations::bounds::{
    ApplyCodeUnitBoundsContext, ApplyOutOfBoundsContext, CodeUnitBoundsMutation,
    OutOfBoundsMutation,
};
use libra_types::{account_address::AccountAddress, byte_array::ByteArray, vm_error::StatusCode};
use move_core_types::identifier::Identifier;
use proptest::{collection::vec, prelude::*};
use vm::{check_bounds::BoundsChecker, file_format::*, proptest_types::CompiledModuleStrategyGen};

#[test]
fn empty_module_no_errors() {
    basic_test_module().freeze().unwrap();
}

#[test]
fn invalid_type_param_in_fn_return_types() {
    use SignatureToken::*;

    let mut m = basic_test_module();
    m.function_signatures[0].return_types = vec![TypeParameter(0)];
    m.freeze().unwrap_err();
}

#[test]
fn invalid_type_param_in_fn_arg_types() {
    use SignatureToken::*;

    let mut m = basic_test_module();
    m.function_signatures[0].arg_types = vec![TypeParameter(0)];
    m.freeze().unwrap_err();
}

#[test]
fn invalid_struct_in_fn_return_types() {
    use SignatureToken::*;

    let mut m = basic_test_module();
    m.function_signatures[0].return_types = vec![Struct(StructHandleIndex::new(1), vec![])];
    m.freeze().unwrap_err();
}

#[test]
fn invalid_type_param_in_field() {
    use SignatureToken::*;

    let mut m = basic_test_module();
    m.type_signatures[0].0 = TypeParameter(0);
    m.freeze().unwrap_err();
}

#[test]
fn invalid_struct_in_field() {
    use SignatureToken::*;

    let mut m = basic_test_module();
    m.type_signatures[0].0 = Struct(StructHandleIndex::new(3), vec![]);
    m.freeze().unwrap_err();
}

#[test]
fn invalid_struct_with_actuals_in_field() {
    use SignatureToken::*;

    let mut m = basic_test_module();
    m.type_signatures[0].0 = Struct(StructHandleIndex::new(0), vec![TypeParameter(0)]);
    m.freeze().unwrap_err();
}

#[test]
fn invalid_locals_id_in_call() {
    use Bytecode::*;

    let mut m = basic_test_module();
    m.function_defs[0].code.code = vec![Call(
        FunctionHandleIndex::new(0),
        LocalsSignatureIndex::new(1),
    )];
    m.freeze().unwrap_err();
}

#[test]
fn invalid_type_param_in_call() {
    use Bytecode::*;
    use SignatureToken::*;

    let mut m = basic_test_module();
    m.locals_signatures
        .push(LocalsSignature(vec![TypeParameter(0)]));
    m.function_defs[0].code.code = vec![Call(
        FunctionHandleIndex::new(0),
        LocalsSignatureIndex::new(1),
    )];
    m.freeze().unwrap_err();
}

#[test]
fn invalid_struct_as_type_actual_in_exists() {
    use Bytecode::*;
    use SignatureToken::*;

    let mut m = basic_test_module();
    m.locals_signatures.push(LocalsSignature(vec![Struct(
        StructHandleIndex::new(3),
        vec![],
    )]));
    m.function_defs[0].code.code = vec![Call(
        FunctionHandleIndex::new(0),
        LocalsSignatureIndex::new(1),
    )];
    m.freeze().unwrap_err();
}

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
        for violation in actual_violations.iter_mut() {
            violation.set_message("".to_string())
        }
        for violation in expected_violations.iter_mut() {
            violation.set_message("".to_string())
        }
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
        for violation in actual_violations.iter_mut() {
            violation.set_message("".to_string())
        }
        for violation in expected_violations.iter_mut() {
            violation.set_message("".to_string())
        }
        prop_assert_eq!(expected_violations, actual_violations);
    }

    #[test]
    fn no_module_handles(
        identifiers in vec(any::<Identifier>(), 0..20),
        address_pool in vec(any::<AccountAddress>(), 0..20),
        byte_array_pool in vec(any::<ByteArray>(), 0..20),
    ) {
        // If there are no module handles, the only other things that can be stored are intrinsic
        // data.
        let mut module = CompiledModuleMut::default();
        module.identifiers = identifiers;
        module.address_pool = address_pool;
        module.byte_array_pool = byte_array_pool;

        let bounds_checker = BoundsChecker::new(&module);
        let actual_violations: Vec<StatusCode> = bounds_checker.verify().into_iter().map(|status| status.major_status).collect();
        prop_assert_eq!(
            actual_violations,
            vec![StatusCode::NO_MODULE_HANDLES]
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
