// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    compiled_unit::CompiledModuleIdent,
    expansion::ast::{self as E, Address, ModuleIdent, ModuleIdent_},
    parser::ast::ModuleName,
    shared::{unique_map::UniqueMap, *},
};
use move_ir_types::location::*;
use std::collections::BTreeMap;

//**************************************************************************************************
// Entry
//**************************************************************************************************

/// Verifies that modules remain unique, even after substituting named addresses for their values
pub fn verify(
    compilation_env: &mut CompilationEnv,
    addresses: &UniqueMap<Name, AddressBytes>,
    modules: &UniqueMap<ModuleIdent, E::ModuleDefinition>,
) {
    let mut decl_locs: BTreeMap<(AddressBytes, String), CompiledModuleIdent> = BTreeMap::new();
    for (sp!(loc, ModuleIdent_ { address, module }), _mdef) in modules.key_cloned_iter() {
        let sp!(nloc, n_) = module.0;
        let addr_name = match &address {
            Address::Anonymous(_) => None,
            Address::Named(n) => Some(n.clone()),
        };
        let addr_bytes = match address.to_addr_bytes_opt(addresses) {
            None => continue,
            Some(addr_bytes) => addr_bytes,
        };
        let mident_ = (addr_bytes, n_.clone());
        let compiled_mident =
            CompiledModuleIdent::new(loc, addr_name, addr_bytes, ModuleName(sp(nloc, n_)));
        if let Some(prev) = decl_locs.insert(mident_.clone(), compiled_mident) {
            let cur = &decl_locs[&mident_];
            // Formatting here is a bit weird, but it is guaranteed that at least one of the
            // declarations (prev or cur) will have an address_name of Some(_)
            let format_name = |m: &CompiledModuleIdent| match &m.address_name {
                None => format!("'{}::{}'", &m.address_bytes, &m.module_name),
                Some(aname) => format!(
                    "'{aname}::{mname}', with '{aname}' = {abytes}",
                    aname = aname,
                    abytes = &m.address_bytes,
                    mname = &m.module_name
                ),
            };
            let msg = format!("Duplicate definition of {}", format_name(&cur));
            let prev_msg = format!("Previously defined here as {}", format_name(&prev));
            compilation_env.add_error(vec![(cur.loc, msg), (prev.loc, prev_msg)]);
        }
    }
}
