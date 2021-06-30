// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    file_format::{AbilitySet, StructTypeParameter, Visibility},
    normalized::Module,
};
use std::collections::BTreeSet;

/// The result of a linking and layout compatibility check. Here is what the different combinations
/// mean:
/// `{ struct: true, struct_layout: true }`: fully backward compatible
/// `{ struct_and_function_linking: true, struct_layout: false }`: Dependent modules that reference functions or types in this module may not link. However, fixing, recompiling, and redeploying all dependent modules will work--no data migration needed.
/// `{ type_and_function_linking: true, struct_layout: false }`: Attempting to read structs published by this module will now fail at runtime. However, dependent modules will continue to link. Requires data migration, but no changes to dependent modules.
/// `{ type_and_function_linking: false, struct_layout: false }`: Everything is broken. Need both a data migration and changes to dependent modules.
pub struct Compatibility {
    /// If false, dependent modules that reference functions or structs in this module may not link
    pub struct_and_function_linking: bool,
    /// If false, attempting to read structs previously published by this module will fail at runtime
    pub struct_layout: bool,
}

impl Compatibility {
    /// Return true if the two module s compared in the compatiblity check are both linking and
    /// layout compatible.
    pub fn is_fully_compatible(&self) -> bool {
        self.struct_and_function_linking && self.struct_layout
    }

    /// Return compatibility assessment for `new_module` relative to old module `old_module`.
    pub fn check(old_module: &Module, new_module: &Module) -> Compatibility {
        let mut struct_and_function_linking = true;
        let mut struct_layout = true;

        // module's name and address are unchanged
        if old_module.address != new_module.address || old_module.name != new_module.name {
            struct_and_function_linking = false;
        }

        // old module's structs are a subset of the new module's structs
        for (name, old_struct) in &old_module.structs {
            let new_struct = match new_module.structs.get(name) {
                Some(new_struct) => new_struct,
                None => {
                    // Struct not present in new . Existing modules that depend on this struct will fail to link with the new version of the module.
                    struct_and_function_linking = false;
                    // Note: we intentionally do *not* label this a layout compatibility violation.
                    // Existing modules can still successfully read previously published values of
                    // this struct `Parent::T`. That is, code like the function `foo` in
                    // ```
                    // struct S { t: Parent::T }
                    // public fun foo(a: addr): S { move_from<S>(addr) }
                    // ```
                    // in module `Child` will continue to run without error. But values of type
                    // `Parent::T` in `Child` are now "orphaned" in the sense that `Parent` no
                    // longer exposes any API for reading/writing them.
                    continue;
                }
            };

            if !struct_abilities_compatibile(old_struct.abilities, new_struct.abilities)
                || !struct_type_parameters_compatibile(
                    &old_struct.type_parameters,
                    &new_struct.type_parameters,
                )
            {
                struct_and_function_linking = false;
            }
            if new_struct.fields != old_struct.fields {
                // Fields changed. Code in this module will fail at runtime if it tries to
                // read a previously published struct value
                // TODO: this is a stricter definition than required. We could in principle
                // choose to label the following as compatible
                // (1) changing the name (but not position or type) of a field. The VM does
                //     not care about the name of a field (it's purely informational), but
                //     clients presumably do.
                // (2) changing the type of a field to a different, but layout and kind
                //     compatible type. E.g. `struct S { b: bool }` to `struct S { b: B }`
                // where
                //     B is struct B { some_name: bool }. TODO: does this affect clients? I
                //     think not--the serialization of the same data with these two types
                //     will be the same.
                struct_layout = false
            }
        }

        // The modules are considered as compatible function-wise when all the conditions are met:
        //
        // - old module's public functions are a subset of the new module's public functions
        //   (i.e. we cannot remove or change public functions)
        // - old module's script functions are a subset of the new module's script functions
        //   (i.e. we cannot remove or change script functions)
        // - for any friend function that is removed or changed in the old module
        //   - if the function visibility is upgraded to public, it is OK
        //   - otherwise, it is considered as incompatible.
        //
        // NOTE: it is possible to relax the compatibility checking for a friend function, i.e.,
        // we can remove/change a friend function if the function is not used by any module in the
        // friend list. But for simplicity, we decided to go to the more restrictive form now and
        // we may revisit this in the future.
        for (name, old_func) in &old_module.exposed_functions {
            let new_func = match new_module.exposed_functions.get(name) {
                Some(new_func) => new_func,
                None => {
                    struct_and_function_linking = false;
                    continue;
                }
            };
            let is_vis_compatible = match (old_func.visibility, new_func.visibility) {
                (Visibility::Public, Visibility::Public) => true,
                (Visibility::Public, _) => false,
                (Visibility::Script, Visibility::Script) => true,
                (Visibility::Script, _) => false,
                (Visibility::Friend, Visibility::Public)
                | (Visibility::Friend, Visibility::Friend) => true,
                (Visibility::Friend, _) => false,
                (Visibility::Private, _) => unreachable!("A private function can never be exposed"),
            };
            if !is_vis_compatible
                || old_func.parameters != new_func.parameters
                || old_func.return_ != new_func.return_
                || !fun_type_parameters_compatibile(
                    &old_func.type_parameters,
                    &new_func.type_parameters,
                )
            {
                struct_and_function_linking = false;
            }
        }

        // check friend declarations compatibility
        //
        // - additions to the list are allowed
        // - removals are not allowed
        //
        // NOTE: we may also relax this checking a bit in the future: we may allow the removal of
        // a module removed from the friend list if the module does not call any friend function
        // in this module.
        let old_friend_module_ids: BTreeSet<_> = old_module.friends.iter().cloned().collect();
        let new_friend_module_ids: BTreeSet<_> = new_module.friends.iter().cloned().collect();
        if !old_friend_module_ids.is_subset(&new_friend_module_ids) {
            struct_and_function_linking = false;
        }

        Compatibility {
            struct_and_function_linking,
            struct_layout,
        }
    }
}

// When upgrading, the new abilities must be a superset of the old abilities.
// Adding an ability is fine, but removing an ability could cause existing usages to fail.
fn struct_abilities_compatibile(old_abilities: AbilitySet, new_abilities: AbilitySet) -> bool {
    old_abilities.is_subset(new_abilities)
}

// When upgrading, the new type parameters must be the same length, and the new type parameter
// constraints must be compatible
fn fun_type_parameters_compatibile(
    old_type_parameters: &[AbilitySet],
    new_type_parameters: &[AbilitySet],
) -> bool {
    old_type_parameters.len() == new_type_parameters.len()
        && old_type_parameters.iter().zip(new_type_parameters).all(
            |(old_type_parameter_constraint, new_type_parameter_constraint)| {
                type_parameter_constraints_compatibile(
                    *old_type_parameter_constraint,
                    *new_type_parameter_constraint,
                )
            },
        )
}

fn struct_type_parameters_compatibile(
    old_type_parameters: &[StructTypeParameter],
    new_type_parameters: &[StructTypeParameter],
) -> bool {
    old_type_parameters.len() == new_type_parameters.len()
        && old_type_parameters.iter().zip(new_type_parameters).all(
            |(old_type_parameter, new_type_parameter)| {
                type_parameter_phantom_decl_compatibile(old_type_parameter, new_type_parameter)
                    && type_parameter_constraints_compatibile(
                        old_type_parameter.constraints,
                        new_type_parameter.constraints,
                    )
            },
        )
}

// When upgrading, the new constraints must be a subset of (or equal to) the old constraints.
// Removing an ability is fine, but adding an ability could cause existing callsites to fail
fn type_parameter_constraints_compatibile(
    old_type_constraints: AbilitySet,
    new_type_constraints: AbilitySet,
) -> bool {
    new_type_constraints.is_subset(old_type_constraints)
}

// Adding a phantom annotation to a parameter won't break clients because that can only increase the
// the set of abilities in struct instantiations. Put it differently, adding phantom declarations
// relaxes the requirements for clients.
fn type_parameter_phantom_decl_compatibile(
    old_type_parameter: &StructTypeParameter,
    new_type_parameter: &StructTypeParameter,
) -> bool {
    // old_type_paramter.is_phantom => new_type_parameter.is_phantom
    !old_type_parameter.is_phantom || new_type_parameter.is_phantom
}
