// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! Types and data used by the VM runtime

#[macro_use]
extern crate lazy_static;

#[cfg(any(test, feature = "fuzzing"))]
mod proptest_types;

pub mod loaded_data;
pub mod native_functions;
pub mod native_structs;
pub mod type_context;
pub mod value;
