// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use bytecode::function_target_pipeline::{FunctionTargetPipeline, FunctionTargetsHolder};
use move_model::model::GlobalEnv;

mod assembly;
mod concrete;
mod shared;

use crate::assembly::{display::AstDebug, translate::translate};

/// Options passed into the interpreter generator.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct InterpreterOptions {
    /// Run the interpreter at every step of the transformation pipeline
    pub stepwise: bool,
    /// Output directory that holds the debug information of produced during interpretation
    pub dump_workdir: Option<String>,
}

//**************************************************************************************************
// Entry
//**************************************************************************************************
pub fn interpret(
    options: &InterpreterOptions,
    env: &GlobalEnv,
    pipeline: FunctionTargetPipeline,
) -> Result<()> {
    let workdir_opt = match &options.dump_workdir {
        None => None,
        Some(workdir) => {
            fs::create_dir_all(workdir)?;
            Some(workdir)
        }
    };

    // collect function targets
    let mut targets = FunctionTargetsHolder::default();
    for module_env in env.get_modules() {
        for func_env in module_env.get_functions() {
            targets.add_target(&func_env)
        }
    }

    // run through the pipeline
    if options.stepwise {
        pipeline.run_with_hook(
            env,
            &mut targets,
            |holder| stepwise_processing(env, workdir_opt, 0, "stackless", holder),
            |step, processor, holders| {
                stepwise_processing(env, workdir_opt, step, &processor.name(), holders)
            },
        );
    } else {
        pipeline.run(env, &mut targets);
        stepwise_processing(env, workdir_opt, 0, "final", &targets);
    }
    Ok(())
}

fn stepwise_processing<P: AsRef<Path>>(
    env: &GlobalEnv,
    workdir_opt: Option<P>,
    step: usize,
    name: &str,
    targets: &FunctionTargetsHolder,
) {
    match stepwise_processing_internal(env, workdir_opt, step, name, targets) {
        Ok(_) => (),
        Err(e) => panic!("Unexpected error during step {} - {}: {}", step, name, e),
    }
}

fn stepwise_processing_internal<P: AsRef<Path>>(
    env: &GlobalEnv,
    workdir_opt: Option<P>,
    step: usize,
    name: &str,
    targets: &FunctionTargetsHolder,
) -> Result<()> {
    // short-circuit the execution if prior phases run into errors
    if env.has_errors() {
        return Ok(());
    }
    let filebase_opt =
        workdir_opt.map(|workdir| workdir.as_ref().join(format!("{}_{}", step, name)));

    // dump the bytecode if requested
    if let Some(filebase) = &filebase_opt {
        let mut text = String::new();
        for module_env in env.get_modules() {
            for func_env in module_env.get_functions() {
                for (variant, target) in targets.get_targets(&func_env) {
                    if !target.data.code.is_empty() {
                        target.register_annotation_formatters_for_test();
                        text += &format!("\n[variant {}]\n{}\n", variant, target);
                    }
                }
            }
        }
        let mut file = File::create(filebase.with_extension("bytecode"))?;
        file.write_all(text.as_bytes())?;
    }

    // generate the assembly
    let program = match translate(env, targets) {
        None => {
            assert!(env.has_errors());
            return Ok(());
        }
        Some(p) => p,
    };

    // dump the translated program if requested
    if let Some(filebase) = filebase_opt {
        let mut file = File::create(filebase.with_extension("asm"))?;
        file.write_all(program.print_ast().as_bytes())?;
    }
    Ok(())
}
