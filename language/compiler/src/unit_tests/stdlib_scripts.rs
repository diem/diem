// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::unit_tests::testutils::{
    compile_module_string, compile_module_string_with_deps, compile_script_string_with_stdlib,
};

#[test]
fn compile_native_hash() {
    let code = include_str!("../../../stdlib/modules/hash.mvir");
    let _compiled_module = compile_module_string(&code).unwrap();
}

#[test]
fn compile_libra_coin() {
    let code = include_str!("../../../stdlib/modules/libra_coin.mvir");
    let _compiled_module = compile_module_string(&code).unwrap();
}

#[test]
fn compile_account_module() {
    let address_util_code = include_str!("../../../stdlib/modules/address_util.mvir");
    let u64_util_code = include_str!("../../../stdlib/modules/u64_util.mvir");
    let bytearray_util_code = include_str!("../../../stdlib/modules/bytearray_util.mvir");

    let hash_code = include_str!("../../../stdlib/modules/hash.mvir");
    let coin_code = include_str!("../../../stdlib/modules/libra_coin.mvir");
    let account_code = include_str!("../../../stdlib/modules/libra_account.mvir");

    let address_util_module = compile_module_string(address_util_code).unwrap();
    let u64_util_module = compile_module_string(u64_util_code).unwrap();
    let bytearray_util_module = compile_module_string(bytearray_util_code).unwrap();
    let hash_module = compile_module_string(hash_code).unwrap();

    let coin_module = compile_module_string(coin_code).unwrap();

    let _compiled_module = compile_module_string_with_deps(
        account_code,
        vec![
            hash_module,
            address_util_module,
            u64_util_module,
            bytearray_util_module,
            coin_module,
        ],
    )
    .unwrap();
}

#[test]
fn compile_create_account_script() {
    let code = include_str!("../../../stdlib/transaction_scripts/create_account.mvir");
    let _compiled_script = compile_script_string_with_stdlib(code).unwrap();
}

#[test]
fn compile_mint_script() {
    let code = include_str!("../../../stdlib/transaction_scripts/mint.mvir");
    let _compiled_script = compile_script_string_with_stdlib(code).unwrap();
}

#[test]
fn compile_rotate_authentication_key_script() {
    let code = include_str!("../../../stdlib/transaction_scripts/rotate_authentication_key.mvir");
    let _compiled_script = compile_script_string_with_stdlib(code).unwrap();
}

#[test]
fn compile_peer_to_peer_transfer_script() {
    let code = include_str!("../../../stdlib/transaction_scripts/peer_to_peer_transfer.mvir");
    let _compiled_script = compile_script_string_with_stdlib(code).unwrap();
}
