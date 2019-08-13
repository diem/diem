// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Types and data used by the VM runtime

#[macro_use]
extern crate lazy_static;

mod value_serializer;

#[cfg(any(test, feature = "testing"))]
mod proptest_types;

pub mod loaded_data;
pub mod native_functions;
pub mod native_structs;
pub mod value;
