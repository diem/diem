// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{options::ModuleGeneratorOptions, padding::Pad, utils::random_string};
use bytecode_verifier::verify_module;
use ir_to_bytecode::compiler::compile_module;
use libra_types::account_address::AccountAddress;
use move_ir_types::{ast::*, location::*};
use rand::{rngs::StdRng, Rng};
use std::collections::{BTreeSet, VecDeque};
use vm::file_format::CompiledModule;

type Set<K> = BTreeSet<K>;

macro_rules! init {
    ($len:expr, $e:expr) => {
        (0..$len).map(|_| $e).collect()
    };
}

pub fn generate_module(rng: &mut StdRng, options: ModuleGeneratorOptions) -> CompiledModule {
    generate_modules(rng, 1, options).0
}

/// Generate a `number - 1` modules. Then generate a root module that imports all of these modules.
pub fn generate_modules(
    rng: &mut StdRng,
    number: usize,
    options: ModuleGeneratorOptions,
) -> (CompiledModule, Vec<CompiledModule>) {
    assert!(number > 0, "We cannot generate zero modules");

    let table_size = options.min_table_size;
    let (callee_names, callees): (Set<String>, Vec<ModuleDefinition>) = (0..(number - 1))
        .map(|_| {
            let module = ModuleGenerator::create(rng, options.clone(), &Set::new());
            let module_name = module.name.as_inner().to_string();
            (module_name, module)
        })
        .unzip();

    let root_module = ModuleGenerator::create(rng, options.clone(), &callee_names);
    let empty_deps: Vec<CompiledModule> = Vec::new();
    let compiled_callees = callees
        .into_iter()
        .map(|module| {
            let mut module = compile_module(AccountAddress::ZERO, module, &empty_deps)
                .unwrap()
                .0
                .into_inner();
            Pad::pad(table_size, &mut module, options.clone());
            module.freeze().unwrap()
        })
        .collect();

    let mut compiled_root = compile_module(AccountAddress::ZERO, root_module, &compiled_callees)
        .unwrap()
        .0
        .into_inner();
    Pad::pad(table_size, &mut compiled_root, options);
    (compiled_root.freeze().unwrap(), compiled_callees)
}

pub fn generate_verified_modules(
    rng: &mut StdRng,
    number: usize,
    options: ModuleGeneratorOptions,
) -> (CompiledModule, Vec<CompiledModule>) {
    let (root, callees) = generate_modules(rng, number, options);
    for callee in &callees {
        verify_module(callee).unwrap()
    }
    verify_module(&root).unwrap();
    (root, callees)
}

///////////////////////////////////////////////////////////////////////////
// Generation of IR-level modules
///////////////////////////////////////////////////////////////////////////

pub struct ModuleGenerator<'a> {
    options: ModuleGeneratorOptions,
    current_module: ModuleDefinition,
    gen: &'a mut StdRng,
}

impl<'a> ModuleGenerator<'a> {
    fn index(&mut self, bound: usize) -> usize {
        self.gen.gen_range(0, bound)
    }

    fn identifier(&mut self) -> String {
        let len = self.gen.gen_range(10, self.options.max_string_size);
        random_string(&mut self.gen, len)
    }

    fn base_type(&mut self, ty_param_context: &[(TypeVar, Kind)]) -> Type {
        // TODO: Don't generate nested resources for now. Once we allow functions to take resources
        // (and have type parameters of kind Resource or All) then we should revisit this here.
        let structs: Vec<_> = self
            .current_module
            .structs
            .iter()
            .filter(|s| !s.value.is_nominal_resource)
            .cloned()
            .collect();

        let mut end = 5;
        if !ty_param_context.is_empty() {
            end += 1;
        };
        if !structs.is_empty() {
            end += 1;
        };

        match self.index(end) {
            0 => Type::Address,
            1 => Type::U8,
            2 => Type::U64,
            3 => Type::U128,
            4 => Type::Bool,
            5 if !structs.is_empty() => {
                let index = self.index(structs.len());
                let struct_def = structs[index].value.clone();
                let ty_instants = {
                    let num_typ_params = struct_def.type_formals.len();
                    // NB: Relying on randomness for termination here
                    init!(num_typ_params, self.base_type(ty_param_context))
                };
                let struct_ident = {
                    let struct_name = struct_def.name;
                    let module_name = ModuleName::module_self();
                    QualifiedStructIdent::new(module_name, struct_name)
                };
                Type::Struct(struct_ident, ty_instants)
            }
            _ => {
                let index = self.index(ty_param_context.len());
                let ty_var = ty_param_context[index].clone().0.value;
                Type::TypeParameter(ty_var)
            }
        }
    }

    fn typ(&mut self, ty_param_context: &[(TypeVar, Kind)]) -> Type {
        let typ = self.base_type(ty_param_context);
        // TODO: Always change the base type to a reference if it's resource type. Then we can
        // allow functions to take resources.
        // if typ.is_nominal_resource { .... }
        if self.options.references_allowed && self.gen.gen_bool(0.25) {
            let is_mutable = self.gen.gen_bool(0.25);
            Type::Reference(is_mutable, Box::new(typ))
        } else {
            typ
        }
    }

    fn type_parameters(&mut self) -> Vec<(TypeVar, Kind)> {
        // Don't generate type parameters if we're generating simple types only
        if self.options.simple_types_only {
            vec![]
        } else {
            let num_ty_params = self.index(self.options.max_ty_params);
            init!(
                num_ty_params,
                (
                    Spanned::unsafe_no_loc(TypeVar_::new(self.identifier())),
                    Kind::Copyable,
                )
            )
        }
    }

    // All functions will have unit return type, and an empty body with the exception of a return.
    // We'll scoop this out and replace it later on in the compiled module that we generate.
    fn function_signature(&mut self) -> FunctionSignature {
        let ty_params = self.type_parameters();
        let number_of_args = self.index(self.options.max_function_call_size);
        let mut formals: Vec<(Var, Type)> = init!(number_of_args, {
            let param_name = Spanned::unsafe_no_loc(Var_::new(self.identifier()));
            let ty = self.typ(&ty_params);
            (param_name, ty)
        });

        if self.options.args_for_ty_params {
            let mut ty_formals = ty_params
                .iter()
                .map(|(ty_var_, _)| {
                    let param_name = Spanned::unsafe_no_loc(Var_::new(self.identifier()));
                    let ty = Type::TypeParameter(ty_var_.value.clone());
                    (param_name, ty)
                })
                .collect();

            formals.append(&mut ty_formals);
        }

        FunctionSignature::new(formals, vec![], ty_params)
    }

    fn struct_fields(&mut self, ty_params: &[(TypeVar, Kind)]) -> StructDefinitionFields {
        let num_fields = self
            .gen
            .gen_range(self.options.min_fields, self.options.max_fields);
        let fields: Fields<Type> = init!(num_fields, {
            (
                Spanned::unsafe_no_loc(Field_::new(self.identifier())),
                self.base_type(ty_params),
            )
        });

        StructDefinitionFields::Move { fields }
    }

    fn function_def(&mut self) {
        let signature = self.function_signature();
        let num_locals = self.index(self.options.max_locals);
        let locals = init!(num_locals, {
            (
                Spanned::unsafe_no_loc(Var_::new(self.identifier())),
                self.typ(&signature.type_formals),
            )
        });
        let fun = Function_ {
            visibility: FunctionVisibility::Public,
            acquires: Vec::new(),
            specifications: Vec::new(),
            signature,
            body: FunctionBody::Move {
                locals,
                code: Block_ {
                    stmts: VecDeque::from(vec![Statement::CommandStatement(
                        Spanned::unsafe_no_loc(Cmd_::return_empty()),
                    )]),
                },
            },
        };
        let fun_name = FunctionName::new(self.identifier());
        self.current_module
            .functions
            .push((fun_name, Spanned::unsafe_no_loc(fun)));
    }

    fn struct_def(&mut self, is_nominal_resource: bool) {
        let name = StructName::new(self.identifier());
        let type_parameters = self.type_parameters();
        let fields = self.struct_fields(&type_parameters);
        let strct = StructDefinition_ {
            is_nominal_resource,
            name,
            type_formals: type_parameters,
            fields,
            invariants: vec![],
        };
        self.current_module
            .structs
            .push(Spanned::unsafe_no_loc(strct))
    }

    fn imports(callees: &Set<String>) -> Vec<ImportDefinition> {
        callees
            .iter()
            .map(|ident| {
                let module_name = ModuleName::new(ident.clone());
                let qualified_mod_ident =
                    QualifiedModuleIdent::new(module_name, AccountAddress::ZERO);
                let module_ident = ModuleIdent::Qualified(qualified_mod_ident);
                ImportDefinition::new(module_ident, None)
            })
            .collect()
    }

    fn gen(mut self) -> ModuleDefinition {
        let num_structs = self.index(self.options.max_structs) + 1;
        let num_functions = self.index(self.options.max_functions) + 1;
        // TODO: the order of generation here means that functions can't take resources as arguments.
        // We will need to generate (valid) bytecode bodies for these functions before we allow
        // resources.
        {
            // We generate a function at this point as an "entry point" into the module: since we
            // haven't generated any structs yet, this function will only take base types as its input
            // parameters. Likewise we can't take references since there isn't any value stack.
            let simple_types = self.options.simple_types_only;
            self.options.simple_types_only = true;
            self.function_def();
            self.options.simple_types_only = simple_types;
        }
        (0..num_structs).for_each(|_| self.struct_def(false));
        // TODO/XXX: We can allow references to resources here
        (0..num_functions).for_each(|_| self.function_def());
        if self.options.add_resources {
            (0..num_structs).for_each(|_| self.struct_def(true));
        }
        self.current_module
    }

    pub fn create(
        gen: &'a mut StdRng,
        options: ModuleGeneratorOptions,
        callable_modules: &Set<String>,
    ) -> ModuleDefinition {
        // TODO: Generation of struct and function handles to the `callable_modules`
        let module_name = {
            let len = gen.gen_range(10, options.max_string_size);
            random_string(gen, len)
        };
        let current_module = ModuleDefinition {
            name: ModuleName::new(module_name),
            imports: Self::imports(callable_modules),
            explicit_dependency_declarations: Vec::new(),
            structs: Vec::new(),
            functions: Vec::new(),
            constants: Vec::new(),
            synthetics: Vec::new(),
        };
        Self {
            options,
            gen,
            current_module,
        }
        .gen()
    }
}
