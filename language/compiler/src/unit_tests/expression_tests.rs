// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[macro_use]
mod testutils;
use super::*;
use testutils::{
    compile_module_string, compile_script_string, compile_script_string_and_assert_error,
    count_locals,
};
use vm::{
    access::{BaseAccess, ScriptAccess},
    file_format::Bytecode::*,
};

#[test]
fn compile_script_expr_addition() {
    let code = String::from(
        "
        main() {
            let x: u64;
            let y: u64;
            let z: u64;
            x = 3;
            y = 5;
            z = move(x) + move(y);
            return;
        }
        ",
    );
    let compiled_script_res = compile_script_string(&code);
    let compiled_script = compiled_script_res.unwrap();
    assert!(compiled_script.main().code.max_stack_size == 2);
    assert!(count_locals(&compiled_script) == 3);
    assert!(compiled_script.main().code.code.len() == 9);
    assert!(compiled_script.struct_handles().len() == 0);
    assert!(compiled_script.function_handles().len() == 1);
    assert!(compiled_script.type_signatures().len() == 0);
    assert!(compiled_script.function_signatures().len() == 1); // method sig
    assert!(compiled_script.locals_signatures().len() == 1); // local variables sig
    assert!(compiled_script.module_handles().len() == 1); // the <SELF> module
    assert!(compiled_script.string_pool().len() == 2); // the name of `main()` + the name of the "<SELF>" module
    assert!(compiled_script.address_pool().len() == 1); // the empty address of <SELF> module
}

#[test]
fn compile_script_expr_combined() {
    let code = String::from(
        "
        main() {
            let x: u64;
            let y: u64;
            let z: u64;
            x = 3;
            y = 5;
            z = move(x) + copy(y) * 5 - copy(y);
            return;
        }
        ",
    );
    let compiled_script_res = compile_script_string(&code);
    let compiled_script = compiled_script_res.unwrap();
    assert!(compiled_script.main().code.max_stack_size == 3);
    assert!(count_locals(&compiled_script) == 3);
    assert!(compiled_script.main().code.code.len() == 13);
    assert!(compiled_script.struct_handles().len() == 0);
    assert!(compiled_script.function_handles().len() == 1);
    assert!(compiled_script.type_signatures().len() == 0);
    assert!(compiled_script.function_signatures().len() == 1); // method sig
    assert!(compiled_script.locals_signatures().len() == 1); // local variables sig
    assert!(compiled_script.module_handles().len() == 1); // the <SELF> module
    assert!(compiled_script.string_pool().len() == 2); // the name of `main()` + the name of the "<SELF>" module
    assert!(compiled_script.address_pool().len() == 1); // the empty address of <SELF> module
}

#[test]
fn compile_script_borrow_local() {
    let code = String::from(
        "
        main() {
            let x: u64;
            let ref_x: &u64;
            x = 3;
            ref_x = &x;
            release(move(ref_x));
            return;
        }
        ",
    );
    let compiled_script_res = compile_script_string(&code);
    let compiled_script = compiled_script_res.unwrap();
    assert!(count_locals(&compiled_script) == 2);
    assert!(compiled_script.struct_handles().len() == 0);
    assert!(compiled_script.function_handles().len() == 1);
    assert!(compiled_script.type_signatures().len() == 0);
    assert!(compiled_script.function_signatures().len() == 1); // method sig
    assert!(compiled_script.locals_signatures().len() == 1); // local variables sig
    assert!(compiled_script.module_handles().len() == 1); // the <SELF> module
    assert!(compiled_script.string_pool().len() == 2); // the name of `main()` + the name of the "<SELF>" module
    assert!(compiled_script.address_pool().len() == 1); // the empty address of <SELF> module
}

#[test]
fn compile_script_borrow_local_mutable() {
    let code = String::from(
        "
        main() {
            let x: u64;
            let ref_x: &mut u64;
            x = 3;
            ref_x = &mut x;
            *move(ref_x) = 42;
            return;
        }
        ",
    );
    let compiled_script_res = compile_script_string(&code);
    let compiled_script = compiled_script_res.unwrap();
    assert!(count_locals(&compiled_script) == 2);
    assert!(compiled_script.struct_handles().len() == 0);
    assert!(compiled_script.function_handles().len() == 1);
    assert!(compiled_script.type_signatures().len() == 0);
    assert!(compiled_script.function_signatures().len() == 1); // method sig
    assert!(compiled_script.locals_signatures().len() == 1); // local variables sig
    assert!(compiled_script.module_handles().len() == 1); // the <SELF> module
    assert!(compiled_script.string_pool().len() == 2); // the name of `main()` + the name of the "<SELF>" module
    assert!(compiled_script.address_pool().len() == 1); // the empty address of <SELF> module
}

#[test]
fn compile_script_borrow_reference() {
    let code = String::from(
        "
        main() {
            let x: u64;
            let ref_x: &u64;
            let ref_ref_x: &u64;
            x = 3;
            ref_x = &x;
            ref_ref_x = &ref_x;
            return;
        }
        ",
    );
    let compiled_script_res = compile_script_string_and_assert_error(&code, None);
    let compiled_script = compiled_script_res.unwrap();
    assert!(count_locals(&compiled_script) == 3);
    assert!(compiled_script.struct_handles().len() == 0);
    assert!(compiled_script.function_handles().len() == 1);
    assert!(compiled_script.type_signatures().len() == 0);
    assert!(compiled_script.function_signatures().len() == 1); // method sig
    assert!(compiled_script.locals_signatures().len() == 1); // local variables sig
    assert!(compiled_script.module_handles().len() == 1); // the <SELF> module
    assert!(compiled_script.string_pool().len() == 2); // the name of `main()` + the name of the "<SELF>" module
    assert!(compiled_script.address_pool().len() == 1); // the empty address of <SELF> module
}

#[test]
fn compile_assert() {
    let code = String::from(
        "
        main() {
            let x: u64;
            x = 3;
            assert(copy(x) > 2, 42);
            return;
        }
        ",
    );
    let compiled_script_res = compile_script_string(&code);
    let _compiled_script = compiled_script_res.unwrap();
}

#[test]
fn single_resource() {
    let code = String::from(
        "
module Test {
    resource T { i: u64 }

    public new_t(): R#Self.T {
        return T { i: 0 };
    }
}",
    );
    let compiled_module = compile_module_string(&code).unwrap();
    assert!(compiled_module.struct_handles().len() == 1);
}

#[test]
fn compile_immutable_borrow_local() {
    let code = String::from(
        "
        main() {
            let x: u64;
            let ref_x: &u64;

            x = 5;
            ref_x = &x;

            release(move(ref_x));

            return;
        }
        ",
    );
    let compiled_script_res = compile_script_string(&code);
    let compiled_script = compiled_script_res.unwrap();
    assert!(instr_count!(compiled_script, FreezeRef) == 1);
}

#[test]
fn compile_borrow_field() {
    let code = String::from(
        "
        module Foobar {
            resource FooCoin { value: u64 }

            public borrow_immut_field(arg: &R#Self.FooCoin) {
                let field_ref: &u64;
                field_ref = &move(arg).value;
                release(move(field_ref));
                return;
            }

            public borrow_immut_field_from_mut_ref(arg: &mut R#Self.FooCoin) {
                let field_ref: &u64;
                field_ref = &move(arg).value;
                release(move(field_ref));
                return;
            }

            public borrow_mut_field(arg: &mut R#Self.FooCoin) {
                let field_ref: &mut u64;
                field_ref = &mut move(arg).value;
                release(move(field_ref));
                return;
            }
        }
        ",
    );
    let compiled_module_res = compile_module_string(&code);
    let _compiled_module = compiled_module_res.unwrap();
}
