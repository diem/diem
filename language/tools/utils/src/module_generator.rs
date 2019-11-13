// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::common::*;
use ir_to_bytecode::compiler::compile_module;
use ir_to_bytecode_syntax::ast::*;
use libra_types::{account_address::AccountAddress, identifier::Identifier};
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use vm::file_format::CompiledModule;

pub type Map<K, V> = BTreeMap<K, V>;
pub type Set<K> = BTreeSet<K>;

macro_rules! init {
    ($len:expr, $e:expr) => {
        (0..$len).map(|_| $e).collect()
    };
}

/// Generate a `number - 1` modules. Then generate a root module that imports all of these modules.
pub fn generate_modules(number: usize) -> (CompiledModule, Vec<CompiledModule>) {
    assert!(number > 0, "We cannot generate zero modules");

    let (callee_names, callees): (Set<Identifier>, Vec<ModuleDefinition>) = (0..(number - 1))
        .map(|_| {
            let module = ModuleGenerator::create(&Set::new());
            let module_name = module.name.as_inner().to_string();
            (Identifier::new(module_name).unwrap(), module)
        })
        .unzip();

    let root_module = ModuleGenerator::create(&callee_names);
    let empty_deps: Vec<CompiledModule> = Vec::new();
    let compiled_callees = callees
        .into_iter()
        .map(|module| {
            compile_module(AccountAddress::default(), module, &empty_deps)
                .unwrap()
                .0
        })
        .collect();

    let (compiled_root, _) =
        compile_module(AccountAddress::default(), root_module, &compiled_callees).unwrap();
    (compiled_root, compiled_callees)
}

pub struct ModuleGenerator {
    current_module: ModuleDefinition,
    gen: StdRng,
}

impl ModuleGenerator {
    fn index(&mut self, bound: usize) -> usize {
        self.gen.gen_range(0, bound)
    }

    fn identifier(gen: &mut StdRng) -> Identifier {
        let len = gen.gen_range(10, MAX_STRING_SIZE);
        Identifier::new(random_string(gen, len)).unwrap()
    }

    fn base_type(&mut self, ty_param_context: &[(TypeVar_, Kind)]) -> Type {
        let mut end = 4;
        if !ty_param_context.is_empty() {
            end += 1;
        };

        if !self.current_module.structs.is_empty() {
            end += 1;
        };

        match self.index(end) {
            0 => Type::Address,
            1 => Type::U64,
            2 => Type::Bool,
            3 => Type::ByteArray,
            4 if !self.current_module.structs.is_empty() => {
                let index = self.index(self.current_module.structs.len());
                let struct_def = self.current_module.structs[index].value.clone();
                let ty_instants = {
                    let num_typ_params = struct_def.type_formals.len();
                    init!(num_typ_params, self.base_type(ty_param_context))
                };
                let struct_ident = {
                    let struct_name = struct_def.name.clone();
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
        if self.gen.gen_bool(0.25) {
            let is_mutable = self.gen.gen_bool(0.25);
            Type::Reference(is_mutable, Box::new(typ))
        } else {
            typ
        }
    }

    fn type_formals(&mut self) -> Vec<(TypeVar_, Kind)> {
        let num_ty_params = self.index(MAX_NUM_TY_PARAMS);
        init!(
            num_ty_params,
            (
                Spanned::no_loc(TypeVar::new(Self::identifier(&mut self.gen))),
                Kind::Unrestricted,
            )
        )
    }

    // All functions will have unit return type, and an empty body with the exception of a return.
    // We'll scoop this out and replace it later on in the compiled module that we generate.
    fn function_signature(&mut self) -> FunctionSignature {
        let ty_params = self.type_formals();
        let number_of_args = self.index(MAX_FUNCTION_CALL_SIZE);
        let formals = init!(number_of_args, {
            let param_name = Var::new_(Self::identifier(&mut self.gen));
            let ty = self.typ(&ty_params);
            (param_name, ty)
        });

        FunctionSignature::new(formals, vec![], ty_params)
    }

    fn struct_fields(&mut self, ty_params: &[(TypeVar_, Kind)]) -> StructDefinitionFields {
        let num_fields = self.index(MAX_FIELDS);
        let fields: Fields<Type> = init!(num_fields, {
            (
                Spanned::no_loc(Field::new(Self::identifier(&mut self.gen))),
                self.base_type(ty_params),
            )
        });

        StructDefinitionFields::Move { fields }
    }

    fn function_def(&mut self) {
        let signature = self.function_signature();
        let num_locals = self.index(MAX_NUM_LOCALS);
        let locals = init!(num_locals, {
            (
                Var::new_(Self::identifier(&mut self.gen)),
                self.typ(&signature.type_formals),
            )
        });
        let fun = Function {
            visibility: FunctionVisibility::Public,
            acquires: Vec::new(),
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
        let fun_name = FunctionName::new(Self::identifier(&mut self.gen));
        self.current_module
            .functions
            .push((fun_name, Spanned::no_loc(fun)));
    }

    fn struct_def(&mut self) {
        let name = StructName::new(Self::identifier(&mut self.gen));
        let type_formals = self.type_formals();
        let fields = self.struct_fields(&type_formals);
        let strct = StructDefinition {
            // TODO: Generate resources
            is_nominal_resource: false, // self.gen.gen_bool(0.25),
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
        let num_structs = self.index(MAX_STRUCTS);
        let num_functions = self.index(MAX_FUNCTIONS);
        (1..num_structs).for_each(|_| self.struct_def());
        (1..num_functions).for_each(|_| self.function_def());
        self.current_module
    }

    pub fn create(callable_modules: &Set<Identifier>) -> ModuleDefinition {
        let seed: [u8; 32] = [0; 32];
        let mut gen = StdRng::from_seed(seed);
        let module_name = Self::identifier(&mut gen);
        let current_module = ModuleDefinition {
            name: ModuleName::new(module_name),
            imports: Self::imports(callable_modules),
            structs: Vec::new(),
            functions: Vec::new(),
        };
        Self {
            gen,
            current_module,
        }
        .gen()
    }
}
