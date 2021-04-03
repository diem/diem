// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;

use move_ir_types::{location::sp, sp};
use move_lang::{
    expansion::ast::{
        Function, FunctionBody, FunctionBody_, ModuleDefinition, Program, Script, Sequence,
    },
    parser::ast::{FunctionName, ModuleIdent},
    shared::{unique_map::UniqueMap, Address, Identifier},
};

use crate::model::{FunctionEnv, GlobalEnv, ModuleEnv};

// =================================================================================================
// Entry point
pub(crate) fn run(env: &GlobalEnv, ast: Program) -> Program {
    let Program {
        modules: emodules,
        scripts: escripts,
    } = ast;
    let modules = modules(env, emodules);
    let scripts = scripts(env, escripts);
    Program { modules, scripts }
}

fn modules(
    env: &GlobalEnv,
    modules: UniqueMap<ModuleIdent, ModuleDefinition>,
) -> UniqueMap<ModuleIdent, ModuleDefinition> {
    modules.map(|ident, mdef| module(env, ident, mdef))
}

fn scripts(env: &GlobalEnv, scripts: BTreeMap<String, Script>) -> BTreeMap<String, Script> {
    scripts
        .into_iter()
        .map(|(n, s)| (n, script(env, s)))
        .collect()
}

fn module(env: &GlobalEnv, ident: ModuleIdent, mdef: ModuleDefinition) -> ModuleDefinition {
    // get the module env
    let module_data = env.module_data.iter().find(|m| {
        let module_id = m.module.self_id();
        module_id.address().as_ref() == ident.value.0.as_ref()
            && module_id.name().as_str() == ident.value.1
    });
    if module_data.is_none() {
        env.error(
            &env.to_loc(&ident.loc()),
            "Unable to find ModuleData with this module id",
        );
        return mdef;
    }
    let menv = env.get_module(module_data.unwrap().id);

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
    let functions = efunctions.map(|name, f| function(&menv, name, f));

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

fn script(env: &GlobalEnv, sdef: Script) -> Script {
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
            "Unable to find ModuleData with this script",
        );
        return sdef;
    }
    let menv = env.get_module(module_data.unwrap().id);

    // construct the new script field by field
    let Script {
        loc,
        constants,
        function_name,
        function: efunction,
        specs,
    } = sdef;
    let function = function(&menv, function_name.clone(), efunction);

    // re-pack the script definition
    Script {
        loc,
        constants,
        function_name,
        function,
        specs,
    }
}

fn function(menv: &ModuleEnv<'_>, name: FunctionName, func: Function) -> Function {
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
        loc,
        visibility,
        signature,
        acquires,
        body: ebody,
        specs,
    } = func;
    let body = function_body(&fenv, ebody);

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

fn function_body(fenv: &FunctionEnv<'_>, sp!(loc, body): FunctionBody) -> FunctionBody {
    match body {
        FunctionBody_::Native => sp(loc, FunctionBody_::Native),
        FunctionBody_::Defined(seq) => sp(loc, FunctionBody_::Defined(sequence(fenv, seq))),
    }
}

fn sequence(_fenv: &FunctionEnv<'_>, seq: Sequence) -> Sequence {
    let mut instrumented = Sequence::new();
    for item in seq {
        instrumented.push_back(item);
    }
    instrumented
}
