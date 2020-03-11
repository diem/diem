// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::parser::ast::{FunctionName, ModuleIdent, ModuleIdent_, ModuleName, StructName};
use libra_types::account_address::AccountAddress as LibraAddress;
use move_ir_types::ast as IR;
use std::{
    clone::Clone,
    collections::{BTreeMap, BTreeSet, HashMap},
};

/// Compilation context for a single compilation unit (module or script).
/// Contains all of the dependencies actually used in the module
pub struct Context<'a> {
    current_module: Option<&'a ModuleIdent>,
    seen_structs: BTreeSet<(ModuleIdent, StructName)>,
    seen_functions: BTreeSet<(ModuleIdent, FunctionName)>,
}

impl<'a> Context<'a> {
    /// Given the dependencies and the current module, creates an empty context.
    /// The current module is a dummy `Self` for CompiledScript.
    /// It initializes an "import" of `Self` as the alias for the current_module.
    pub fn new(current_module: Option<&'a ModuleIdent>) -> Self {
        Self {
            current_module,
            seen_structs: BTreeSet::new(),
            seen_functions: BTreeSet::new(),
        }
    }

    pub fn current_module(&self) -> Option<&'a ModuleIdent> {
        self.current_module
    }

    fn is_current_module(&self, m: &ModuleIdent) -> bool {
        self.current_module.map(|cur| cur == m).unwrap_or(false)
    }

    //**********************************************************************************************
    // Dependency item building
    //**********************************************************************************************

    pub fn materialize(
        self,
        dependency_orderings: &HashMap<ModuleIdent, usize>,
        struct_declarations: &HashMap<
            (ModuleIdent, StructName),
            (bool, Vec<(IR::TypeVar, IR::Kind)>),
        >,
        function_declarations: &HashMap<(ModuleIdent, FunctionName), IR::FunctionSignature>,
    ) -> (Vec<IR::ImportDefinition>, Vec<IR::ModuleDependency>) {
        let Context {
            current_module: _current_module,
            seen_structs,
            seen_functions,
        } = self;
        let mut module_dependencies = BTreeMap::new();
        for (module, sname) in seen_structs {
            let struct_dep = Self::struct_dependency(struct_declarations, &module, sname);
            module_dependencies
                .entry(module)
                .or_insert_with(|| (vec![], vec![]))
                .0
                .push(struct_dep);
        }
        for (module, fname) in seen_functions {
            let function_dep = Self::function_dependency(function_declarations, &module, fname);
            module_dependencies
                .entry(module)
                .or_insert_with(|| (vec![], vec![]))
                .1
                .push(function_dep);
        }
        let mut imports = vec![];
        let mut ordered_dependencies = vec![];
        for (module, (structs, functions)) in module_dependencies {
            let dependency_order = dependency_orderings[&module];
            let ir_name = Self::ir_module_alias(&module);
            let ir_ident = Self::translate_module_ident(module);
            imports.push(IR::ImportDefinition::new(ir_ident, Some(ir_name.clone())));
            ordered_dependencies.push((
                dependency_order,
                IR::ModuleDependency {
                    name: ir_name,
                    structs,
                    functions,
                },
            ));
        }
        ordered_dependencies.sort_by_key(|(ordering, _)| *ordering);
        let dependencies = ordered_dependencies.into_iter().map(|(_, m)| m).collect();
        (imports, dependencies)
    }

    fn struct_dependency(
        struct_declarations: &HashMap<
            (ModuleIdent, StructName),
            (bool, Vec<(IR::TypeVar, IR::Kind)>),
        >,
        module: &ModuleIdent,
        sname: StructName,
    ) -> IR::StructDependency {
        let key = (module.clone(), sname.clone());
        let (is_nominal_resource, type_formals) = struct_declarations.get(&key).unwrap().clone();
        let name = Self::translate_struct_name(sname);
        IR::StructDependency {
            name,
            is_nominal_resource,
            type_formals,
        }
    }

    fn function_dependency(
        function_declarations: &HashMap<(ModuleIdent, FunctionName), IR::FunctionSignature>,
        module: &ModuleIdent,
        fname: FunctionName,
    ) -> IR::FunctionDependency {
        let key = (module.clone(), fname.clone());
        let signature = function_declarations.get(&key).unwrap().clone();
        let name = Self::translate_function_name(fname);
        IR::FunctionDependency { name, signature }
    }

    //**********************************************************************************************
    // Name translation
    //**********************************************************************************************

    fn ir_module_alias(ident: &ModuleIdent) -> IR::ModuleName {
        let ModuleIdent_ { address, name } = &ident.0.value;
        IR::ModuleName::new(format!("{}::{}", address, name))
    }

    fn translate_module_ident(ident: ModuleIdent) -> IR::ModuleIdent {
        let ModuleIdent_ { address, name } = ident.0.value;
        let name = Self::translate_module_name(name);
        IR::ModuleIdent::Qualified(IR::QualifiedModuleIdent::new(
            name,
            LibraAddress::new(address.to_u8()),
        ))
    }

    fn translate_module_name(n: ModuleName) -> IR::ModuleName {
        IR::ModuleName::new(n.0.value)
    }

    fn translate_struct_name(n: StructName) -> IR::StructName {
        IR::StructName::new(n.0.value)
    }

    fn translate_function_name(n: FunctionName) -> IR::FunctionName {
        IR::FunctionName::new(n.0.value)
    }

    //**********************************************************************************************
    // Name resolution
    //**********************************************************************************************

    pub fn struct_definition_name(&self, m: &ModuleIdent, s: StructName) -> IR::StructName {
        assert!(
            self.is_current_module(m),
            "ICE invalid struct definition lookup"
        );
        Self::translate_struct_name(s)
    }

    pub fn qualified_struct_name(
        &mut self,
        m: &ModuleIdent,
        s: StructName,
    ) -> IR::QualifiedStructIdent {
        let mname = if self.is_current_module(m) {
            IR::ModuleName::module_self()
        } else {
            self.seen_structs.insert((m.clone(), s.clone()));
            Self::ir_module_alias(m)
        };
        let n = Self::translate_struct_name(s);
        IR::QualifiedStructIdent::new(mname, n)
    }

    pub fn function_definition_name(
        &self,
        m: Option<&ModuleIdent>,
        f: FunctionName,
    ) -> IR::FunctionName {
        assert!(
            self.current_module == m,
            "ICE invalid function definition lookup"
        );
        Self::translate_function_name(f)
    }

    pub fn qualified_function_name(
        &mut self,
        m: &ModuleIdent,
        f: FunctionName,
    ) -> (IR::ModuleName, IR::FunctionName) {
        let mname = if self.is_current_module(m) {
            IR::ModuleName::module_self()
        } else {
            self.seen_functions.insert((m.clone(), f.clone()));
            Self::ir_module_alias(m)
        };
        let n = Self::translate_function_name(f);
        (mname, n)
    }
}
