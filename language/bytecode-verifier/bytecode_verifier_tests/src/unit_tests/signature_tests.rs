// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bytecode_verifier::{SignatureChecker, VerifiedModule};
use invalid_mutations::signature::{
    ApplySignatureDoubleRefContext, ApplySignatureFieldRefContext, DoubleRefMutation,
    FieldRefMutation,
};
use libra_types::{account_address::AccountAddress, vm_error::StatusCode};
use move_core_types::identifier::Identifier;
use proptest::{collection::vec, prelude::*};
use vm::file_format::{Bytecode::*, CompiledModule, SignatureToken::*, *};

#[test]
fn test_reference_of_reference() {
    let mut m = basic_test_module();
    m.locals_signatures[0] = LocalsSignature(vec![Reference(Box::new(Reference(Box::new(
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
        mutations in vec(DoubleRefMutation::strategy(), 0..40),
    ) {
        let mut module = module.into_inner();
        let mut expected_violations = {
            let context = ApplySignatureDoubleRefContext::new(&mut module, mutations);
            context.apply()
        };
        expected_violations.sort();
        let module = module.freeze().expect("should satisfy bounds checker");

        let signature_checker = SignatureChecker::new(&module);

        let actual_violations = signature_checker.verify();
        // Since some type signatures are field definition references as well, actual_violations
        // will also contain VMStaticViolation::InvalidFieldDefReference errors -- filter those
        // out.
        let mut actual_violations: Vec<_> = actual_violations
            .into_iter()
            .filter(|err| err.major_status != StatusCode::INVALID_FIELD_DEF)
            .collect();
        actual_violations.sort();
        // The error messages are slightly different from the invalid mutations, so clean these out
        for violation in actual_violations.iter_mut() {
            violation.set_message("".to_string())
        }
        for violation in expected_violations.iter_mut() {
            violation.set_message("".to_string())
        }
        prop_assert_eq!(expected_violations, actual_violations);
    }

    #[test]
    fn field_def_references(
        module in CompiledModule::valid_strategy(20),
        mutations in vec(FieldRefMutation::strategy(), 0..40),
    ) {
        let mut module = module.into_inner();
        let mut expected_violations = {
            let context = ApplySignatureFieldRefContext::new(&mut module, mutations);
            context.apply()
        };
        expected_violations.sort();
        let module = module.freeze().expect("should satisfy bounds checker");

        let signature_checker = SignatureChecker::new(&module);

        let mut actual_violations = signature_checker.verify();
        // Note that this shouldn't cause any InvalidSignatureToken errors because there are no
        // double references involved. So no filtering is required here.
        actual_violations.sort();
        // The error messages are slightly different from the invalid mutations, so clean these out
        for violation in actual_violations.iter_mut() {
            violation.set_message("".to_string())
        }
        for violation in expected_violations.iter_mut() {
            violation.set_message("".to_string())
        }
        prop_assert_eq!(expected_violations, actual_violations);
    }
}

#[test]
fn no_verify_locals_good() {
    let compiled_module_good = CompiledModuleMut {
        module_handles: vec![ModuleHandle {
            address: AddressPoolIndex(0),
            name: IdentifierIndex(0),
        }],
        struct_handles: vec![],
        function_handles: vec![
            FunctionHandle {
                module: ModuleHandleIndex(0),
                name: IdentifierIndex(1),
                signature: FunctionSignatureIndex(0),
            },
            FunctionHandle {
                module: ModuleHandleIndex(0),
                name: IdentifierIndex(2),
                signature: FunctionSignatureIndex(1),
            },
        ],
        type_signatures: vec![],
        function_signatures: vec![
            FunctionSignature {
                return_types: vec![],
                arg_types: vec![Address],
                type_formals: vec![],
            },
            FunctionSignature {
                return_types: vec![],
                arg_types: vec![U64],
                type_formals: vec![],
            },
        ],
        locals_signatures: vec![LocalsSignature(vec![Address]), LocalsSignature(vec![U64])],
        identifiers: vec![
            Identifier::new("Bad").unwrap(),
            Identifier::new("blah").unwrap(),
            Identifier::new("foo").unwrap(),
        ],
        byte_array_pool: vec![],
        address_pool: vec![AccountAddress::new([0; AccountAddress::LENGTH])],
        struct_defs: vec![],
        field_defs: vec![],
        function_defs: vec![
            FunctionDefinition {
                function: FunctionHandleIndex(0),
                flags: 1,
                acquires_global_resources: vec![],
                code: CodeUnit {
                    max_stack_size: 0,
                    locals: LocalsSignatureIndex(0),
                    code: vec![Ret],
                },
            },
            FunctionDefinition {
                function: FunctionHandleIndex(1),
                flags: 0,
                acquires_global_resources: vec![],
                code: CodeUnit {
                    max_stack_size: 0,
                    locals: LocalsSignatureIndex(1),
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
            address: AddressPoolIndex(0),
            name: IdentifierIndex(0),
        }],
        struct_handles: vec![],
        function_handles: vec![FunctionHandle {
            module: ModuleHandleIndex(0),
            name: IdentifierIndex(1),
            signature: FunctionSignatureIndex(0),
        }],
        type_signatures: vec![],
        function_signatures: vec![FunctionSignature {
            return_types: vec![],
            arg_types: vec![Address],
            type_formals: vec![],
        }],
        locals_signatures: vec![LocalsSignature(vec![U64])],
        identifiers: vec![
            Identifier::new("Bad").unwrap(),
            Identifier::new("blah").unwrap(),
        ],
        byte_array_pool: vec![],
        address_pool: vec![AccountAddress::new([0; AccountAddress::LENGTH])],
        struct_defs: vec![],
        field_defs: vec![],
        function_defs: vec![FunctionDefinition {
            function: FunctionHandleIndex(0),
            flags: 1,
            acquires_global_resources: vec![],
            code: CodeUnit {
                max_stack_size: 0,
                locals: LocalsSignatureIndex(0),
                code: vec![Ret],
            },
        }],
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
            address: AddressPoolIndex(0),
            name: IdentifierIndex(0),
        }],
        struct_handles: vec![],
        function_handles: vec![FunctionHandle {
            module: ModuleHandleIndex(0),
            name: IdentifierIndex(1),
            signature: FunctionSignatureIndex(0),
        }],
        type_signatures: vec![],
        function_signatures: vec![FunctionSignature {
            return_types: vec![],
            arg_types: vec![Address],
            type_formals: vec![],
        }],
        locals_signatures: vec![LocalsSignature(vec![])],
        identifiers: vec![
            Identifier::new("Bad").unwrap(),
            Identifier::new("blah").unwrap(),
        ],
        byte_array_pool: vec![],
        address_pool: vec![AccountAddress::new([0; AccountAddress::LENGTH])],
        struct_defs: vec![],
        field_defs: vec![],
        function_defs: vec![FunctionDefinition {
            function: FunctionHandleIndex(0),
            flags: 1,
            acquires_global_resources: vec![],
            code: CodeUnit {
                max_stack_size: 0,
                locals: LocalsSignatureIndex(0),
                code: vec![Ret],
            },
        }],
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
            address: AddressPoolIndex(0),
            name: IdentifierIndex(0),
        }],
        struct_handles: vec![],
        function_handles: vec![FunctionHandle {
            module: ModuleHandleIndex(0),
            name: IdentifierIndex(1),
            signature: FunctionSignatureIndex(0),
        }],
        type_signatures: vec![],
        function_signatures: vec![FunctionSignature {
            return_types: vec![],
            arg_types: vec![Address],
            type_formals: vec![],
        }],
        locals_signatures: vec![LocalsSignature(vec![U64, Address])],
        identifiers: vec![
            Identifier::new("Bad").unwrap(),
            Identifier::new("blah").unwrap(),
        ],
        byte_array_pool: vec![],
        address_pool: vec![AccountAddress::new([0; AccountAddress::LENGTH])],
        struct_defs: vec![],
        field_defs: vec![],
        function_defs: vec![FunctionDefinition {
            function: FunctionHandleIndex(0),
            flags: 1,
            acquires_global_resources: vec![],
            code: CodeUnit {
                max_stack_size: 0,
                locals: LocalsSignatureIndex(0),
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
            address: AddressPoolIndex(0),
            name: IdentifierIndex(0),
        }],
        struct_handles: vec![],
        function_handles: vec![FunctionHandle {
            module: ModuleHandleIndex(0),
            name: IdentifierIndex(1),
            signature: FunctionSignatureIndex(0),
        }],
        type_signatures: vec![],
        function_signatures: vec![FunctionSignature {
            return_types: vec![],
            arg_types: vec![U64, Address],
            type_formals: vec![],
        }],
        locals_signatures: vec![LocalsSignature(vec![U64, U64, Address])],
        identifiers: vec![
            Identifier::new("Bad").unwrap(),
            Identifier::new("blah").unwrap(),
        ],
        byte_array_pool: vec![],
        address_pool: vec![AccountAddress::new([0; AccountAddress::LENGTH])],
        struct_defs: vec![],
        field_defs: vec![],
        function_defs: vec![FunctionDefinition {
            function: FunctionHandleIndex(0),
            flags: 1,
            acquires_global_resources: vec![],
            code: CodeUnit {
                max_stack_size: 0,
                locals: LocalsSignatureIndex(0),
                code: vec![Ret],
            },
        }],
    };
    assert!(VerifiedModule::new(compiled_module_bad1.freeze().unwrap()).is_err());
}
