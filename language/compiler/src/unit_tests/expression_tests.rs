// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::unit_tests::testutils::{
    compile_module_string, compile_script_string, compile_script_string_and_assert_error,
    count_locals,
};
use move_binary_format::{
    access::{ModuleAccess, ScriptAccess},
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
    assert_eq!(count_locals(&compiled_script), 3);
    assert_eq!(compiled_script.code().code.len(), 9);
    assert!(compiled_script.struct_handles().is_empty());
    assert_eq!(compiled_script.function_handles().len(), 0);
    assert_eq!(compiled_script.signatures().len(), 2);
    assert_eq!(compiled_script.module_handles().len(), 0);
    assert_eq!(compiled_script.identifiers().len(), 0);
    assert_eq!(compiled_script.address_identifiers().len(), 0);
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
    assert_eq!(count_locals(&compiled_script), 3);
    assert_eq!(compiled_script.code().code.len(), 13);
    assert!(compiled_script.struct_handles().is_empty());
    assert_eq!(compiled_script.function_handles().len(), 0);
    assert_eq!(compiled_script.signatures().len(), 2);
    assert_eq!(compiled_script.module_handles().len(), 0);
    assert_eq!(compiled_script.identifiers().len(), 0);
    assert_eq!(compiled_script.address_identifiers().len(), 0);
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
            _ = move(ref_x);
            return;
        }
        ",
    );
    let compiled_script_res = compile_script_string(&code);
    let compiled_script = compiled_script_res.unwrap();
    assert_eq!(count_locals(&compiled_script), 2);
    assert!(compiled_script.struct_handles().is_empty());
    assert_eq!(compiled_script.function_handles().len(), 0);
    assert_eq!(compiled_script.signatures().len(), 2);
    assert_eq!(compiled_script.module_handles().len(), 0);
    assert_eq!(compiled_script.identifiers().len(), 0);
    assert_eq!(compiled_script.address_identifiers().len(), 0);
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
    assert_eq!(count_locals(&compiled_script), 2);
    assert!(compiled_script.struct_handles().is_empty());
    assert_eq!(compiled_script.function_handles().len(), 0);
    assert_eq!(compiled_script.signatures().len(), 2);
    assert_eq!(compiled_script.module_handles().len(), 0);
    assert_eq!(compiled_script.identifiers().len(), 0);
    assert_eq!(compiled_script.address_identifiers().len(), 0);
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
    let compiled_script_res = compile_script_string_and_assert_error(&code, vec![]);
    let compiled_script = compiled_script_res.unwrap();
    assert_eq!(count_locals(&compiled_script), 3);
    assert!(compiled_script.struct_handles().is_empty());
    assert_eq!(compiled_script.function_handles().len(), 0);
    assert_eq!(compiled_script.signatures().len(), 2);
    assert_eq!(compiled_script.module_handles().len(), 0);
    assert_eq!(compiled_script.identifiers().len(), 0);
    assert_eq!(compiled_script.address_identifiers().len(), 0);
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
module 0xf1.Test {
    struct T { i: u64 }

    public new_t(): Self.T {
        return T { i: 0 };
    }
}",
    );
    let compiled_module = compile_module_string(&code).unwrap();
    assert_eq!(compiled_module.struct_handles().len(), 1);
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

            _ = move(ref_x);

            return;
        }
        ",
    );
    let compiled_script_res = compile_script_string(&code);
    let compiled_script = compiled_script_res.unwrap();
    assert!(instr_count!(compiled_script, ImmBorrowLoc(_)) == 1);
}

#[test]
fn compile_borrow_field() {
    let code = String::from(
        "
        module 0x3d.Foobar {
            struct FooCoin { value: u64 }

            public borrow_immut_field(arg: &Self.FooCoin) {
                let field_ref: &u64;
                field_ref = &move(arg).FooCoin::value;
                _ = move(field_ref);
                return;
            }

            public borrow_immut_field_from_mut_ref(arg: &mut Self.FooCoin) {
                let field_ref: &u64;
                field_ref = &move(arg).FooCoin::value;
                _ = move(field_ref);
                return;
            }

            public borrow_mut_field(arg: &mut Self.FooCoin) {
                let field_ref: &mut u64;
                field_ref = &mut move(arg).FooCoin::value;
                _ = move(field_ref);
                return;
            }
        }
        ",
    );
    let compiled_module_res = compile_module_string(&code);
    let _compiled_module = compiled_module_res.unwrap();
}

#[test]
fn compile_borrow_field_generic() {
    let code = String::from(
        "
        module 0x4d.Foobar {
            struct FooCoin<T> { value: u64 }

            public borrow_immut_field(arg: &Self.FooCoin<u64>) {
                let field_ref: &u64;
                field_ref = &move(arg).FooCoin<u64>::value;
                _ = move(field_ref);
                return;
            }

            public borrow_immut_field_from_mut_ref(arg: &mut Self.FooCoin<u128>) {
                let field_ref: &u64;
                field_ref = &move(arg).FooCoin<u128>::value;
                _ = move(field_ref);
                return;
            }

            public borrow_mut_field(arg: &mut Self.FooCoin<address>) {
                let field_ref: &mut u64;
                field_ref = &mut move(arg).FooCoin<address>::value;
                _ = move(field_ref);
                return;
            }
        }
        ",
    );
    let compiled_module_res = compile_module_string(&code);
    let _compiled_module = compiled_module_res.unwrap();
}

#[test]
fn compile_builtin_vector_ops() {
    let code = String::from(
        "
        module 0xfab.Foobar {
            public vector_ops() {
                let v: vector<u64>;
                let v_imm: &vector<u64>;
                let v_mut: &mut vector<u64>;
                let v_len: u64;
                let e_imm: &u64;
                let e_mut: &mut u64;

                v = vec_pack_0<u64>();
                v_imm = &v;
                v_len = vec_len<u64>(copy(v_imm));
                _ = move(v_imm);

                v_mut = &mut v;
                vec_push_back<u64>(copy(v_mut), 0);
                vec_push_back<u64>(copy(v_mut), 1);

                e_imm = vec_imm_borrow<u64>(copy(v_mut), 0);
                _ = move(e_imm);
                e_mut = vec_mut_borrow<u64>(copy(v_mut), 1);
                _ = move(e_mut);

                vec_swap<u64>(copy(v_mut), 0, 1);
                _ = vec_pop_back<u64>(copy(v_mut));
                _ = vec_pop_back<u64>(copy(v_mut));
                _ = move(v_mut);

                vec_unpack_0<u64>(move(v));
                return;
            }
        }
        ",
    );
    assert!(compile_module_string(&code).is_ok());
}
