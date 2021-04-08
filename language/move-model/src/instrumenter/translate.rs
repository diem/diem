// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use num::ToPrimitive;
use std::collections::BTreeMap;

use move_ir_types::{
    location::{sp, Loc as MoveLoc},
    sp,
};
use move_lang::{
    compiled_unit::{CompiledUnit, FunctionInfo},
    expansion::ast::{AbilitySet, Value_ as MoveValue},
    hlir::ast::{
        BaseType_, Block, Command, Command_, Exp as MoveExp, ExpListItem, Function, FunctionBody,
        FunctionBody_, ModuleCall, ModuleDefinition, Program, Script, SingleType, SingleType_,
        Statement, Statement_, TypeName_, Type_ as MoveType, UnannotatedExp_,
    },
    naming::ast::{BuiltinTypeName_, TParam, TParamID},
    parser::ast::{Ability_, BinOp_, FunctionName, ModuleIdent, StructName, UnaryOp_, Var},
    shared::{unique_map::UniqueMap, Address, Identifier},
};

use crate::{
    ast::{ConditionKind, Exp as SpecExp, Operation, Spec, TempIndex, Value as SpecValue},
    instrumenter::context::CompilationContext,
    model::{FunctionEnv, GlobalEnv, ModuleEnv, NodeId},
    ty::{PrimitiveType as SpecPrimitiveType, Type as SpecType},
};

// =================================================================================================
// Entry point
pub fn run(env: &GlobalEnv, units: &[CompiledUnit], ast: Program) -> Program {
    let Program {
        modules: emodules,
        scripts: escripts,
    } = ast;
    let modules = modules(env, units, emodules);
    let scripts = scripts(env, units, escripts);
    Program { modules, scripts }
}

//
// Module
//

fn modules(
    env: &GlobalEnv,
    units: &[CompiledUnit],
    modules: UniqueMap<ModuleIdent, ModuleDefinition>,
) -> UniqueMap<ModuleIdent, ModuleDefinition> {
    modules.map(|ident, mdef| module(env, units, ident, mdef))
}

fn module(
    env: &GlobalEnv,
    units: &[CompiledUnit],
    ident: ModuleIdent,
    mdef: ModuleDefinition,
) -> ModuleDefinition {
    // get the module env
    let module_data = env.module_data.iter().find(|m| {
        let module_id = m.module.self_id();
        module_id.address().as_ref() == ident.value.0.as_ref()
            && module_id.name().as_str() == ident.value.1
    });
    if module_data.is_none() {
        env.error(
            &env.to_loc(&ident.loc()),
            "Unable to find the ModuleData associated with this module id",
        );
        return mdef;
    }
    let menv = env.get_module(module_data.unwrap().id);

    // get the information hidden behind the compilation unit
    let ctxt_opt = units.iter().find_map(|item| match item {
        CompiledUnit::Module {
            ident: item_ident,
            source_map,
            function_infos,
            ..
        } => {
            if item_ident == &ident {
                Some(CompilationContext {
                    source_map: source_map.clone(),
                    function_infos: function_infos.clone(),
                })
            } else {
                None
            }
        }
        CompiledUnit::Script { .. } => None,
    });
    if ctxt_opt.is_none() {
        env.error(
            &env.to_loc(&ident.loc()),
            "Unable to find the CompilationUnit associated with this module id",
        );
        return mdef;
    }
    let ctxt = ctxt_opt.unwrap();

    // do the actual conversion
    module_(&menv, &ctxt, mdef)
}

fn module_(
    menv: &ModuleEnv<'_>,
    ctxt: &CompilationContext,
    mdef: ModuleDefinition,
) -> ModuleDefinition {
    // construct the new module field by field
    let ModuleDefinition {
        is_source_module,
        dependency_order,
        friends,
        structs,
        functions: efunctions,
        constants,
    } = mdef;
    let functions = efunctions.map(|name, func| function(&menv, ctxt, name, func));

    // re-pack the module definition
    ModuleDefinition {
        is_source_module,
        dependency_order,
        friends,
        structs,
        functions,
        constants,
    }
}

//
// Script
//

fn scripts(
    env: &GlobalEnv,
    units: &[CompiledUnit],
    scripts: BTreeMap<String, Script>,
) -> BTreeMap<String, Script> {
    scripts
        .into_iter()
        .map(|(n, s)| (n, script(env, units, s)))
        .collect()
}

fn script(env: &GlobalEnv, units: &[CompiledUnit], sdef: Script) -> Script {
    // find the env that represent the script
    let default_addr = Address::default();
    let module_data = env.module_data.iter().find(|m| {
        let module_id = m.module.self_id();
        module_id.address().as_ref() == default_addr.as_ref()
            && module_id.name().as_str() == sdef.function_name.0.value
    });
    if module_data.is_none() {
        env.error(
            &env.to_loc(&sdef.loc),
            "Unable to find the ModuleData associated with this script",
        );
        return sdef;
    }
    let menv = env.get_module(module_data.unwrap().id);

    // get the information hidden behind the compilation unit
    let ctxt_opt = units.iter().find_map(|item| match item {
        CompiledUnit::Module { .. } => None,
        CompiledUnit::Script {
            loc,
            source_map,
            function_info,
            ..
        } => {
            if loc == &sdef.loc {
                let mut function_infos = UniqueMap::new();
                function_infos
                    .add(sdef.function_name.clone(), function_info.clone())
                    .unwrap();
                Some(CompilationContext {
                    source_map: source_map.clone(),
                    function_infos,
                })
            } else {
                None
            }
        }
    });
    if ctxt_opt.is_none() {
        env.error(
            &env.to_loc(&sdef.loc),
            "Unable to find the CompilationUnit associated with this script",
        );
        return sdef;
    }
    let ctxt = ctxt_opt.unwrap();

    // do the actual conversion
    script_(&menv, &ctxt, sdef)
}

fn script_(menv: &ModuleEnv<'_>, ctxt: &CompilationContext, sdef: Script) -> Script {
    // construct the new script field by field
    let Script {
        loc,
        constants,
        function_name,
        function: efunction,
    } = sdef;
    let function = function(menv, ctxt, function_name.clone(), efunction);

    // re-pack the script definition
    Script {
        loc,
        constants,
        function_name,
        function,
    }
}

//
// Function
//

fn function(
    menv: &ModuleEnv<'_>,
    ctxt: &CompilationContext,
    name: FunctionName,
    func: Function,
) -> Function {
    let env = menv.env;

    // get the function env
    let symbol = env.symbol_pool().make(name.value());
    let fenv_opt = menv.find_function(symbol);
    if fenv_opt.is_none() {
        env.error(
            &env.to_loc(&name.loc()),
            "Unable to find FunctionEnv with this function name",
        );
        return func;
    }
    let fenv = fenv_opt.unwrap();

    // get the information hidden behind the compilation unit
    let info_opt = ctxt.function_infos.get(&name);
    if info_opt.is_none() {
        env.error(
            &env.to_loc(&name.loc()),
            "Unable to find the CompilationUnit with this function name",
        );
        return func;
    }
    let info = info_opt.unwrap();

    // do the actual conversion
    function_(&fenv, info, func)
}

fn function_(fenv: &FunctionEnv<'_>, info: &FunctionInfo, func: Function) -> Function {
    // construct the new function field by field
    let Function {
        visibility,
        signature,
        acquires,
        body: ebody,
    } = func;
    let body = function_body(&fenv, info, ebody);

    // re-pack the function definition
    Function {
        visibility,
        signature,
        acquires,
        body,
    }
}

fn function_body(
    fenv: &FunctionEnv<'_>,
    info: &FunctionInfo,
    sp!(loc, body): FunctionBody,
) -> FunctionBody {
    match body {
        FunctionBody_::Native => sp(loc, FunctionBody_::Native),
        FunctionBody_::Defined { mut locals, body } => {
            let new_body = statement_block(fenv, info, &mut locals, body);
            sp(
                loc,
                FunctionBody_::Defined {
                    locals,
                    body: new_body,
                },
            )
        }
    }
}

//
// Statement
//

fn statement_block(
    fenv: &FunctionEnv<'_>,
    info: &FunctionInfo,
    locals: &mut UniqueMap<Var, SingleType>,
    block: Block,
) -> Block {
    let mut instrumented = Block::new();
    for stmt in block {
        statement(fenv, info, stmt, locals, &mut instrumented);
    }
    instrumented
}

fn statement(
    fenv: &FunctionEnv<'_>,
    info: &FunctionInfo,
    sp!(loc, stmt): Statement,
    locals: &mut UniqueMap<Var, SingleType>,
    block: &mut Block,
) {
    let new_stmt = match stmt {
        Statement_::Command(cmd) => {
            let new_cmd = command(fenv, info, cmd, locals, block);
            Statement_::Command(new_cmd)
        }
        Statement_::IfElse {
            cond,
            if_block,
            else_block,
        } => {
            let new_cond = expression(fenv, info, *cond, locals, block);
            let new_if_block = statement_block(fenv, info, locals, if_block);
            let new_else_block = statement_block(fenv, info, locals, else_block);
            Statement_::IfElse {
                cond: Box::new(new_cond),
                if_block: new_if_block,
                else_block: new_else_block,
            }
        }
        Statement_::While {
            cond: (cond_block, cond_exp),
            block: loop_block,
        } => {
            let new_cond_exp = expression(fenv, info, *cond_exp, locals, block);
            let new_cond_block = statement_block(fenv, info, locals, cond_block);
            let new_loop_block = statement_block(fenv, info, locals, loop_block);
            Statement_::While {
                cond: (new_cond_block, Box::new(new_cond_exp)),
                block: new_loop_block,
            }
        }
        Statement_::Loop {
            block: loop_block,
            has_break,
        } => {
            let new_block = statement_block(fenv, info, locals, loop_block);
            Statement_::Loop {
                block: new_block,
                has_break,
            }
        }
    };
    block.push_back(sp(loc, new_stmt));
}

fn command(
    fenv: &FunctionEnv<'_>,
    info: &FunctionInfo,
    sp!(loc, cmd): Command,
    locals: &mut UniqueMap<Var, SingleType>,
    block: &mut Block,
) -> Command {
    let new_cmd = match cmd {
        Command_::Assign(lhs, rhs) => {
            let new_rhs = expression(fenv, info, *rhs, locals, block);
            Command_::Assign(lhs, Box::new(new_rhs))
        }
        Command_::Mutate(eref, eval) => {
            let new_eval = expression(fenv, info, *eval, locals, block);
            let new_eref = expression(fenv, info, *eref, locals, block);
            Command_::Mutate(Box::new(new_eref), Box::new(new_eval))
        }
        Command_::Abort(exp) => {
            let new_exp = expression(fenv, info, exp, locals, block);
            Command_::Abort(new_exp)
        }
        Command_::Return { from_user, exp } => {
            let new_exp = expression(fenv, info, exp, locals, block);
            Command_::Return {
                from_user,
                exp: new_exp,
            }
        }
        Command_::Break => Command_::Break,
        Command_::Continue => Command_::Continue,
        Command_::IgnoreAndPop { pop_num, exp } => {
            let new_exp = expression(fenv, info, exp, locals, block);
            Command_::IgnoreAndPop {
                pop_num,
                exp: new_exp,
            }
        }
        Command_::Jump { from_user, target } => Command_::Jump { from_user, target },
        Command_::JumpIf {
            cond,
            if_true,
            if_false,
        } => {
            let new_cond = expression(fenv, info, cond, locals, block);
            Command_::JumpIf {
                cond: new_cond,
                if_true,
                if_false,
            }
        }
    };
    sp(loc, new_cmd)
}

fn expression(
    fenv: &FunctionEnv<'_>,
    info: &FunctionInfo,
    exp: MoveExp,
    locals: &mut UniqueMap<Var, SingleType>,
    block: &mut Block,
) -> MoveExp {
    let env = fenv.module_env.env;
    let spec = fenv.get_spec();
    let MoveExp {
        ty,
        exp: sp!(loc, exp_),
    } = exp;
    let new_exp = match exp_ {
        // point of instrumentation
        UnannotatedExp_::Spec(spec_id, vars) => {
            match info.spec_info.get(&spec_id) {
                None => env.error(
                    &env.to_loc(&loc),
                    "Unable to find the CodeOffset in FunctionInfo for this spec block",
                ),
                Some(spec_info) => match spec.on_impl.get(&spec_info.offset) {
                    None => env.error(
                        &env.to_loc(&loc),
                        "Unable to find the Spec in FunctionEnv for this spec block",
                    ),
                    Some(inline_spec) => {
                        // TODO (mengxu): remove these checking once we know that
                        // `spec_info.used_locals` and `vars` are in fact the same thing
                        assert_eq!(spec_info.used_locals.len(), vars.len());
                        for v in vars.keys() {
                            assert!(spec_info.used_locals.contains_key(v));
                        }
                        // build the TempIndex to Var mapping
                        let vidx: BTreeMap<TempIndex, Var> = spec_info
                            .used_locals
                            .key_cloned_iter()
                            .map(|(v, v_info)| {
                                assert_eq!(locals.get(&v), Some(&v_info.type_));
                                (v_info.index as TempIndex, v)
                            })
                            .collect();
                        assert_eq!(vidx.len(), vars.len());
                        instrument_inline_spec(fenv, &vidx, loc, inline_spec, locals, block);
                    }
                },
            }
            UnannotatedExp_::Spec(spec_id, vars)
        }
        // propagate other cases
        exp_ @ UnannotatedExp_::Unit { .. } => exp_,
        exp_ @ UnannotatedExp_::Value(..) => exp_,
        exp_ @ UnannotatedExp_::Move { .. } => exp_,
        exp_ @ UnannotatedExp_::Copy { .. } => exp_,
        exp_ @ UnannotatedExp_::Constant(..) => exp_,
        UnannotatedExp_::ModuleCall(call) => {
            let ModuleCall {
                module,
                name,
                type_arguments,
                arguments,
                acquires,
            } = *call;
            let new_arguments = expression(fenv, info, *arguments, locals, block);
            let new_call = ModuleCall {
                module,
                name,
                type_arguments,
                arguments: Box::new(new_arguments),
                acquires,
            };
            UnannotatedExp_::ModuleCall(Box::new(new_call))
        }
        UnannotatedExp_::Builtin(func, arguments) => {
            let new_arguments = expression(fenv, info, *arguments, locals, block);
            UnannotatedExp_::Builtin(func, Box::new(new_arguments))
        }
        UnannotatedExp_::Freeze(ref_exp) => {
            let new_ref_exp = expression(fenv, info, *ref_exp, locals, block);
            UnannotatedExp_::Freeze(Box::new(new_ref_exp))
        }
        UnannotatedExp_::Dereference(ref_exp) => {
            let new_ref_exp = expression(fenv, info, *ref_exp, locals, block);
            UnannotatedExp_::Dereference(Box::new(new_ref_exp))
        }
        UnannotatedExp_::Cast(sub_exp, ty) => {
            let new_sub_exp = expression(fenv, info, *sub_exp, locals, block);
            UnannotatedExp_::Cast(Box::new(new_sub_exp), ty)
        }
        UnannotatedExp_::UnaryExp(op, sub_exp) => {
            let new_sub_exp = expression(fenv, info, *sub_exp, locals, block);
            UnannotatedExp_::UnaryExp(op, Box::new(new_sub_exp))
        }
        UnannotatedExp_::BinopExp(lhs, op, rhs) => {
            let new_lhs = expression(fenv, info, *lhs, locals, block);
            let new_rhs = expression(fenv, info, *rhs, locals, block);
            UnannotatedExp_::BinopExp(Box::new(new_lhs), op, Box::new(new_rhs))
        }
        UnannotatedExp_::Pack(struct_name, base_type, fields) => {
            let new_fields = fields
                .into_iter()
                .map(|(field, ty, field_exp)| {
                    (field, ty, expression(fenv, info, field_exp, locals, block))
                })
                .collect();
            UnannotatedExp_::Pack(struct_name, base_type, new_fields)
        }
        UnannotatedExp_::ExpList(items) => {
            let new_items = items
                .into_iter()
                .map(|item| match item {
                    ExpListItem::Single(sub_exp, ty) => {
                        ExpListItem::Single(expression(fenv, info, sub_exp, locals, block), ty)
                    }
                    ExpListItem::Splat(sub_loc, sub_exp, tys) => ExpListItem::Splat(
                        sub_loc,
                        expression(fenv, info, sub_exp, locals, block),
                        tys,
                    ),
                })
                .collect();
            UnannotatedExp_::ExpList(new_items)
        }
        UnannotatedExp_::Borrow(is_mut, sub_exp, field) => {
            let new_sub_exp = expression(fenv, info, *sub_exp, locals, block);
            UnannotatedExp_::Borrow(is_mut, Box::new(new_sub_exp), field)
        }
        exp_ @ UnannotatedExp_::BorrowLocal(..) => exp_,
        exp_ @ UnannotatedExp_::Unreachable => exp_,
        // a valid HLIR ast should not have unresolved expressions
        UnannotatedExp_::UnresolvedError => unreachable!(),
    };
    MoveExp {
        ty,
        exp: sp(loc, new_exp),
    }
}

fn instrument_inline_spec(
    fenv: &FunctionEnv<'_>,
    vars: &BTreeMap<TempIndex, Var>,
    loc: MoveLoc,
    spec: &Spec,
    locals: &mut UniqueMap<Var, SingleType>,
    block: &mut Block,
) {
    // an inline spec should have no `on_impl` specs
    assert!(spec.on_impl.is_empty());

    // iterate and convert the conditions
    for condition in &spec.conditions {
        // only `assert` and `assume` are allowed for in-code spec blocks
        assert!(matches!(
            &condition.kind,
            ConditionKind::Assert | ConditionKind::Assume
        ));
        // `assert` and `assume` expressions do not have other info attached
        assert!(condition.properties.is_empty());
        assert!(condition.additional_exps.is_empty());
        // now the actual instrumentation
        // TODO (mengxu): this just gets the condition expression, we still need
        // to construct the if-else statement
        convert_spec_expression(fenv, vars, loc, &condition.exp, locals, block);
    }
}

fn convert_spec_type(fenv: &FunctionEnv<'_>, loc: MoveLoc, ty: &SpecType) -> MoveType {
    let env = fenv.module_env.env;
    match ty {
        SpecType::Primitive(SpecPrimitiveType::Bool) => MoveType::Single(SingleType_::bool(loc)),
        SpecType::Primitive(SpecPrimitiveType::U8) => MoveType::Single(SingleType_::u8(loc)),
        SpecType::Primitive(SpecPrimitiveType::U64) => MoveType::Single(SingleType_::u64(loc)),
        SpecType::Primitive(SpecPrimitiveType::U128) => MoveType::Single(SingleType_::u128(loc)),
        SpecType::Primitive(SpecPrimitiveType::Address) => {
            MoveType::Single(SingleType_::address(loc))
        }
        SpecType::Primitive(SpecPrimitiveType::Signer) => MoveType::Single(sp(
            loc,
            SingleType_::Base(BaseType_::builtin(loc, BuiltinTypeName_::Signer, vec![])),
        )),
        SpecType::Primitive(_) => {
            env.error(&env.to_loc(&loc), "Unable to resolve a spec-specific type");
            unresolved_move_type(loc)
        }
        SpecType::Tuple(components) => {
            let converted_components = components
                .iter()
                .filter_map(|item| match convert_spec_type(fenv, loc, item) {
                    MoveType::Single(single_type) => Some(single_type),
                    MoveType::Multiple(..) | MoveType::Unit => {
                        env.error(&env.to_loc(&loc), "Only SingleType is allowed in a Tuple");
                        None
                    }
                })
                .collect();
            MoveType::Multiple(converted_components)
        }
        SpecType::Vector(sub_type) => {
            let converted_sub_type = match convert_spec_type(fenv, loc, sub_type) {
                MoveType::Single(single_type) => match single_type.value {
                    SingleType_::Base(base_type) => base_type,
                    SingleType_::Ref(..) => {
                        env.error(
                            &env.to_loc(&loc),
                            "Reference types cannot be a type parameter for the vector type",
                        );
                        sp(loc, BaseType_::UnresolvedError)
                    }
                },
                MoveType::Multiple(..) | MoveType::Unit => {
                    env.error(
                        &env.to_loc(&loc),
                        "Only BaseType can be a type parameter for the vector type",
                    );
                    sp(loc, BaseType_::UnresolvedError)
                }
            };
            MoveType::Single(sp(
                loc,
                SingleType_::Base(BaseType_::builtin(
                    loc,
                    BuiltinTypeName_::Vector,
                    vec![converted_sub_type],
                )),
            ))
        }
        SpecType::Struct(module_id, struct_id, sub_types) => {
            let module_env = env.get_module(*module_id);
            let mid = module_env.get_verified_module().self_id();
            let module_ident = ModuleIdent {
                locs: (loc, loc),
                value: (
                    Address::new(mid.address().clone().to_u8()),
                    mid.name().to_owned().into_string(),
                ),
            };
            let struct_env = module_env.get_struct(*struct_id);
            let struct_name = StructName(sp(loc, struct_env.get_identifier().into_string()));
            let abilities = struct_env.get_abilities();
            let mut ability_set = AbilitySet::empty();
            if abilities.has_copy() {
                ability_set.add(sp(loc, Ability_::Copy)).unwrap();
            }
            if abilities.has_drop() {
                ability_set.add(sp(loc, Ability_::Drop)).unwrap();
            }
            if abilities.has_store() {
                ability_set.add(sp(loc, Ability_::Store)).unwrap();
            }
            if abilities.has_key() {
                ability_set.add(sp(loc, Ability_::Key)).unwrap();
            }
            let converted_sub_types = sub_types
                .iter()
                .map(|sub_type| {
                    let converted = convert_spec_type(fenv, loc, sub_type);
                    match converted {
                        MoveType::Single(single_type) => match single_type.value {
                            SingleType_::Base(base_type) => base_type,
                            SingleType_::Ref(..) => {
                                env.error(
                                    &env.to_loc(&loc),
                                    "Reference types cannot be a type parameter for the struct type",
                                );
                                sp(loc, BaseType_::UnresolvedError)
                            }
                        },
                        MoveType::Multiple(..) | MoveType::Unit => {
                            env.error(
                                &env.to_loc(&loc),
                                "Only BaseType can be a type parameter for the struct type",
                            );
                            sp(loc, BaseType_::UnresolvedError)
                        }
                    }
                })
                .collect();
            MoveType::Single(sp(
                loc,
                SingleType_::Base(sp(
                    loc,
                    BaseType_::Apply(
                        ability_set,
                        sp(loc, TypeName_::ModuleType(module_ident, struct_name)),
                        converted_sub_types,
                    ),
                )),
            ))
        }
        SpecType::TypeParameter(idx) => {
            let type_params = fenv.get_type_parameters();
            let type_param = &type_params[*idx as usize];
            let param_name = sp(loc, env.symbol_pool().string(type_param.0).to_string());
            let abilities = &type_param.1 .0;
            let mut ability_set = AbilitySet::empty();
            if abilities.has_copy() {
                ability_set.add(sp(loc, Ability_::Copy)).unwrap();
            }
            if abilities.has_drop() {
                ability_set.add(sp(loc, Ability_::Drop)).unwrap();
            }
            if abilities.has_store() {
                ability_set.add(sp(loc, Ability_::Store)).unwrap();
            }
            if abilities.has_key() {
                ability_set.add(sp(loc, Ability_::Key)).unwrap();
            }
            let tparam = TParam {
                id: TParamID::next(),
                user_specified_name: param_name,
                abilities: ability_set,
            };
            MoveType::Single(sp(
                loc,
                SingleType_::Base(sp(loc, BaseType_::Param(tparam))),
            ))
        }
        SpecType::Reference(is_mut, sub_type) => {
            let converted_sub_type = match convert_spec_type(fenv, loc, sub_type) {
                MoveType::Single(single_type) => match single_type.value {
                    SingleType_::Base(base_type) => base_type,
                    SingleType_::Ref(..) => {
                        env.error(
                            &env.to_loc(&loc),
                            "Reference types cannot be the base type for the reference type",
                        );
                        sp(loc, BaseType_::UnresolvedError)
                    }
                },
                MoveType::Multiple(..) | MoveType::Unit => {
                    env.error(
                        &env.to_loc(&loc),
                        "Only BaseType can be a base type for the reference type",
                    );
                    sp(loc, BaseType_::UnresolvedError)
                }
            };
            MoveType::Single(sp(loc, SingleType_::Ref(*is_mut, converted_sub_type)))
        }
        SpecType::Fun(..)
        | SpecType::TypeDomain(..)
        | SpecType::ResourceDomain(..)
        | SpecType::TypeLocal(..) => {
            env.error(&env.to_loc(&loc), "Unable to resolve a spec-specific type");
            unresolved_move_type(loc)
        }
        SpecType::Var(..) | SpecType::Error => unreachable!(),
    }
}

fn convert_spec_expression(
    fenv: &FunctionEnv<'_>,
    vars: &BTreeMap<TempIndex, Var>,
    loc: MoveLoc,
    exp: &SpecExp,
    locals: &mut UniqueMap<Var, SingleType>,
    block: &mut Block,
) -> MoveExp {
    let env = fenv.module_env.env;
    match exp {
        SpecExp::Value(node_id, val) => convert_spec_value(env, loc, *node_id, val),
        SpecExp::Temporary(_, idx) => convert_spec_var_temporary(env, vars, locals, loc, *idx),
        SpecExp::Call(node_id, Operation::Not, args) => convert_spec_op_unary(
            fenv,
            vars,
            loc,
            *node_id,
            UnaryOp_::Not,
            args,
            locals,
            block,
        ),
        SpecExp::Call(node_id, Operation::Add, args) => {
            convert_spec_op_binary(fenv, vars, loc, *node_id, BinOp_::Add, args, locals, block)
        }
        SpecExp::Call(node_id, Operation::Sub, args) => {
            convert_spec_op_binary(fenv, vars, loc, *node_id, BinOp_::Sub, args, locals, block)
        }
        SpecExp::Call(node_id, Operation::Mul, args) => {
            convert_spec_op_binary(fenv, vars, loc, *node_id, BinOp_::Mul, args, locals, block)
        }
        SpecExp::Call(node_id, Operation::Mod, args) => {
            convert_spec_op_binary(fenv, vars, loc, *node_id, BinOp_::Mod, args, locals, block)
        }
        SpecExp::Call(node_id, Operation::Div, args) => {
            convert_spec_op_binary(fenv, vars, loc, *node_id, BinOp_::Div, args, locals, block)
        }
        SpecExp::Call(node_id, Operation::BitOr, args) => convert_spec_op_binary(
            fenv,
            vars,
            loc,
            *node_id,
            BinOp_::BitOr,
            args,
            locals,
            block,
        ),
        SpecExp::Call(node_id, Operation::BitAnd, args) => convert_spec_op_binary(
            fenv,
            vars,
            loc,
            *node_id,
            BinOp_::BitAnd,
            args,
            locals,
            block,
        ),
        SpecExp::Call(node_id, Operation::Xor, args) => {
            convert_spec_op_binary(fenv, vars, loc, *node_id, BinOp_::Xor, args, locals, block)
        }
        SpecExp::Call(node_id, Operation::Shl, args) => {
            convert_spec_op_binary(fenv, vars, loc, *node_id, BinOp_::Shl, args, locals, block)
        }
        SpecExp::Call(node_id, Operation::Shr, args) => {
            convert_spec_op_binary(fenv, vars, loc, *node_id, BinOp_::Shr, args, locals, block)
        }
        SpecExp::Call(node_id, Operation::And, args) => {
            convert_spec_op_binary(fenv, vars, loc, *node_id, BinOp_::And, args, locals, block)
        }
        SpecExp::Call(node_id, Operation::Or, args) => {
            convert_spec_op_binary(fenv, vars, loc, *node_id, BinOp_::Or, args, locals, block)
        }
        SpecExp::Call(node_id, Operation::Eq, args) => {
            convert_spec_op_binary(fenv, vars, loc, *node_id, BinOp_::Eq, args, locals, block)
        }
        SpecExp::Call(node_id, Operation::Neq, args) => {
            convert_spec_op_binary(fenv, vars, loc, *node_id, BinOp_::Neq, args, locals, block)
        }
        SpecExp::Call(node_id, Operation::Lt, args) => {
            convert_spec_op_binary(fenv, vars, loc, *node_id, BinOp_::Lt, args, locals, block)
        }
        SpecExp::Call(node_id, Operation::Gt, args) => {
            convert_spec_op_binary(fenv, vars, loc, *node_id, BinOp_::Gt, args, locals, block)
        }
        SpecExp::Call(node_id, Operation::Le, args) => {
            convert_spec_op_binary(fenv, vars, loc, *node_id, BinOp_::Le, args, locals, block)
        }
        SpecExp::Call(node_id, Operation::Ge, args) => {
            convert_spec_op_binary(fenv, vars, loc, *node_id, BinOp_::Ge, args, locals, block)
        }
        // TODO (mengxu) implement rest of operations for call
        SpecExp::Call(_, _, _) => unimplemented!("call"),
        // TODO (mengxu) handle these unimplemented cases
        SpecExp::LocalVar(..) => unimplemented!("local variables (in quantifiers and lambda)"),
        SpecExp::SpecVar(..) => unimplemented!("ghost variables"),
        SpecExp::Lambda(..) => unimplemented!("lambda definition"),
        SpecExp::Invoke(..) => unimplemented!("lambda invocation"),
        SpecExp::Quant(..) => unimplemented!("quantifiers"),
        SpecExp::Block(..) => unimplemented!("block"),
        SpecExp::IfElse(..) => unimplemented!("if-else"),
        // a valid spec-lang ast should not have invalid spec expressions
        SpecExp::Invalid(..) => unreachable!(),
    }
}

fn convert_spec_op_unary(
    fenv: &FunctionEnv<'_>,
    vars: &BTreeMap<TempIndex, Var>,
    loc: MoveLoc,
    node: NodeId,
    op: UnaryOp_,
    args: &[SpecExp],
    locals: &mut UniqueMap<Var, SingleType>,
    block: &mut Block,
) -> MoveExp {
    let env = fenv.module_env.env;
    if args.len() != 1 {
        env.error(
            &env.to_loc(&loc),
            &format!("Expect 1 operand for UnaryOp, got: {}", args.len()),
        );
        return unresolved_move_expression(loc);
    }
    let move_ty = convert_spec_type(fenv, loc, &env.get_node_type(node));
    let mut converted_args: Vec<_> = args
        .iter()
        .map(|arg| convert_spec_expression(fenv, vars, loc, arg, locals, block))
        .collect();
    let operand = converted_args.pop().unwrap();
    MoveExp {
        ty: sp(loc, move_ty),
        exp: sp(
            loc,
            UnannotatedExp_::UnaryExp(sp(loc, op), Box::new(operand)),
        ),
    }
}

fn convert_spec_op_binary(
    fenv: &FunctionEnv<'_>,
    vars: &BTreeMap<TempIndex, Var>,
    loc: MoveLoc,
    node: NodeId,
    op: BinOp_,
    args: &[SpecExp],
    locals: &mut UniqueMap<Var, SingleType>,
    block: &mut Block,
) -> MoveExp {
    let env = fenv.module_env.env;
    if args.len() != 2 {
        env.error(
            &env.to_loc(&loc),
            &format!("Expect 2 operands for BinOp, got: {}", args.len()),
        );
        return unresolved_move_expression(loc);
    }
    let move_ty = convert_spec_type(fenv, loc, &env.get_node_type(node));
    let mut converted_args: Vec<_> = args
        .iter()
        .map(|arg| convert_spec_expression(fenv, vars, loc, arg, locals, block))
        .collect();
    let rhs = converted_args.pop().unwrap();
    let lhs = converted_args.pop().unwrap();
    MoveExp {
        ty: sp(loc, move_ty),
        exp: sp(
            loc,
            UnannotatedExp_::BinopExp(Box::new(lhs), sp(loc, op), Box::new(rhs)),
        ),
    }
}

fn convert_spec_value(env: &GlobalEnv, loc: MoveLoc, node: NodeId, value: &SpecValue) -> MoveExp {
    let (move_val, move_ty, has_error) = match value {
        SpecValue::Address(v) => {
            let addr = Address::parse_str(&v.to_str_radix(16)).unwrap();
            (
                MoveValue::Address(addr),
                MoveType::Single(sp(loc, SingleType_::Base(BaseType_::address(loc)))),
                false,
            )
        }
        SpecValue::Number(v) => {
            let spec_ty = env.get_node_type(node);
            match spec_ty {
                SpecType::Primitive(SpecPrimitiveType::U8) => (
                    MoveValue::U8(v.to_u8().unwrap()),
                    MoveType::Single(SingleType_::u8(loc)),
                    false,
                ),
                SpecType::Primitive(SpecPrimitiveType::U64) => (
                    MoveValue::U64(v.to_u64().unwrap()),
                    MoveType::Single(SingleType_::u64(loc)),
                    false,
                ),
                SpecType::Primitive(SpecPrimitiveType::U128) => (
                    MoveValue::U128(v.to_u128().unwrap()),
                    MoveType::Single(SingleType_::u128(loc)),
                    false,
                ),
                _ => {
                    env.error(
                        &env.to_loc(&loc),
                        "Invalid type for the Number variant in movel-model AST values",
                    );
                    (
                        MoveValue::U64(v.to_u64().unwrap()),
                        MoveType::Single(sp(loc, SingleType_::Base(BaseType_::u64(loc)))),
                        true,
                    )
                }
            }
        }
        SpecValue::Bool(v) => (
            MoveValue::Bool(*v),
            MoveType::Single(sp(loc, SingleType_::Base(BaseType_::bool(loc)))),
            false,
        ),
        SpecValue::ByteArray(v) => (
            MoveValue::Bytearray(v.to_owned()),
            MoveType::Single(sp(
                loc,
                SingleType_::Base(BaseType_::builtin(
                    loc,
                    BuiltinTypeName_::Vector,
                    vec![BaseType_::u8(loc)],
                )),
            )),
            false,
        ),
    };
    if has_error {
        unresolved_move_expression(loc)
    } else {
        MoveExp {
            ty: sp(loc, move_ty),
            exp: sp(loc, UnannotatedExp_::Value(sp(loc, move_val))),
        }
    }
}

fn convert_spec_var_temporary(
    env: &GlobalEnv,
    vars: &BTreeMap<TempIndex, Var>,
    locals: &UniqueMap<Var, SingleType>,
    loc: MoveLoc,
    idx: TempIndex,
) -> MoveExp {
    match vars.get(&idx) {
        None => {
            env.error(
                &env.to_loc(&loc),
                "Unable to find TempIndex for this Temporary variable",
            );
            unresolved_move_expression(loc)
        }
        Some(var) => match locals.get(var) {
            None => {
                env.error(
                    &env.to_loc(&loc),
                    "Unable to find move type for this temporary variable",
                );
                unresolved_move_expression(loc)
            }
            Some(move_ty) => {
                // if the var is a base type:
                // - copy the var if the type has a Copy ability
                // - move the var otherwise
                // otherwise, the var is a reference type,
                // - always copy the var in this case
                // TODO (mengxu) I don't have much confidence in this logic, revisit
                let move_exp = match &move_ty.value {
                    SingleType_::Base(base_type) => {
                        let abilities = base_type.value.abilities(base_type.loc);
                        if abilities.has_ability_(Ability_::Copy) {
                            UnannotatedExp_::Copy {
                                from_user: false,
                                var: var.clone(),
                            }
                        } else {
                            UnannotatedExp_::Move {
                                from_user: false,
                                var: var.clone(),
                            }
                        }
                    }
                    SingleType_::Ref(..) => UnannotatedExp_::Copy {
                        from_user: false,
                        var: var.clone(),
                    },
                };
                MoveExp {
                    ty: sp(loc, MoveType::Single(move_ty.clone())),
                    exp: sp(loc, move_exp),
                }
            }
        },
    }
}

fn unresolved_move_type(loc: MoveLoc) -> MoveType {
    MoveType::Single(sp(
        loc,
        SingleType_::Base(sp(loc, BaseType_::UnresolvedError)),
    ))
}

fn unresolved_move_expression(loc: MoveLoc) -> MoveExp {
    MoveExp {
        ty: sp(loc, unresolved_move_type(loc)),
        exp: sp(loc, UnannotatedExp_::UnresolvedError),
    }
}
