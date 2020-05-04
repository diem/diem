// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bytecode_verifier::{SignatureChecker, VerifiedModule};
use invalid_mutations::signature::{FieldRefMutation, SignatureRefMutation};
use libra_types::account_address::AccountAddress;
use move_core_types::identifier::Identifier;
use proptest::{collection::vec, prelude::*, sample::Index as PropIndex};
use vm::file_format::{Bytecode::*, CompiledModule, SignatureToken::*, *};

#[test]
fn test_reference_of_reference() {
    let mut m = basic_test_module();
    m.signatures[0] = Signature(vec![Reference(Box::new(Reference(Box::new(
        SignatureToken::Bool,
    ))))]);
    let errors = SignatureChecker::new(&m.freeze().unwrap()).verify();
    assert!(errors.is_err());
}

proptest! {
    #[test]
    fn valid_signatures(module in CompiledModule::valid_strategy(20)) {
        let signature_checker = SignatureChecker::new(&module);
        prop_assert!(signature_checker.verify().is_ok())
    }

    #[test]
    fn double_refs(
        module in CompiledModule::valid_strategy(20),
        mutations in vec((any::<PropIndex>(), any::<PropIndex>()), 0..20),
    ) {
        let mut module = module.into_inner();
        let context = SignatureRefMutation::new(&mut module, mutations);
        let expected_violations = context.apply();
        let module = module.freeze().expect("should satisfy bounds checker");

        let signature_checker = SignatureChecker::new(&module);
        let result = signature_checker.verify();

        prop_assert_eq!(expected_violations, result.is_err());
    }

    #[test]
    fn field_def_references(
        module in CompiledModule::valid_strategy(20),
        mutations in vec((any::<PropIndex>(), any::<PropIndex>()), 0..40),
    ) {
        let mut module = module.into_inner();
        let context = FieldRefMutation::new(&mut module, mutations);
        let expected_violations = context.apply();
        let module = module.freeze().expect("should satisfy bounds checker");

        let signature_checker = SignatureChecker::new(&module);
        let result = signature_checker.verify();

        prop_assert_eq!(expected_violations, result.is_err());
    }
}

#[test]
fn no_verify_locals_good() {
    let compiled_module_good = CompiledModuleMut {
        module_handles: vec![ModuleHandle {
            address: AddressIdentifierIndex(0),
            name: IdentifierIndex(0),
        }],
        struct_handles: vec![],
        signatures: vec![
            Signature(vec![Address]),
            Signature(vec![U64]),
            Signature(vec![]),
        ],
        function_handles: vec![
            FunctionHandle {
                module: ModuleHandleIndex(0),
                name: IdentifierIndex(1),
                return_: SignatureIndex(2),
                parameters: SignatureIndex(0),
                type_parameters: vec![],
            },
            FunctionHandle {
                module: ModuleHandleIndex(0),
                name: IdentifierIndex(2),
                return_: SignatureIndex(2),
                parameters: SignatureIndex(1),
                type_parameters: vec![],
            },
        ],
        field_handles: vec![],
        struct_def_instantiations: vec![],
        function_instantiations: vec![],
        field_instantiations: vec![],
        identifiers: vec![
            Identifier::new("Bad").unwrap(),
            Identifier::new("blah").unwrap(),
            Identifier::new("foo").unwrap(),
        ],
        address_identifiers: vec![AccountAddress::new([0; AccountAddress::LENGTH])],
        constant_pool: vec![],
        struct_defs: vec![],
        function_defs: vec![
            FunctionDefinition {
                function: FunctionHandleIndex(0),
                is_public: true,
                acquires_global_resources: vec![],
                code: Some(CodeUnit {
                    locals: SignatureIndex(0),
                    code: vec![Ret],
                }),
            },
            FunctionDefinition {
                function: FunctionHandleIndex(1),
                is_public: true,
                acquires_global_resources: vec![],
                code: Some(CodeUnit {
                    locals: SignatureIndex(1),
                    code: vec![Ret],
                }),
            },
        ],
    };
    assert!(VerifiedModule::new(compiled_module_good.freeze().unwrap()).is_ok());
}
