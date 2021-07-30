// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    expansion::ast::{Address, ModuleIdent, ModuleIdent_, SpecId},
    hlir::ast as H,
    parser::ast::{ConstantName, FunctionName, StructName, Var},
    shared::{AddressBytes, CompilationEnv},
};
use move_core_types::account_address::AccountAddress as MoveAddress;
use move_ir_types::ast as IR;
use move_symbol_pool::Symbol;
use std::{
    clone::Clone,
    collections::{BTreeMap, BTreeSet, HashMap},
};
use IR::Ability;

/// Compilation context for a single compilation unit (module or script).
/// Contains all of the dependencies actually used in the module
pub struct Context<'a> {
    pub env: &'a mut CompilationEnv,
    current_module: Option<&'a ModuleIdent>,
    seen_structs: BTreeSet<(ModuleIdent, StructName)>,
    seen_functions: BTreeSet<(ModuleIdent, FunctionName)>,
    spec_info: BTreeMap<SpecId, (IR::NopLabel, BTreeMap<Var, H::SingleType>)>,
}

impl<'a> Context<'a> {
    pub fn new(env: &'a mut CompilationEnv, current_module: Option<&'a ModuleIdent>) -> Self {
        Self {
            env,
            current_module,
            seen_structs: BTreeSet::new(),
            seen_functions: BTreeSet::new(),
            spec_info: BTreeMap::new(),
        }
    }

    pub fn current_module(&self) -> Option<&'a ModuleIdent> {
        self.current_module
    }

    fn is_current_module(&self, m: &ModuleIdent) -> bool {
        self.current_module.map(|cur| cur == m).unwrap_or(false)
    }

    pub fn finish_function(
        &mut self,
    ) -> BTreeMap<SpecId, (IR::NopLabel, BTreeMap<Var, H::SingleType>)> {
        std::mem::take(&mut self.spec_info)
    }

    //**********************************************************************************************
    // Dependency item building
    //**********************************************************************************************

    pub fn materialize(
        self,
        dependency_orderings: &HashMap<ModuleIdent, usize>,
        struct_declarations: &HashMap<
            (ModuleIdent, StructName),
            (BTreeSet<IR::Ability>, Vec<IR::StructTypeParameter>),
        >,
        function_declarations: &HashMap<
            (ModuleIdent, FunctionName),
            (BTreeSet<(ModuleIdent, StructName)>, IR::FunctionSignature),
        >,
    ) -> (Vec<IR::ImportDefinition>, Vec<IR::ModuleDependency>) {
        let Context {
            env,
            current_module: _current_module,
            mut seen_structs,
            seen_functions,
            ..
        } = self;
        let mut module_dependencies = BTreeMap::new();
        Self::function_dependencies(
            function_declarations,
            &mut module_dependencies,
            &mut seen_structs,
            seen_functions,
        );
        Self::struct_dependencies(struct_declarations, &mut module_dependencies, seen_structs);
        let mut imports = vec![];
        let mut ordered_dependencies = vec![];
        for (module, (structs, functions)) in module_dependencies {
            let dependency_order = dependency_orderings[&module];
            let ir_name = Self::ir_module_alias(&module);
            let ir_ident = Self::translate_module_ident_impl(env.named_address_mapping(), module);
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

    fn insert_struct_dependency(
        module_dependencies: &mut BTreeMap<
            ModuleIdent,
            (Vec<IR::StructDependency>, Vec<IR::FunctionDependency>),
        >,
        module: ModuleIdent,
        struct_dep: IR::StructDependency,
    ) {
        module_dependencies
            .entry(module)
            .or_insert_with(|| (vec![], vec![]))
            .0
            .push(struct_dep);
    }

    fn insert_function_dependency(
        module_dependencies: &mut BTreeMap<
            ModuleIdent,
            (Vec<IR::StructDependency>, Vec<IR::FunctionDependency>),
        >,
        module: ModuleIdent,
        function_dep: IR::FunctionDependency,
    ) {
        module_dependencies
            .entry(module)
            .or_insert_with(|| (vec![], vec![]))
            .1
            .push(function_dep);
    }

    fn struct_dependencies(
        struct_declarations: &HashMap<
            (ModuleIdent, StructName),
            (BTreeSet<Ability>, Vec<IR::StructTypeParameter>),
        >,
        module_dependencies: &mut BTreeMap<
            ModuleIdent,
            (Vec<IR::StructDependency>, Vec<IR::FunctionDependency>),
        >,
        seen_structs: BTreeSet<(ModuleIdent, StructName)>,
    ) {
        for (module, sname) in seen_structs {
            let struct_dep = Self::struct_dependency(struct_declarations, &module, sname);
            Self::insert_struct_dependency(module_dependencies, module, struct_dep);
        }
    }

    fn struct_dependency(
        struct_declarations: &HashMap<
            (ModuleIdent, StructName),
            (BTreeSet<Ability>, Vec<IR::StructTypeParameter>),
        >,
        module: &ModuleIdent,
        sname: StructName,
    ) -> IR::StructDependency {
        let key = (*module, sname);
        let (abilities, type_formals) = struct_declarations.get(&key).unwrap().clone();
        let name = Self::translate_struct_name(sname);
        IR::StructDependency {
            abilities,
            name,
            type_formals,
        }
    }

    fn function_dependencies(
        function_declarations: &HashMap<
            (ModuleIdent, FunctionName),
            (BTreeSet<(ModuleIdent, StructName)>, IR::FunctionSignature),
        >,
        module_dependencies: &mut BTreeMap<
            ModuleIdent,
            (Vec<IR::StructDependency>, Vec<IR::FunctionDependency>),
        >,
        seen_structs: &mut BTreeSet<(ModuleIdent, StructName)>,
        seen_functions: BTreeSet<(ModuleIdent, FunctionName)>,
    ) {
        for (module, fname) in seen_functions {
            let (functions_seen_structs, function_dep) =
                Self::function_dependency(function_declarations, &module, fname);
            Self::insert_function_dependency(module_dependencies, module, function_dep);
            seen_structs.extend(functions_seen_structs)
        }
    }

    fn function_dependency(
        function_declarations: &HashMap<
            (ModuleIdent, FunctionName),
            (BTreeSet<(ModuleIdent, StructName)>, IR::FunctionSignature),
        >,
        module: &ModuleIdent,
        fname: FunctionName,
    ) -> (BTreeSet<(ModuleIdent, StructName)>, IR::FunctionDependency) {
        let key = (*module, fname);
        let (seen_structs, signature) = function_declarations.get(&key).unwrap().clone();
        let name = Self::translate_function_name(fname);
        (seen_structs, IR::FunctionDependency { name, signature })
    }

    //**********************************************************************************************
    // Name translation
    //**********************************************************************************************

    fn ir_module_alias(sp!(_, ModuleIdent_ { address, module }): &ModuleIdent) -> IR::ModuleName {
        IR::ModuleName(format!("{}::{}", address, module).into())
    }

    pub fn resolve_address(&self, addr: Address) -> AddressBytes {
        addr.into_addr_bytes(self.env.named_address_mapping())
    }

    pub fn translate_module_ident(&self, ident: ModuleIdent) -> IR::ModuleIdent {
        Self::translate_module_ident_impl(self.env.named_address_mapping(), ident)
    }

    fn translate_module_ident_impl(
        addresses: &BTreeMap<Symbol, AddressBytes>,
        sp!(_, ModuleIdent_ { address, module }): ModuleIdent,
    ) -> IR::ModuleIdent {
        let address_bytes = address.into_addr_bytes(addresses);
        let name = Self::translate_module_name_(module.0.value);
        IR::ModuleIdent::Qualified(IR::QualifiedModuleIdent::new(
            name,
            MoveAddress::new(address_bytes.into_bytes()),
        ))
    }

    fn translate_module_name_(s: Symbol) -> IR::ModuleName {
        IR::ModuleName(s)
    }

    fn translate_struct_name(n: StructName) -> IR::StructName {
        IR::StructName(n.0.value)
    }

    fn translate_constant_name(n: ConstantName) -> IR::ConstantName {
        IR::ConstantName(n.0.value)
    }

    fn translate_function_name(n: FunctionName) -> IR::FunctionName {
        IR::FunctionName(n.0.value)
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
            self.seen_structs.insert((*m, s));
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
            self.seen_functions.insert((*m, f));
            Self::ir_module_alias(m)
        };
        let n = Self::translate_function_name(f);
        (mname, n)
    }

    pub fn constant_definition_name(
        &self,
        m: Option<&ModuleIdent>,
        f: ConstantName,
    ) -> IR::ConstantName {
        assert!(
            self.current_module == m,
            "ICE invalid constant definition lookup"
        );
        Self::translate_constant_name(f)
    }

    pub fn constant_name(&mut self, f: ConstantName) -> IR::ConstantName {
        Self::translate_constant_name(f)
    }

    //**********************************************************************************************
    // Nops
    //**********************************************************************************************

    pub fn spec(&mut self, id: SpecId, used_locals: BTreeMap<Var, H::SingleType>) -> IR::NopLabel {
        let label = IR::NopLabel(format!("{}", id).into());
        assert!(self
            .spec_info
            .insert(id, (label.clone(), used_locals))
            .is_none());
        label
    }
}
