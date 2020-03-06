// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Defines constants and types that are used throughout cost synthesis.
use move_vm_types::values::Value;
use vm::file_format::TableIndex;

/// The default index to use when we need to have a frame on the execution stack.
///
/// We are always guaranteed to have at least one function definition in a generated module. We can
/// therefore always count on having a function definition at index 0.
pub const DEFAULT_FUNCTION_IDX: TableIndex = 0;

/// The type of the value stack.
pub type Stack = Vec<Value>;
