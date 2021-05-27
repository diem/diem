// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod bcs;
pub mod event;
pub mod hash;
pub mod signer;
pub mod vector;

#[cfg(feature = "testing")]
pub mod unit_test;

#[cfg(feature = "testing")]
pub mod debug;

use move_core_types::{account_address::AccountAddress, identifier::Identifier};
use move_vm_runtime::native_functions::NativeFunction;

pub fn all_natives(
    move_std_addr: AccountAddress,
) -> Vec<(AccountAddress, Identifier, Identifier, NativeFunction)> {
    const NATIVES: &[(&str, &str, NativeFunction)] = &[
        ("BCS", "to_bytes", bcs::native_to_bytes),
        (
            "Event",
            "write_to_event_store",
            event::native_write_to_event_store,
        ),
        ("Hash", "sha2_256", hash::native_sha2_256),
        ("Hash", "sha3_256", hash::native_sha3_256),
        ("Signer", "borrow_address", signer::native_borrow_address),
        ("Vector", "length", vector::native_length),
        ("Vector", "empty", vector::native_empty),
        ("Vector", "borrow", vector::native_borrow),
        ("Vector", "borrow_mut", vector::native_borrow),
        ("Vector", "push_back", vector::native_push_back),
        ("Vector", "pop_back", vector::native_pop),
        ("Vector", "destroy_empty", vector::native_destroy_empty),
        ("Vector", "swap", vector::native_swap),
        #[cfg(feature = "testing")]
        ("Debug", "print", debug::native_print),
        #[cfg(feature = "testing")]
        (
            "Debug",
            "print_stack_trace",
            debug::native_print_stack_trace,
        ),
        #[cfg(feature = "testing")]
        (
            "UnitTest",
            "create_signers_for_testing",
            unit_test::native_create_signers_for_testing,
        ),
    ];
    NATIVES
        .iter()
        .cloned()
        .map(|(module_name, func_name, func)| {
            (
                move_std_addr,
                Identifier::new(module_name).unwrap(),
                Identifier::new(func_name).unwrap(),
                func,
            )
        })
        .collect()
}
