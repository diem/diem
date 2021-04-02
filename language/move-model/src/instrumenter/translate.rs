// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;

use move_ir_types::{location::sp, sp};
use move_lang::{
    compiled_unit::CompiledUnit,
    naming::ast::{
        Exp as MoveExp, ExpDotted, ExpDotted_, Exp_ as MoveExp_, Function, FunctionBody,
        FunctionBody_, ModuleDefinition, Program, Script, Sequence, SequenceItem as MoveStmt,
        SequenceItem_ as MoveStmt_,
    },
    parser::ast::{FunctionName, ModuleIdent},
    shared::{unique_map::UniqueMap, Address, Identifier},
};

use crate::{
    instrumenter::context::{CompilationContext, InstrumentedFunction, SpecTranslationContext},
    model::{FunctionEnv, GlobalEnv, ModuleEnv},
    spec_translator::SpecTranslator,
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
                Some(CompilationContext::new(
                    source_map.clone(),
                    function_infos.clone(),
                ))
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

    // construct the new module field by field
    let ModuleDefinition {
        attributes,
        is_source_module,
        dependency_order,
        friends,
        structs,
        functions: efunctions,
        constants,
    } = mdef;
    let functions = efunctions.map(|name, func| function(&menv, &ctxt, name, func));

    // re-pack the module definition
    ModuleDefinition {
        attributes,
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
                Some(CompilationContext::new(source_map.clone(), function_infos))
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

    // construct the new script field by field
    let Script {
        attributes,
        loc,
        constants,
        function_name,
        function: efunction,
    } = sdef;
    let function = function(&menv, &ctxt, function_name.clone(), efunction);

    // re-pack the script definition
    Script {
        attributes,
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

    // construct the new function field by field
    let Function {
        attributes,
        visibility,
        signature,
        acquires,
        body: ebody,
    } = func;
    let body = function_body(&fenv, ctxt, ebody);

    // re-pack the function definition
    Function {
        attributes,
        visibility,
        signature,
        acquires,
        body,
    }
}

fn function_body(
    fenv: &FunctionEnv<'_>,
    ctxt: &CompilationContext,
    sp!(loc, body): FunctionBody,
) -> FunctionBody {
    match body {
        FunctionBody_::Native => sp(loc, FunctionBody_::Native),
        FunctionBody_::Defined(seq) => {
            // translate the raw function-level spec
            let mut trans_ctxt = SpecTranslationContext::new(fenv);
            let trans_spec =
                SpecTranslator::translate_fun_spec(false, &mut trans_ctxt, fenv, &[], None, None);

            // perform instrumentation with a context
            let new_seq =
                match InstrumentedFunction::new(fenv, ctxt, &trans_spec, &trans_ctxt.local_types) {
                    None => seq,
                    Some(mut instrumented) => {
                        // first instrument the pre-conditions
                        instrumented.instrument_pre_conditions();

                        // then handle the function body
                        for item in seq {
                            statement(&mut instrumented, item);
                        }

                        // extract the instrumented body from the context
                        let new_seq = instrumented.pop_scope();
                        assert!(instrumented.end_of_life());
                        new_seq
                    }
                };
            sp(loc, FunctionBody_::Defined(new_seq))
        }
    }
}

//
// Statements and Expressions
//

fn sequence(instrumented: &mut InstrumentedFunction, seq: Sequence) -> Sequence {
    instrumented.push_scope();
    for item in seq {
        statement(instrumented, item);
    }
    instrumented.pop_scope()
}

fn statement(instrumented: &mut InstrumentedFunction, sp!(loc, stmt): MoveStmt) {
    let new_stmt = match stmt {
        MoveStmt_::Seq(exp) => {
            let new_exp = expression(instrumented, exp);
            MoveStmt_::Seq(new_exp)
        }
        item @ MoveStmt_::Declare(..) => item,
        MoveStmt_::Bind(lvals, exp) => {
            let new_exp = expression(instrumented, exp);
            MoveStmt_::Bind(lvals, new_exp)
        }
    };
    instrumented.add_statement(sp(loc, new_stmt));
}

fn expression(instrumented: &mut InstrumentedFunction, sp!(loc, exp): MoveExp) -> MoveExp {
    fn expression_dotted(
        instrumented: &mut InstrumentedFunction,
        sp!(loc, exp): ExpDotted,
    ) -> ExpDotted {
        let new_exp = match exp {
            ExpDotted_::Exp(sub_exp) => {
                let new_sub_exp = expression(instrumented, *sub_exp);
                ExpDotted_::Exp(Box::new(new_sub_exp))
            }
            ExpDotted_::Dot(sub_dot, field) => {
                let new_sub_dot = expression_dotted(instrumented, *sub_dot);
                ExpDotted_::Dot(Box::new(new_sub_dot), field)
            }
        };
        sp(loc, new_exp)
    }

    let new_exp = match exp {
        // point of instrumentation
        MoveExp_::Spec(spec_id, used_vars) => {
            instrumented.instrument_inline_spec(&loc, &spec_id, &used_vars);
            MoveExp_::Spec(spec_id, used_vars)
        }

        // propagate other cases
        exp_ @ MoveExp_::Unit { .. } => exp_,
        exp_ @ MoveExp_::Value(..) => exp_,
        exp_ @ MoveExp_::InferredNum(..) => exp_,
        exp_ @ MoveExp_::Move { .. } => exp_,
        exp_ @ MoveExp_::Copy { .. } => exp_,
        exp_ @ MoveExp_::Use { .. } => exp_,
        exp_ @ MoveExp_::Constant(..) => exp_,

        MoveExp_::ModuleCall(callee_module, callee_function, ty_args, sp!(args_loc, args)) => {
            let new_args = args
                .into_iter()
                .map(|arg_exp| expression(instrumented, arg_exp))
                .collect();
            MoveExp_::ModuleCall(
                callee_module,
                callee_function,
                ty_args,
                sp(args_loc, new_args),
            )
        }
        MoveExp_::Builtin(builtin, sp!(args_loc, args)) => {
            let new_args = args
                .into_iter()
                .map(|arg_exp| expression(instrumented, arg_exp))
                .collect();
            MoveExp_::Builtin(builtin, sp(args_loc, new_args))
        }

        MoveExp_::IfElse(cond_exp, then_exp, else_exp) => {
            let new_cond_exp = expression(instrumented, *cond_exp);
            let new_then_exp = expression(instrumented, *then_exp);
            let new_else_exp = expression(instrumented, *else_exp);
            MoveExp_::IfElse(
                Box::new(new_cond_exp),
                Box::new(new_then_exp),
                Box::new(new_else_exp),
            )
        }
        MoveExp_::While(cond_exp, body_exp) => {
            let new_cond_exp = expression(instrumented, *cond_exp);
            let new_body_exp = expression(instrumented, *body_exp);
            MoveExp_::While(Box::new(new_cond_exp), Box::new(new_body_exp))
        }
        MoveExp_::Loop(body_exp) => {
            let new_body_exp = expression(instrumented, *body_exp);
            MoveExp_::Loop(Box::new(new_body_exp))
        }
        MoveExp_::Block(seq) => {
            let new_seq = sequence(instrumented, seq);
            MoveExp_::Block(new_seq)
        }

        MoveExp_::Assign(lvals, rhs_exp) => {
            let new_rhs_exp = expression(instrumented, *rhs_exp);
            MoveExp_::Assign(lvals, Box::new(new_rhs_exp))
        }
        MoveExp_::FieldMutate(dotted_exp, rhs_exp) => {
            let new_lhs_dot = expression_dotted(instrumented, dotted_exp);
            let new_rhs_exp = expression(instrumented, *rhs_exp);
            MoveExp_::FieldMutate(new_lhs_dot, Box::new(new_rhs_exp))
        }
        MoveExp_::Mutate(lhs_exp, rhs_exp) => {
            let new_lhs_exp = expression(instrumented, *lhs_exp);
            let new_rhs_exp = expression(instrumented, *rhs_exp);
            MoveExp_::Mutate(Box::new(new_lhs_exp), Box::new(new_rhs_exp))
        }

        MoveExp_::Return(sub_exp) => {
            // first instrument post conditions
            instrumented.instrument_post_conditions();

            // then return normally
            let new_sub_exp = expression(instrumented, *sub_exp);
            MoveExp_::Return(Box::new(new_sub_exp))
        }
        MoveExp_::Abort(sub_exp) => {
            let new_sub_exp = expression(instrumented, *sub_exp);
            MoveExp_::Abort(Box::new(new_sub_exp))
        }
        exp_ @ MoveExp_::Break => exp_,
        exp_ @ MoveExp_::Continue => exp_,

        MoveExp_::Dereference(ref_exp) => {
            let new_ref_exp = expression(instrumented, *ref_exp);
            MoveExp_::Dereference(Box::new(new_ref_exp))
        }
        MoveExp_::UnaryExp(op, sub_exp) => {
            let new_sub_exp = expression(instrumented, *sub_exp);
            MoveExp_::UnaryExp(op, Box::new(new_sub_exp))
        }
        MoveExp_::BinopExp(lhs_exp, op, rhs_exp) => {
            let new_lhs_exp = expression(instrumented, *lhs_exp);
            let new_rhs_exp = expression(instrumented, *rhs_exp);
            MoveExp_::BinopExp(Box::new(new_lhs_exp), op, Box::new(new_rhs_exp))
        }

        MoveExp_::Pack(struct_module, struct_name, type_args, fields) => {
            let new_fields = fields
                .map(|_, (field_idx, field_exp)| (field_idx, expression(instrumented, field_exp)));
            MoveExp_::Pack(struct_module, struct_name, type_args, new_fields)
        }
        MoveExp_::DerefBorrow(dotted_exp) => {
            let new_dotted_exp = expression_dotted(instrumented, dotted_exp);
            MoveExp_::DerefBorrow(new_dotted_exp)
        }
        MoveExp_::Borrow(is_mut, dotted_exp) => {
            let new_dotted_exp = expression_dotted(instrumented, dotted_exp);
            MoveExp_::Borrow(is_mut, new_dotted_exp)
        }

        MoveExp_::Cast(sub_exp, ty) => {
            let new_sub_exp = expression(instrumented, *sub_exp);
            MoveExp_::Cast(Box::new(new_sub_exp), ty)
        }
        MoveExp_::Annotate(sub_exp, ty) => {
            let new_sub_exp = expression(instrumented, *sub_exp);
            MoveExp_::Annotate(Box::new(new_sub_exp), ty)
        }

        MoveExp_::ExpList(items) => {
            let new_items = items
                .into_iter()
                .map(|item| expression(instrumented, item))
                .collect();
            MoveExp_::ExpList(new_items)
        }

        MoveExp_::UnresolvedError => unreachable!(),
    };
    sp(loc, new_exp)
}
