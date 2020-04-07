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
    assert!(!errors.is_empty());
}

proptest! {
    #[test]
    fn valid_signatures(module in CompiledModule::valid_strategy(20)) {
        let signature_checker = SignatureChecker::new(&module);
        prop_assert_eq!(signature_checker.verify(), vec![]);
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
        let actual_violations = signature_checker.verify();

        prop_assert_eq!(expected_violations, !actual_violations.is_empty());
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
        let actual_violations = signature_checker.verify();

        prop_assert_eq!(expected_violations, !actual_violations.is_empty());
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
                flags: 1,
                acquires_global_resources: vec![],
                code: CodeUnit {
                    max_stack_size: 0,
                    locals: SignatureIndex(0),
                    code: vec![Ret],
                },
            },
            FunctionDefinition {
                function: FunctionHandleIndex(1),
                flags: 0,
                acquires_global_resources: vec![],
                code: CodeUnit {
                    max_stack_size: 0,
                    locals: SignatureIndex(1),
                    code: vec![Ret],
                },
            },
        ],
    };
    assert!(VerifiedModule::new(compiled_module_good.freeze().unwrap()).is_ok());
}

#[test]
fn no_verify_locals_bad1() {
    // This test creates a function with one argument of type Address and
    // a vector of locals containing a single entry of type U64. The function
    // must fail verification since the argument type at position 0 is different
    // from the local type at position 0.
    let compiled_module_bad1 = CompiledModuleMut {
        module_handles: vec![ModuleHandle {
            address: AddressIdentifierIndex(0),
            name: IdentifierIndex(0),
        }],
        struct_handles: vec![],
        signatures: vec![
            Signature(vec![U64]),
            Signature(vec![Address]),
            Signature(vec![]),
        ],
        function_handles: vec![FunctionHandle {
            module: ModuleHandleIndex(0),
            name: IdentifierIndex(1),
            parameters: SignatureIndex(1),
            return_: SignatureIndex(2),
            type_parameters: vec![],
        }],
        function_defs: vec![FunctionDefinition {
            function: FunctionHandleIndex(0),
            flags: 1,
            acquires_global_resources: vec![],
            code: CodeUnit {
                max_stack_size: 0,
                locals: SignatureIndex(0),
                code: vec![Ret],
            },
        }],
        field_handles: vec![],
        struct_def_instantiations: vec![],
        function_instantiations: vec![],
        field_instantiations: vec![],
        identifiers: vec![
            Identifier::new("Bad").unwrap(),
            Identifier::new("blah").unwrap(),
        ],
        address_identifiers: vec![AccountAddress::new([0; AccountAddress::LENGTH])],
        constant_pool: vec![],
        struct_defs: vec![],
    };
    assert!(VerifiedModule::new(compiled_module_bad1.freeze().unwrap()).is_err());
}

#[test]
fn no_verify_locals_bad2() {
    // This test creates a function with one argument of type Address and
    // an empty vector of locals. The function must fail verification since
    // number of arguments is greater than the number of locals.
    let compiled_module_bad2 = CompiledModuleMut {
        module_handles: vec![ModuleHandle {
            address: AddressIdentifierIndex(0),
            name: IdentifierIndex(0),
        }],
        struct_handles: vec![],
        signatures: vec![Signature(vec![]), Signature(vec![Address])],
        function_handles: vec![FunctionHandle {
            module: ModuleHandleIndex(0),
            name: IdentifierIndex(1),
            parameters: SignatureIndex(1),
            return_: SignatureIndex(0),
            type_parameters: vec![],
        }],
        function_defs: vec![FunctionDefinition {
            function: FunctionHandleIndex(0),
            flags: 1,
            acquires_global_resources: vec![],
            code: CodeUnit {
                max_stack_size: 0,
                locals: SignatureIndex(0),
                code: vec![Ret],
            },
        }],
        field_handles: vec![],
        struct_def_instantiations: vec![],
        function_instantiations: vec![],
        field_instantiations: vec![],
        identifiers: vec![
            Identifier::new("Bad").unwrap(),
            Identifier::new("blah").unwrap(),
        ],
        address_identifiers: vec![AccountAddress::new([0; AccountAddress::LENGTH])],
        constant_pool: vec![],
        struct_defs: vec![],
    };
    assert!(VerifiedModule::new(compiled_module_bad2.freeze().unwrap()).is_err());
}

#[test]
fn no_verify_locals_bad3() {
    // This test creates a function with one argument of type Address and
    // a vector of locals containing two types, U64 and Address. The function
    // must fail verification since the argument type at position 0 is different
    // from the local type at position 0.
    let compiled_module_bad1 = CompiledModuleMut {
        module_handles: vec![ModuleHandle {
            address: AddressIdentifierIndex(0),
            name: IdentifierIndex(0),
        }],
        struct_handles: vec![],
        signatures: vec![
            Signature(vec![U64, Address]),
            Signature(vec![]),
            Signature(vec![Address]),
        ],
        function_handles: vec![FunctionHandle {
            module: ModuleHandleIndex(0),
            name: IdentifierIndex(1),
            return_: SignatureIndex(1),
            parameters: SignatureIndex(2),
            type_parameters: vec![],
        }],
        field_handles: vec![],
        struct_def_instantiations: vec![],
        function_instantiations: vec![],
        field_instantiations: vec![],
        identifiers: vec![
            Identifier::new("Bad").unwrap(),
            Identifier::new("blah").unwrap(),
        ],
        address_identifiers: vec![AccountAddress::new([0; AccountAddress::LENGTH])],
        constant_pool: vec![],
        struct_defs: vec![],
        function_defs: vec![FunctionDefinition {
            function: FunctionHandleIndex(0),
            flags: 1,
            acquires_global_resources: vec![],
            code: CodeUnit {
                max_stack_size: 0,
                locals: SignatureIndex(0),
                code: vec![Ret],
            },
        }],
    };
    assert!(VerifiedModule::new(compiled_module_bad1.freeze().unwrap()).is_err());
}

#[test]
fn no_verify_locals_bad4() {
    // This test creates a function with two arguments of type U64 and Address and
    // a vector of locals containing three types, U64, U64 and Address. The function
    // must fail verification since the argument type at position 0 is different
    // from the local type at position 0.
    let compiled_module_bad1 = CompiledModuleMut {
        module_handles: vec![ModuleHandle {
            address: AddressIdentifierIndex(0),
            name: IdentifierIndex(0),
        }],
        struct_handles: vec![],
        signatures: vec![
            Signature(vec![U64, U64, Address]),
            Signature(vec![]),
            Signature(vec![U64, Address]),
        ],
        function_handles: vec![FunctionHandle {
            module: ModuleHandleIndex(0),
            name: IdentifierIndex(1),
            return_: SignatureIndex(1),
            parameters: SignatureIndex(2),
            type_parameters: vec![],
        }],
        field_handles: vec![],
        struct_def_instantiations: vec![],
        function_instantiations: vec![],
        field_instantiations: vec![],
        identifiers: vec![
            Identifier::new("Bad").unwrap(),
            Identifier::new("blah").unwrap(),
        ],
        constant_pool: vec![],
        address_identifiers: vec![AccountAddress::new([0; AccountAddress::LENGTH])],
        struct_defs: vec![],
        function_defs: vec![FunctionDefinition {
            function: FunctionHandleIndex(0),
            flags: 1,
            acquires_global_resources: vec![],
            code: CodeUnit {
                max_stack_size: 0,
                locals: SignatureIndex(0),
                code: vec![Ret],
            },
        }],
    };
    assert!(VerifiedModule::new(compiled_module_bad1.freeze().unwrap()).is_err());
}
