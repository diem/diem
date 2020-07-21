// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{data_cache::RemoteCache, move_vm::MoveVM};
use libra_types::vm_status::StatusCode;
use move_core_types::{
    account_address::AccountAddress,
    gas_schedule::{GasAlgebra, GasUnits},
    identifier::Identifier,
    language_storage::{ModuleId, TypeTag},
};
use move_vm_types::{
    gas_schedule::{zero_cost_schedule, CostStrategy},
    values::Value,
};
use vm::{
    errors::{PartialVMResult, VMResult},
    file_format::{
        AddressIdentifierIndex, Bytecode, CodeUnit, CompiledScriptMut, FunctionHandle,
        FunctionHandleIndex, IdentifierIndex, ModuleHandle, ModuleHandleIndex, Signature,
        SignatureIndex, SignatureToken, StructHandle, StructHandleIndex,
    },
};

// make a script with a given signature for main. The main just return, cannot
// pass resources or the verifier will fail as being still on the stack (args)
fn make_script(signature: Signature) -> Vec<u8> {
    let mut blob = vec![];
    CompiledScriptMut {
        module_handles: vec![],
        struct_handles: vec![],
        function_handles: vec![],

        function_instantiations: vec![],

        signatures: vec![Signature(vec![]), signature],

        identifiers: vec![],
        address_identifiers: vec![],
        constant_pool: vec![],

        type_parameters: vec![],
        parameters: SignatureIndex(1),
        code: CodeUnit {
            locals: SignatureIndex(0),
            code: vec![Bytecode::Ret],
        },
    }
    .serialize(&mut blob)
    .expect("script must serialize");
    blob
}

// make a script with a given signature for main. The main just return; cannot
// define resources in signature or the verifier will fail with resource not being consumed.
// The script has an imported struct that can be used in main's signature.
// Dependencies check happens after main signature check, so we should expect
// a signature check error.
fn make_script_with_imports(signature: Signature) -> Vec<u8> {
    let mut blob = vec![];
    CompiledScriptMut {
        module_handles: vec![ModuleHandle {
            address: AddressIdentifierIndex(0),
            name: IdentifierIndex(0),
        }],
        struct_handles: vec![StructHandle {
            module: ModuleHandleIndex(0),
            name: IdentifierIndex(1),
            is_nominal_resource: false,
            type_parameters: vec![],
        }],
        function_handles: vec![],

        function_instantiations: vec![],

        signatures: vec![Signature(vec![]), signature],

        identifiers: vec![
            Identifier::new("one").unwrap(),
            Identifier::new("two").unwrap(),
        ],
        address_identifiers: vec![AccountAddress::random()],
        constant_pool: vec![],

        type_parameters: vec![],
        parameters: SignatureIndex(1),
        code: CodeUnit {
            locals: SignatureIndex(0),
            code: vec![Bytecode::Ret],
        },
    }
    .serialize(&mut blob)
    .expect("script must serialize");
    blob
}

// make a script with an external function that has the same signature as
// the main. That allows us to pass resources and make the verifier happy that
// they are consumed.
// Dependencies check happens after main signature check, so we should expect
// a signature check error.
fn make_script_consuming_args(signature: Signature) -> Vec<u8> {
    let mut blob = vec![];
    let mut code = vec![];
    for loc_idx in 0..signature.len() {
        code.push(Bytecode::MoveLoc(loc_idx as u8));
        code.push(Bytecode::Call(FunctionHandleIndex(0)));
    }
    code.push(Bytecode::Ret);
    CompiledScriptMut {
        module_handles: vec![ModuleHandle {
            address: AddressIdentifierIndex(0),
            name: IdentifierIndex(0),
        }],
        struct_handles: vec![StructHandle {
            module: ModuleHandleIndex(0),
            name: IdentifierIndex(1),
            is_nominal_resource: false,
            type_parameters: vec![],
        }],
        function_handles: vec![FunctionHandle {
            module: ModuleHandleIndex(0),
            name: IdentifierIndex(2),
            parameters: SignatureIndex(1),
            return_: SignatureIndex(0),
            type_parameters: vec![],
        }],

        function_instantiations: vec![],

        signatures: vec![Signature(vec![]), signature],

        identifiers: vec![
            Identifier::new("one").unwrap(),
            Identifier::new("two").unwrap(),
            Identifier::new("three").unwrap(),
        ],
        address_identifiers: vec![AccountAddress::random()],
        constant_pool: vec![],

        type_parameters: vec![],
        parameters: SignatureIndex(1),
        code: CodeUnit {
            locals: SignatureIndex(0),
            code,
        },
    }
    .serialize(&mut blob)
    .expect("script must serialize");
    blob
}

struct RemoteStore {}

impl RemoteCache for RemoteStore {
    fn get_module(&self, _module_id: &ModuleId) -> VMResult<Option<Vec<u8>>> {
        Ok(None)
    }

    fn get_resource(
        &self,
        _address: &AccountAddress,
        _tag: &TypeTag,
    ) -> PartialVMResult<Option<Vec<u8>>> {
        Ok(None)
    }
}

fn call_script(script: Vec<u8>, args: Vec<Value>) -> VMResult<()> {
    let move_vm = MoveVM::new();
    let remote_view = RemoteStore {};
    let mut session = move_vm.new_session(&remote_view);
    let cost_table = zero_cost_schedule();
    let mut cost_strategy = CostStrategy::system(&cost_table, GasUnits::new(0));
    session.execute_script(
        script,
        vec![],
        args,
        AccountAddress::random(),
        &mut cost_strategy,
    )
}

#[test]
fn check_main_signature() {
    //
    // Bad signatures
    //

    // struct in signature
    let script = make_script_with_imports(Signature(vec![SignatureToken::Struct(
        StructHandleIndex(0),
    )]));
    assert_eq!(
        call_script(script, vec![Value::u128(0)])
            .err()
            .unwrap()
            .major_status(),
        StatusCode::INVALID_MAIN_FUNCTION_SIGNATURE,
    );
    // struct in signature
    let script = make_script_with_imports(Signature(vec![
        SignatureToken::Bool,
        SignatureToken::Struct(StructHandleIndex(0)),
        SignatureToken::U64,
    ]));
    assert_eq!(
        call_script(script, vec![Value::u128(0)])
            .err()
            .unwrap()
            .major_status(),
        StatusCode::INVALID_MAIN_FUNCTION_SIGNATURE,
    );
    // reference to struct in signature
    let script = make_script_with_imports(Signature(vec![
        SignatureToken::Address,
        SignatureToken::MutableReference(Box::new(SignatureToken::Struct(StructHandleIndex(0)))),
    ]));
    assert_eq!(
        call_script(script, vec![Value::u128(0)])
            .err()
            .unwrap()
            .major_status(),
        StatusCode::INVALID_MAIN_FUNCTION_SIGNATURE,
    );
    // vector of struct in signature
    let script = make_script_with_imports(Signature(vec![
        SignatureToken::Bool,
        SignatureToken::Vector(Box::new(SignatureToken::Struct(StructHandleIndex(0)))),
        SignatureToken::U64,
    ]));
    assert_eq!(
        call_script(script, vec![Value::u128(0)])
            .err()
            .unwrap()
            .major_status(),
        StatusCode::INVALID_MAIN_FUNCTION_SIGNATURE,
    );
    // vector of vector of struct in signature
    let script = make_script_with_imports(Signature(vec![
        SignatureToken::Bool,
        SignatureToken::Vector(Box::new(SignatureToken::Vector(Box::new(
            SignatureToken::Struct(StructHandleIndex(0)),
        )))),
        SignatureToken::U64,
    ]));
    assert_eq!(
        call_script(script, vec![Value::u128(0)])
            .err()
            .unwrap()
            .major_status(),
        StatusCode::INVALID_MAIN_FUNCTION_SIGNATURE,
    );
    // reference to vector in signature
    let script = make_script_with_imports(Signature(vec![SignatureToken::Reference(Box::new(
        SignatureToken::Vector(Box::new(SignatureToken::Struct(StructHandleIndex(0)))),
    ))]));
    assert_eq!(
        call_script(script, vec![Value::u128(0)])
            .err()
            .unwrap()
            .major_status(),
        StatusCode::INVALID_MAIN_FUNCTION_SIGNATURE,
    );
    // reference to vector in signature
    let script = make_script_with_imports(Signature(vec![SignatureToken::Reference(Box::new(
        SignatureToken::U64,
    ))]));
    assert_eq!(
        call_script(script, vec![Value::u128(0)])
            .err()
            .unwrap()
            .major_status(),
        StatusCode::INVALID_MAIN_FUNCTION_SIGNATURE,
    );
    // `Signer` in signature (not `&Signer`)
    let script = make_script_consuming_args(Signature(vec![SignatureToken::Signer]));
    assert_eq!(
        call_script(script, vec![Value::u128(0)])
            .err()
            .unwrap()
            .major_status(),
        StatusCode::INVALID_MAIN_FUNCTION_SIGNATURE,
    );
    // vector of `Signer` in signature
    let script = make_script_consuming_args(Signature(vec![SignatureToken::Vector(Box::new(
        SignatureToken::Signer,
    ))]));
    assert_eq!(
        call_script(script, vec![Value::u128(0)])
            .err()
            .unwrap()
            .major_status(),
        StatusCode::INVALID_MAIN_FUNCTION_SIGNATURE,
    );
    // `Signer` ref not first arg
    let script = make_script(Signature(vec![
        SignatureToken::Bool,
        SignatureToken::Reference(Box::new(SignatureToken::Signer)),
    ]));
    assert_eq!(
        call_script(script, vec![Value::bool(false)])
            .err()
            .unwrap()
            .major_status(),
        StatusCode::INVALID_MAIN_FUNCTION_SIGNATURE,
    );

    //
    // Good signatures
    //

    // All constants
    let script = make_script(Signature(vec![SignatureToken::Vector(Box::new(
        SignatureToken::Bool,
    ))]));
    call_script(script, vec![Value::vector_bool(vec![true, false])]).expect("vector<bool> is good");
    let script = make_script(Signature(vec![
        SignatureToken::Bool,
        SignatureToken::Vector(Box::new(SignatureToken::U8)),
        SignatureToken::Address,
    ]));
    call_script(
        script,
        vec![
            Value::bool(true),
            Value::vector_u8(vec![0, 1]),
            Value::address(AccountAddress::random()),
        ],
    )
    .expect("vector<u8> is good");
    // signer ref
    let script = make_script(Signature(vec![
        SignatureToken::Reference(Box::new(SignatureToken::Signer)),
        SignatureToken::Bool,
        SignatureToken::Address,
    ]));
    call_script(
        script,
        vec![Value::bool(false), Value::address(AccountAddress::random())],
    )
    .expect("&Signer first argument is good");
    let script = make_script(Signature(vec![
        SignatureToken::Bool,
        SignatureToken::Vector(Box::new(SignatureToken::U8)),
        SignatureToken::Vector(Box::new(SignatureToken::Vector(Box::new(
            SignatureToken::Address,
        )))),
    ]));
    let mut addresses = vec![];
    addresses.push(Value::vector_address(vec![
        AccountAddress::random(),
        AccountAddress::random(),
    ]));
    addresses.push(Value::vector_address(vec![
        AccountAddress::random(),
        AccountAddress::random(),
    ]));
    addresses.push(Value::vector_address(vec![
        AccountAddress::random(),
        AccountAddress::random(),
    ]));
    let values = Value::constant_vector_generic(
        addresses,
        &SignatureToken::Vector(Box::new(SignatureToken::Address)),
    )
    .expect("vector<vector<address>> can be built");
    call_script(
        script,
        vec![Value::bool(true), Value::vector_u8(vec![0, 1]), values],
    )
    .expect("vector<vector<address>> is good");
}

#[test]
fn check_constant_args() {
    //
    // Simple arguments
    //

    // U128 arg, success
    let script = make_script(Signature(vec![SignatureToken::U128]));
    call_script(script, vec![Value::u128(0)]).expect("u128 is good");

    // error: no args - missing arg comes as type mismatch
    let script = make_script(Signature(vec![SignatureToken::U64]));
    assert_eq!(
        call_script(script, vec![]).err().unwrap().major_status(),
        StatusCode::TYPE_MISMATCH,
    );

    // error: too many args - too many args comes as type mismatch
    let script = make_script(Signature(vec![SignatureToken::Bool]));
    assert_eq!(
        call_script(script, vec![]).err().unwrap().major_status(),
        StatusCode::TYPE_MISMATCH,
    );

    //
    // Vector arguments
    //

    // success: vector of addresses
    let script = make_script(Signature(vec![SignatureToken::Vector(Box::new(
        SignatureToken::Address,
    ))]));
    // empty vector
    call_script(script.clone(), vec![Value::vector_address(vec![])])
        .expect("empty vector<address> is good");
    // one elem vector
    call_script(
        script.clone(),
        vec![Value::vector_address(vec![AccountAddress::random()])],
    )
    .expect("vector<address> is good");
    // multiple elems vector
    call_script(
        script.clone(),
        vec![Value::vector_address(vec![
            AccountAddress::random(),
            AccountAddress::random(),
            AccountAddress::random(),
            AccountAddress::random(),
            AccountAddress::random(),
        ])],
    )
    .expect("multiple vector<address> is good");
    // wrong vector vector<bool> passed for vector<address>
    assert_eq!(
        call_script(script.clone(), vec![Value::vector_bool(vec![true])])
            .err()
            .unwrap()
            .major_status(),
        StatusCode::TYPE_MISMATCH,
    );
    // wrong U128 passed for vector<address>
    assert_eq!(
        call_script(script, vec![Value::u128(12)])
            .err()
            .unwrap()
            .major_status(),
        StatusCode::TYPE_MISMATCH,
    );

    // vector of vector
    let script = make_script(Signature(vec![SignatureToken::Vector(Box::new(
        SignatureToken::Vector(Box::new(SignatureToken::U8)),
    ))]));
    // empty vector
    let arg = Value::constant_vector_generic(
        vec![],
        &SignatureToken::Vector(Box::new(SignatureToken::U8)),
    )
    .expect("create vector of vector");
    call_script(script.clone(), vec![arg]).expect("empty vector<vector<u8>> is good");
    // multiple elements vector
    let inner = vec![
        Value::vector_u8(vec![0, 1]),
        Value::vector_u8(vec![2, 3]),
        Value::vector_u8(vec![4, 5]),
    ];
    let arg = Value::constant_vector_generic(
        inner,
        &SignatureToken::Vector(Box::new(SignatureToken::U8)),
    )
    .expect("create vector of vector");
    call_script(script.clone(), vec![arg]).expect("vector<vector<u8>> is good");
    // wrong U8 passed for vector<U8>
    assert_eq!(
        call_script(script, vec![Value::u8(12)])
            .err()
            .unwrap()
            .major_status(),
        StatusCode::TYPE_MISMATCH,
    );
}
