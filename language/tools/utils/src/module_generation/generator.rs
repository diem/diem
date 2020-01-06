// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::module_generation::{
    options::ModuleGeneratorOptions, padding::Pad, utils::random_string,
};
use bytecode_verifier::VerifiedModule;
use ir_to_bytecode::compiler::compile_module;
use ir_to_bytecode_syntax::ast::*;
use libra_types::{account_address::AccountAddress, identifier::Identifier};
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::collections::{BTreeSet, VecDeque};
use vm::file_format::CompiledModule;

type Set<K> = BTreeSet<K>;

macro_rules! init {
    ($len:expr, $e:expr) => {
        (0..$len).map(|_| $e).collect()
    };
}

pub fn generate_module(options: ModuleGeneratorOptions) -> CompiledModule {
    generate_modules(1, options).0
}

/// Generate a `number - 1` modules. Then generate a root module that imports all of these modules.
pub fn generate_modules(
    number: usize,
    options: ModuleGeneratorOptions,
) -> (CompiledModule, Vec<CompiledModule>) {
    assert!(number > 0, "We cannot generate zero modules");

    let table_size = options.min_table_size;
    let (callee_names, callees): (Set<Identifier>, Vec<ModuleDefinition>) = (0..(number - 1))
        .map(|_| {
            let module = ModuleGenerator::create(options.clone(), &Set::new());
            let module_name = module.name.as_inner().to_string();
            (Identifier::new(module_name).unwrap(), module)
        })
        .unzip();

    let root_module = ModuleGenerator::create(options.clone(), &callee_names);
    let empty_deps: Vec<CompiledModule> = Vec::new();
    let compiled_callees = callees
        .into_iter()
        .map(|module| {
            let mut module = compile_module(AccountAddress::default(), module, &empty_deps)
                .unwrap()
                .0
                .into_inner();
            Pad::pad(table_size, &mut module, options.clone());
            module.freeze().unwrap()
        })
        .collect();

    let mut compiled_root =
        compile_module(AccountAddress::default(), root_module, &compiled_callees)
            .unwrap()
            .0
            .into_inner();
    Pad::pad(table_size, &mut compiled_root, options);
    (compiled_root.freeze().unwrap(), compiled_callees)
}

pub fn generate_verified_modules(
    number: usize,
    options: ModuleGeneratorOptions,
) -> (VerifiedModule, Vec<VerifiedModule>) {
    let (root, callees) = generate_modules(number, options);
    let verified_modules = callees
        .into_iter()
        .map(|m| VerifiedModule::new(m).unwrap())
        .collect();
    let verified_root = VerifiedModule::new(root).unwrap();
    (verified_root, verified_modules)
}

///////////////////////////////////////////////////////////////////////////
// Generation of IR-level modules
///////////////////////////////////////////////////////////////////////////

pub struct ModuleGenerator {
    options: ModuleGeneratorOptions,
    current_module: ModuleDefinition,
    gen: StdRng,
}

impl ModuleGenerator {
    fn index(&mut self, bound: usize) -> usize {
        self.gen.gen_range(0, bound)
    }

    fn identifier(&mut self) -> Identifier {
        let len = self.gen.gen_range(10, self.options.max_string_size);
        Identifier::new(random_string(&mut self.gen, len)).unwrap()
    }

    fn base_type(&mut self, ty_param_context: &[(TypeVar_, Kind)]) -> Type {
        // TODO: Don't generate nested resources for now. Once we allow functions to take resources
        // (and have type parameters of kind Resource or All) then we should revisit this here.
        let structs: Vec<_> = self
            .current_module
            .structs
            .iter()
            .filter(|s| !s.value.is_nominal_resource)
            .cloned()
            .collect();

        let mut end = 4;
        if !ty_param_context.is_empty() {
            end += 1;
        };
        if !structs.is_empty() {
            end += 1;
        };

        match self.index(end) {
            0 => Type::Address,
            1 => Type::U64,
            2 => Type::Bool,
            3 => Type::ByteArray,
            4 if !structs.is_empty() => {
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

    fn typ(&mut self, ty_param_context: &[(TypeVar_, Kind)]) -> Type {
        let typ = self.base_type(ty_param_context);
        // TODO: Always change the base type to a reference if it's resource type. Then we can
        // allow functions to take resources.
        // if typ.is_nominal_resource { .... }
        if !self.options.simple_types_only && self.gen.gen_bool(0.25) {
            let is_mutable = self.gen.gen_bool(0.25);
            Type::Reference(is_mutable, Box::new(typ))
        } else {
            typ
        }
    }

    fn type_formals(&mut self) -> Vec<(TypeVar_, Kind)> {
        // Don't generate type parameters if we're generating simple types only
        if self.options.simple_types_only {
            vec![]
        } else {
            let num_ty_params = self.index(self.options.max_ty_params);
            init!(
                num_ty_params,
                (
                    Spanned::no_loc(TypeVar::new(self.identifier())),
                    Kind::Unrestricted,
                )
            )
        }
    }

    // All functions will have unit return type, and an empty body with the exception of a return.
    // We'll scoop this out and replace it later on in the compiled module that we generate.
    fn function_signature(&mut self) -> FunctionSignature {
        let ty_params = self.type_formals();
        let number_of_args = self.index(self.options.max_function_call_size);
        let formals = init!(number_of_args, {
            let param_name = Var::new_(self.identifier());
            let ty = self.typ(&ty_params);
            (param_name, ty)
        });

        FunctionSignature::new(formals, vec![], ty_params)
    }

    fn struct_fields(&mut self, ty_params: &[(TypeVar_, Kind)]) -> StructDefinitionFields {
        let num_fields = self
            .gen
            .gen_range(self.options.min_fields, self.options.max_fields);
        let fields: Fields<Type> = init!(num_fields, {
            (
                Spanned::no_loc(Field::new(self.identifier())),
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
                Var::new_(self.identifier()),
                self.typ(&signature.type_formals),
            )
        });
        let fun = Function {
            visibility: FunctionVisibility::Public,
            acquires: Vec::new(),
            specifications: Vec::new(),
            signature,
            body: FunctionBody::Move {
                locals,
                code: Block {
                    stmts: VecDeque::from(vec![Statement::CommandStatement(Spanned::no_loc(
                        Cmd::return_empty(),
                    ))]),
                },
            },
        };
        let fun_name = FunctionName::new(self.identifier());
        self.current_module
            .functions
            .push((fun_name, Spanned::no_loc(fun)));
    }

    fn struct_def(&mut self, is_nominal_resource: bool) {
        let name = StructName::new(self.identifier());
        let type_formals = self.type_formals();
        let fields = self.struct_fields(&type_formals);
        let strct = StructDefinition {
            is_nominal_resource,
            name,
            type_formals,
            fields,
        };
        self.current_module.structs.push(Spanned::no_loc(strct))
    }

    fn imports(callees: &Set<Identifier>) -> Vec<ImportDefinition> {
        callees
            .iter()
            .map(|ident| {
                let module_name = ModuleName::new(ident.clone());
                let qualified_mod_ident =
                    QualifiedModuleIdent::new(module_name, AccountAddress::default());
                let module_ident = ModuleIdent::Qualified(qualified_mod_ident);
                ImportDefinition::new(module_ident, None)
            })
            .collect()
    }

    fn gen(mut self) -> ModuleDefinition {
        let num_structs = self.index(self.options.max_structs);
        let num_functions = self.index(self.options.max_functions);
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
        (1..num_structs).for_each(|_| self.struct_def(false));
        // TODO/XXX: We can allow references to resources here
        (1..num_functions).for_each(|_| self.function_def());
        if self.options.add_resources {
            (1..num_structs).for_each(|_| self.struct_def(true));
        }
        self.current_module
    }

    pub fn create(
        options: ModuleGeneratorOptions,
        callable_modules: &Set<Identifier>,
    ) -> ModuleDefinition {
        // TODO: Generation of struct and function handles to the `callable_modules`
        let seed: [u8; 32] = [0; 32];
        let mut gen = StdRng::from_seed(seed);
        let module_name = {
            let len = gen.gen_range(10, options.max_string_size);
            Identifier::new(random_string(&mut gen, len)).unwrap()
        };
        let current_module = ModuleDefinition {
            name: ModuleName::new(module_name),
            imports: Self::imports(callable_modules),
            structs: Vec::new(),
            functions: Vec::new(),
        };
        Self {
            options,
            gen,
            current_module,
        }
        .gen()
    }
}
