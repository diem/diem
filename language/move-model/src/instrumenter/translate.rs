// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;

use move_ir_types::{location::sp, sp};
use move_lang::{
    compiled_unit::{CompiledUnit, FunctionInfo},
    hlir::ast::{
        Block, Command, Command_, Exp, ExpListItem, Function, FunctionBody, FunctionBody_,
        ModuleCall, ModuleDefinition, Program, Script, SingleType, Statement, Statement_,
        UnannotatedExp_,
    },
    parser::ast::{FunctionName, ModuleIdent, Var},
    shared::{unique_map::UniqueMap, Address, Identifier},
};

use crate::{
    instrumenter::context::CompilationContext,
    model::{FunctionEnv, GlobalEnv, ModuleEnv},
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
    exp: Exp,
    locals: &mut UniqueMap<Var, SingleType>,
    block: &mut Block,
) -> Exp {
    let env = fenv.module_env.env;
    let spec = fenv.get_spec();
    let Exp {
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
                        // TODO (mengxu) replace with expr hanndler
                        assert!(inline_spec.loc.is_some())
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
        UnannotatedExp_::UnresolvedError => unreachable!(),
    };
    Exp {
        ty,
        exp: sp(loc, new_exp),
    }
}
