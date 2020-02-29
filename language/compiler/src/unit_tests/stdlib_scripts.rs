// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::unit_tests::testutils::{compile_module_string, compile_module_string_with_deps};

#[test]
fn compile_native_hash() {
    let code = include_str!("../ir_stdlib/modules/hash.mvir");
    let _compiled_module = compile_module_string(&code).unwrap();
}

#[test]
fn compile_libra_coin() {
    let code = include_str!("../ir_stdlib/modules/libra_coin.mvir");
    let _compiled_module = compile_module_string(&code).unwrap();
}

#[test]
fn compile_account_module() {
    let vector_code = include_str!("../ir_stdlib/modules/vector.mvir");
    let address_util_code = include_str!("../ir_stdlib/modules/address_util.mvir");
    let u64_util_code = include_str!("../ir_stdlib/modules/u64_util.mvir");
    let bytearray_util_code = include_str!("../ir_stdlib/modules/bytearray_util.mvir");

    let hash_code = include_str!("../ir_stdlib/modules/hash.mvir");
    let coin_code = include_str!("../ir_stdlib/modules/libra_coin.mvir");
    let time_code = include_str!("../ir_stdlib/modules/libra_time.mvir");
    let ttl_code = include_str!("../ir_stdlib/modules/libra_transaction_timeout.mvir");
    let account_code = include_str!("../ir_stdlib/modules/libra_account.mvir");

    let vector_module = compile_module_string(vector_code).unwrap();
    let address_util_module = compile_module_string(address_util_code).unwrap();
    let u64_util_module = compile_module_string(u64_util_code).unwrap();
    let bytearray_util_module = compile_module_string(bytearray_util_code).unwrap();
    let hash_module = compile_module_string(hash_code).unwrap();
    let time_module = compile_module_string(time_code).unwrap();
    let ttl_module = compile_module_string_with_deps(ttl_code, vec![time_module]).unwrap();

    let coin_module = compile_module_string(coin_code).unwrap();

    let _compiled_module = compile_module_string_with_deps(
        account_code,
        vec![
            vector_module,
            hash_module,
            address_util_module,
            u64_util_module,
            bytearray_util_module,
            coin_module,
            ttl_module,
        ],
    )
    .unwrap();
}
