// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    loaded_data::types::Type,
    native_structs::def::{NativeStructTag, NativeStructType},
};
use libra_types::{
    account_config,
    identifier::{IdentStr, Identifier},
    language_storage::ModuleId,
};
use once_cell::sync::Lazy;
use std::collections::HashMap;
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
pub fn resolve_native_struct(
    module: &ModuleId,
    struct_name: &IdentStr,
) -> Option<&'static NativeStruct> {
    NATIVE_STRUCT_MAP.get(module)?.get(struct_name)
}

macro_rules! add {
    ($m:ident, $addr:expr, $module:expr, $name:expr, $resource: expr, $ty_kinds: expr, $tag: expr) => {{
        let ty_args = $ty_kinds
            .iter()
            .enumerate()
            .map(|(id, _)| Type::TypeVariable(id as u16))
            .collect();
        let id = ModuleId::new($addr, Identifier::new($module).unwrap());
        let struct_table = $m.entry(id).or_insert_with(HashMap::new);
        let expected_index = StructHandleIndex(struct_table.len() as u16);

        let s = NativeStruct {
            expected_nominal_resource: $resource,
            expected_type_formals: $ty_kinds,
            expected_index,
            struct_type: NativeStructType::new($tag, ty_args),
        };
        let old = struct_table.insert(Identifier::new($name).unwrap(), s);
        assert!(old.is_none());
    }};
}

type NativeStructMap = HashMap<ModuleId, HashMap<Identifier, NativeStruct>>;

static NATIVE_STRUCT_MAP: Lazy<NativeStructMap> = Lazy::new(|| {
    let mut m: NativeStructMap = HashMap::new();
    let addr = account_config::CORE_CODE_ADDRESS;
    add!(
        m,
        addr,
        "Vector",
        "T",
        false,
        vec![Kind::All],
        NativeStructTag::Vector
    );
    m
});
