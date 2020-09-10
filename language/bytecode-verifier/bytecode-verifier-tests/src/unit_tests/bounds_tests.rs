// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use invalid_mutations::bounds::{
    ApplyCodeUnitBoundsContext, ApplyOutOfBoundsContext, CodeUnitBoundsMutation,
    OutOfBoundsMutation,
};
use libra_types::{account_address::AccountAddress, vm_status::StatusCode};
use move_core_types::identifier::Identifier;
use proptest::{collection::vec, prelude::*};
use vm::{check_bounds::BoundsChecker, file_format::*, proptest_types::CompiledModuleStrategyGen};

#[test]
fn empty_module_no_errors() {
    basic_test_module().freeze().unwrap();
}

#[test]
fn invalid_type_param_in_fn_return_() {
    use SignatureToken::*;

    let mut m = basic_test_module();
    m.function_handles[0].return_ = SignatureIndex(1);
    m.signatures.push(Signature(vec![TypeParameter(0)]));
    assert_eq!(m.signatures.len(), 2);
    m.freeze().unwrap_err();
}

#[test]
fn invalid_type_param_in_fn_parameters() {
    use SignatureToken::*;

    let mut m = basic_test_module();
    m.function_handles[0].parameters = SignatureIndex(1);
    m.signatures.push(Signature(vec![TypeParameter(0)]));
    m.freeze().unwrap_err();
}

#[test]
fn invalid_struct_in_fn_return_() {
    use SignatureToken::*;

    let mut m = basic_test_module();
    m.function_handles[0].return_ = SignatureIndex(1);
    m.signatures
        .push(Signature(vec![Struct(StructHandleIndex::new(1))]));
    m.freeze().unwrap_err();
}

#[test]
fn invalid_type_param_in_field() {
    use SignatureToken::*;

    let mut m = basic_test_module();
    match &mut m.struct_defs[0].field_information {
        StructFieldInformation::Declared(ref mut fields) => {
            fields[0].signature.0 = TypeParameter(0);
            m.freeze().unwrap_err();
        }
        _ => panic!("attempt to change a field that does not exist"),
    }
}

#[test]
fn invalid_struct_in_field() {
    use SignatureToken::*;

    let mut m = basic_test_module();
    match &mut m.struct_defs[0].field_information {
        StructFieldInformation::Declared(ref mut fields) => {
            fields[0].signature.0 = Struct(StructHandleIndex::new(3));
            m.freeze().unwrap_err();
        }
        _ => panic!("attempt to change a field that does not exist"),
    }
}

#[test]
fn invalid_struct_with_actuals_in_field() {
    use SignatureToken::*;

    let mut m = basic_test_module();
    match &mut m.struct_defs[0].field_information {
        StructFieldInformation::Declared(ref mut fields) => {
            fields[0].signature.0 =
                StructInstantiation(StructHandleIndex::new(0), vec![TypeParameter(0)]);
            m.freeze().unwrap_err();
        }
        _ => panic!("attempt to change a field that does not exist"),
    }
}

#[test]
fn invalid_locals_id_in_call() {
    use Bytecode::*;

    let mut m = basic_test_module();
    m.function_instantiations.push(FunctionInstantiation {
        handle: FunctionHandleIndex::new(0),
        type_parameters: SignatureIndex::new(1),
    });
    let func_inst_idx = FunctionInstantiationIndex(m.function_instantiations.len() as u16 - 1);
    m.function_defs[0].code.as_mut().unwrap().code = vec![CallGeneric(func_inst_idx)];
    m.freeze().unwrap_err();
}

#[test]
fn invalid_type_param_in_call() {
    use Bytecode::*;
    use SignatureToken::*;

    let mut m = basic_test_module();
    m.signatures.push(Signature(vec![TypeParameter(0)]));
    m.function_instantiations.push(FunctionInstantiation {
        handle: FunctionHandleIndex::new(0),
        type_parameters: SignatureIndex::new(1),
    });
    let func_inst_idx = FunctionInstantiationIndex(m.function_instantiations.len() as u16 - 1);
    m.function_defs[0].code.as_mut().unwrap().code = vec![CallGeneric(func_inst_idx)];
    m.freeze().unwrap_err();
}

#[test]
fn invalid_struct_as_type_actual_in_exists() {
    use Bytecode::*;
    use SignatureToken::*;

    let mut m = basic_test_module();
    m.signatures
        .push(Signature(vec![Struct(StructHandleIndex::new(3))]));
    m.function_instantiations.push(FunctionInstantiation {
        handle: FunctionHandleIndex::new(0),
        type_parameters: SignatureIndex::new(1),
    });
    let func_inst_idx = FunctionInstantiationIndex(m.function_instantiations.len() as u16 - 1);
    m.function_defs[0].code.as_mut().unwrap().code = vec![CallGeneric(func_inst_idx)];
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
    gen.zeros_all();
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
        let (module, expected_violations) = {
            let oob_context = ApplyOutOfBoundsContext::new(module, oob_mutations);
            oob_context.apply()
        };

        let actual_violations = BoundsChecker::verify(&module);
        prop_assert_eq!(expected_violations.is_empty(), actual_violations.is_ok());
    }

    #[test]
    fn code_unit_out_of_bounds(
        module in CompiledModule::valid_strategy(20),
        mutations in vec(CodeUnitBoundsMutation::strategy(), 0..40),
    ) {
        let mut module = module.into_inner();
        let expected_violations = {
            let context = ApplyCodeUnitBoundsContext::new(&mut module, mutations);
            context.apply()
        };

        let actual_violations = BoundsChecker::verify(&module);

        prop_assert_eq!(expected_violations.is_empty(), actual_violations.is_ok());
    }

    #[test]
    fn no_module_handles(
        identifiers in vec(any::<Identifier>(), 0..20),
        address_identifiers in vec(any::<AccountAddress>(), 0..20),
    ) {
        // If there are no module handles, the only other things that can be stored are intrinsic
        // data.
        let mut module = CompiledModuleMut::default();
        module.identifiers = identifiers;
        module.address_identifiers = address_identifiers;

        prop_assert_eq!(
            BoundsChecker::verify(&module).map_err(|e| e.major_status()),
            Err(StatusCode::NO_MODULE_HANDLES)
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
