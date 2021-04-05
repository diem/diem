// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod context;

use std::collections::BTreeMap;

use move_ir_types::{location::sp, sp};
use move_lang::{
    compiled_unit::{CompiledUnit, FunctionInfo},
    expansion::ast::{
        Exp_, Function, FunctionBody, FunctionBody_, ModuleDefinition, Program, Script, Sequence,
        SequenceItem_,
    },
    parser::ast::{FunctionName, ModuleIdent},
    shared::{ast_debug, unique_map::UniqueMap, Address, Identifier},
};

use crate::{
    instrumenter::context::CompilationContext,
    model::{FunctionEnv, GlobalEnv, ModuleEnv},
};

// =================================================================================================
// Entry point
pub(crate) fn run(env: &GlobalEnv, units: &[CompiledUnit], ast: Program) -> Program {
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
        loc,
        is_source_module,
        dependency_order,
        friends,
        structs,
        functions: efunctions,
        constants,
        specs,
    } = mdef;
    let functions = efunctions.map(|name, func| function(&menv, ctxt, name, func));

    // re-pack the module definition
    ModuleDefinition {
        loc,
        is_source_module,
        dependency_order,
        friends,
        structs,
        functions,
        constants,
        specs,
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
        specs,
    } = sdef;
    let function = function(menv, ctxt, function_name.clone(), efunction);

    // re-pack the script definition
    Script {
        loc,
        constants,
        function_name,
        function,
        specs,
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
        loc,
        visibility,
        signature,
        acquires,
        body: ebody,
        specs,
    } = func;
    let body = function_body(&fenv, info, ebody);

    // re-pack the function definition
    Function {
        loc,
        visibility,
        signature,
        acquires,
        body,
        specs,
    }
}

fn function_body(
    fenv: &FunctionEnv<'_>,
    info: &FunctionInfo,
    sp!(loc, body): FunctionBody,
) -> FunctionBody {
    match body {
        FunctionBody_::Native => sp(loc, FunctionBody_::Native),
        FunctionBody_::Defined(seq) => {
            sp(loc, FunctionBody_::Defined(sequence_move(fenv, info, seq)))
        }
    }
}

//
// Statement
//

fn sequence_move(fenv: &FunctionEnv<'_>, info: &FunctionInfo, seq: Sequence) -> Sequence {
    let env = fenv.module_env.env;
    let spec = fenv.get_spec();
    let mut instrumented = Sequence::new();
    for (idx, item) in seq.into_iter().enumerate() {
        // TODO (mengxu): remove debugging code
        if cfg!(debug_assertions) {
            print!("{}: ", idx);
            ast_debug::print_verbose(&item);
        }
        match &item.value {
            SequenceItem_::Seq(exp) => match &exp.value {
                Exp_::Spec(spec_id, _) => {
                    match info.spec_info.get(spec_id) {
                        None => env.error(
                            &env.to_loc(&exp.loc),
                            "Unable to find the CodeOffset in FunctionInfo for this spec block",
                        ),
                        Some(spec_info) => match spec.on_impl.get(&spec_info.offset) {
                            None => env.error(
                                &env.to_loc(&exp.loc),
                                "Unable to find the Spec in FunctionEnv for this spec block",
                            ),
                            Some(inline_spec) => {
                                // TODO (mengxu) replace with expr hanndler
                                assert!(inline_spec.loc.is_some())
                            }
                        },
                    }
                }
                Exp_::UnresolvedError => unreachable!(),
                _ => {}
            },
            SequenceItem_::Bind(_, exp) => match &exp.value {
                Exp_::Spec(spec_id, _) => {
                    let offset = info.spec_info.get(spec_id).unwrap().offset;
                    assert!(spec.on_impl.contains_key(&offset));
                }
                Exp_::UnresolvedError => unreachable!(),
                _ => {}
            },
            SequenceItem_::Declare(..) => {}
        }
        instrumented.push_back(item);
    }
    instrumented
}
