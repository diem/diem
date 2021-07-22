// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    diag,
    expansion::ast::{self as E, Address, ModuleIdent, ModuleIdent_},
    parser::ast::ModuleName,
    shared::{unique_map::UniqueMap, *},
};
use move_ir_types::location::*;
use move_symbol_pool::Symbol;
use std::collections::BTreeMap;

//**************************************************************************************************
// Entry
//**************************************************************************************************

type RefinedModuleIdent = (Loc, Option<Name>, AddressBytes, ModuleName);

/// Verifies that modules remain unique, even after substituting named addresses for their values
pub fn verify(
    compilation_env: &mut CompilationEnv,
    modules: &UniqueMap<ModuleIdent, E::ModuleDefinition>,
) {
    let mut decl_locs: BTreeMap<(AddressBytes, Symbol), RefinedModuleIdent> = BTreeMap::new();
    for (sp!(loc, ModuleIdent_ { address, module }), _mdef) in modules.key_cloned_iter() {
        let sp!(nloc, n_) = module.0;
        let addr_name = match &address {
            Address::Anonymous(_) => None,
            Address::Named(n) => Some(*n),
        };
        let addr_bytes = match &address {
            Address::Anonymous(sp!(_, addr_bytes)) => *addr_bytes,
            Address::Named(n) => match compilation_env.named_address_mapping().get(&n.value) {
                // undeclared or no value bound, so can skip
                None => continue,
                // copy the assigned value
                Some(addr_bytes) => *addr_bytes,
            },
        };
        let mident_ = (addr_bytes, n_);
        let compiled_mident = (loc, addr_name, addr_bytes, ModuleName(sp(nloc, n_)));
        if let Some(prev) = decl_locs.insert(mident_, compiled_mident) {
            let cur = &decl_locs[&mident_];
            let (orig, duplicate) =
                if cur.0.file() == prev.0.file() && cur.0.start() > prev.0.start() {
                    (&prev, cur)
                } else {
                    (cur, &prev)
                };

            // Formatting here is a bit weird, but it is guaranteed that at least one of the
            // declarations (prev or cur) will have an address_name of Some(_)
            let format_name = |m: &RefinedModuleIdent| match &m.1 {
                None => format!("'{}::{}'", &m.2, &m.3),
                Some(aname) => format!(
                    "'{aname}::{mname}', with '{aname}' = {abytes}",
                    aname = aname,
                    abytes = &m.2,
                    mname = &m.3
                ),
            };
            let msg = format!("Duplicate definition of {}", format_name(duplicate));
            let prev_msg = format!("Module previously defined here as {}", format_name(orig));
            compilation_env.add_diag(diag!(
                Declarations::DuplicateItem,
                (duplicate.0, msg),
                (orig.0, prev_msg)
            ))
        }
    }
}
