// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::native_structs::def::NativeStructType;
use libra_types::language_storage::ModuleId;
use move_core_types::identifier::IdentStr;
use vm::file_format::{Kind, StructHandleIndex};

/// Struct representing the expected definition for a native struct
pub struct NativeStruct {
    /// The expected boolean indicating if it is a nominal resource or not
    pub expected_nominal_resource: bool,
    /// The expected kind constraints of the type parameters.
    pub expected_type_formals: Vec<Kind>,
    /// The expected index for the struct
    /// Helpful for ensuring proper typing of native functions
    pub expected_index: StructHandleIndex,
    /// Kind of the NativeStruct,
    pub struct_type: NativeStructType,
}

/// Looks up the expected native struct definition from the module id (address and module) and
/// function name where it was expected to be declared
/// TODO: native structs are now deprecated. Remove them.
pub fn resolve_native_struct(
    _module: &ModuleId,
    _struct_name: &IdentStr,
) -> Option<&'static NativeStruct> {
    None
}
