// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};
use move_core_types::{
    account_address::AccountAddress,
    language_storage::TypeTag,
    parser::{parse_function_name, parse_transaction_argument, parse_type_tag},
    transaction_argument::TransactionArgument,
};
use std::collections::BTreeMap;
use yaml_rust::{yaml::Hash, Yaml};

use crate::{
    tasks::{
        MoveVMSessionTask, TaskMoveCompile, TaskMoveVMExecuteFunction, TaskMoveVMExecuteScript,
        TaskMoveVMExecuteSession, TaskMoveVMPublishModule, TaskShowMoveStorage,
    },
    types::{Config, GenesisOption, MoveSource, MoveTask, MoveTestingFlow, MoveUnit},
};

fn extract_dep(dep: &Yaml) -> Result<MoveSource> {
    match dep {
        Yaml::String(code) => return Ok(MoveSource::Embedded(code.clone())),
        Yaml::Hash(map) => {
            match map.get(&Yaml::String("code".to_owned())) {
                Some(Yaml::String(code)) => return Ok(MoveSource::Embedded(code.clone())),
                Some(_) => bail!("Bad code"),
                None => (),
            }

            match map.get(&Yaml::String("path".to_owned())) {
                Some(Yaml::String(path)) => return Ok(MoveSource::Path(path.clone())),
                Some(_) => bail!("Bad path"),
                None => (),
            }
        }
        _ => (),
    }

    bail!("Bad dep")
}

fn extract_move_unit(map: &Hash) -> Result<MoveUnit> {
    match map.get(&Yaml::String("pre-compiled".to_owned())) {
        Some(Yaml::String(var)) => return Ok(MoveUnit::Precompiled(var.to_string())),
        Some(_) => bail!("Bad var"),
        None => (),
    }

    match map.get(&Yaml::String("code".to_owned())) {
        Some(Yaml::String(code)) => {
            let deps = match map.get(&Yaml::String("deps".to_owned())) {
                Some(Yaml::Array(v)) => Some(
                    v.iter()
                        .map(|dep| extract_dep(dep))
                        .collect::<Result<Vec<_>>>()?,
                ),
                Some(_) => bail!("Bad dependencies"),
                None => None,
            };

            return Ok(MoveUnit::Source {
                source: MoveSource::Embedded(code.clone()),
                deps,
            });
        }
        Some(_) => bail!("Bad var"),
        None => (),
    }

    bail!("Module/script not found")
}

fn extract_ty_args(map: &Hash) -> Result<Vec<TypeTag>> {
    match map.get(&Yaml::String("ty-args".to_owned())) {
        Some(Yaml::Array(v)) => Ok(v
            .iter()
            .map(|ty_arg| match ty_arg {
                Yaml::String(ty_arg) => parse_type_tag(ty_arg),
                _ => bail!("Bad ty-arg"),
            })
            .collect::<Result<_>>()?),
        Some(_) => bail!("Bad ty-args"),
        None => Ok(vec![]),
    }
}

fn extract_args(map: &Hash) -> Result<Vec<TransactionArgument>> {
    match map.get(&Yaml::String("args".to_owned())) {
        Some(Yaml::Array(v)) => Ok(v
            .iter()
            .map(|ty_arg| match ty_arg {
                Yaml::String(ty_arg) => parse_transaction_argument(ty_arg),
                _ => bail!("Bad arg"),
            })
            .collect::<Result<_>>()?),
        Some(_) => bail!("Bad args"),
        None => Ok(vec![]),
    }
}

fn extract_sender(map: &Hash) -> Result<Option<AccountAddress>> {
    match map.get(&Yaml::String("sender".to_owned())) {
        Some(Yaml::String(addr)) => Ok(Some(AccountAddress::from_hex_literal(&addr)?)),
        Some(_) => bail!("Bad sender"),
        None => Ok(None),
    }
}

fn extract_senders(map: &Hash) -> Result<Option<Vec<AccountAddress>>> {
    match map.get(&Yaml::String("senders".to_owned())) {
        Some(Yaml::Array(v)) => Ok(Some(
            v.iter()
                .map(|addr| match addr {
                    Yaml::String(addr) => AccountAddress::from_hex_literal(addr),
                    _ => bail!("Bad sender"),
                })
                .collect::<Result<Vec<_>>>()?,
        )),
        Some(_) => bail!("Bad senders"),
        None => Ok(None),
    }
}

pub fn parse_task_vm_publish_module(yaml: &Yaml) -> Result<TaskMoveVMPublishModule> {
    match yaml {
        Yaml::String(code) => Ok(TaskMoveVMPublishModule::new(
            None,
            MoveUnit::Source {
                source: MoveSource::Embedded(code.clone()),
                deps: None,
            },
        )),
        Yaml::Hash(map) => {
            let sender = extract_sender(&map)?;
            let unit = extract_move_unit(&map)?;

            Ok(TaskMoveVMPublishModule::new(sender, unit))
        }
        _ => bail!("Bad task_vm_publish_module"),
    }
}

pub fn parse_task_vm_execute_script(yaml: &Yaml) -> Result<TaskMoveVMExecuteScript> {
    match yaml {
        Yaml::String(code) => Ok(TaskMoveVMExecuteScript::new(
            MoveUnit::Source {
                source: MoveSource::Embedded(code.clone()),
                deps: None,
            },
            vec![],
            vec![],
            None,
        )),
        Yaml::Hash(map) => {
            let senders = extract_senders(&map)?;
            let unit = extract_move_unit(&map)?;
            let ty_args = extract_ty_args(&map)?;
            let args = extract_args(&map)?;

            Ok(TaskMoveVMExecuteScript::new(unit, ty_args, args, senders))
        }
        _ => bail!("Bad task_vm_execute_script"),
    }
}

pub fn parse_task_vm_execute_function(yaml: &Yaml) -> Result<TaskMoveVMExecuteFunction> {
    match yaml {
        Yaml::String(full_func_name) => {
            let (module_id, func_name) = parse_function_name(&full_func_name)?;
            Ok(TaskMoveVMExecuteFunction::new(
                module_id,
                func_name,
                vec![],
                vec![],
                None,
            ))
        }
        Yaml::Hash(map) => {
            let (module_id, func_name) = match map.get(&Yaml::String("sender".to_owned())) {
                Some(Yaml::String(full_func_name)) => parse_function_name(&full_func_name)?,
                Some(_) => bail!("Bad function name"),
                None => bail!("Missing function name"),
            };
            let ty_args = extract_ty_args(&map)?;
            let args = extract_args(&map)?;
            let sender = extract_sender(&map)?;

            Ok(TaskMoveVMExecuteFunction::new(
                module_id, func_name, ty_args, args, sender,
            ))
        }
        _ => bail!("Bad task_vm_execute_function"),
    }
}

pub fn parse_task_vm_publish_module_top_level(yaml: &Yaml) -> Result<TaskMoveVMExecuteSession> {
    Ok(TaskMoveVMExecuteSession::new(vec![
        MoveVMSessionTask::PublishModule(parse_task_vm_publish_module(yaml)?),
    ]))
}

pub fn parse_task_vm_execute_function_top_level(yaml: &Yaml) -> Result<TaskMoveVMExecuteSession> {
    Ok(TaskMoveVMExecuteSession::new(vec![
        MoveVMSessionTask::ExecuteFunction(parse_task_vm_execute_function(yaml)?),
    ]))
}

pub fn parse_task_vm_execute_script_top_level(yaml: &Yaml) -> Result<TaskMoveVMExecuteSession> {
    Ok(TaskMoveVMExecuteSession::new(vec![
        MoveVMSessionTask::ExecuteScript(parse_task_vm_execute_script(yaml)?),
    ]))
}

pub fn parse_task_vm_execute_session(yaml: &Yaml) -> Result<TaskMoveVMExecuteSession> {
    match yaml {
        Yaml::Array(v) => {
            let mut tasks = vec![];
            for task in v {
                match task {
                    Yaml::Hash(map) => {
                        let task = if let Some(yaml) = map.get(&Yaml::String("module".to_owned())) {
                            MoveVMSessionTask::PublishModule(parse_task_vm_publish_module(yaml)?)
                        } else if let Some(yaml) = map.get(&Yaml::String("script".to_owned())) {
                            MoveVMSessionTask::ExecuteScript(parse_task_vm_execute_script(yaml)?)
                        } else if let Some(yaml) = map.get(&Yaml::String("call".to_owned())) {
                            MoveVMSessionTask::ExecuteFunction(parse_task_vm_execute_function(
                                yaml,
                            )?)
                        } else {
                            bail!("Bad session task")
                        };
                        tasks.push(task)
                    }
                    _ => bail!("Bad session task"),
                }
            }
            Ok(TaskMoveVMExecuteSession::new(tasks))
        }
        _ => bail!("Bad session"),
    }
}

pub fn parse_task_move_compile(yaml: &Yaml) -> Result<TaskMoveCompile> {
    match yaml {
        Yaml::Hash(map) => {
            let targets = match map.get(&Yaml::String("targets".to_string())) {
                Some(Yaml::Hash(targets)) => {
                    let mut v = BTreeMap::new();
                    for (var, src) in targets.iter() {
                        let var = match var {
                            Yaml::String(var) => var.to_string(),
                            _ => bail!("Bad var"),
                        };

                        let src = extract_dep(src)?;
                        v.insert(var, src);
                    }
                    v
                }
                Some(_) => bail!("Bad targets"),
                None => bail!("Missing targets"),
            };

            let dependencies = match map.get(&Yaml::String("deps".to_string())) {
                Some(Yaml::Array(deps)) => {
                    let mut v = vec![];
                    for dep in deps {
                        v.push(extract_dep(dep)?);
                    }
                    v
                }
                Some(_) => bail!("Bad deps"),
                None => vec![],
            };

            let sender_opt = extract_sender(&map)?;

            Ok(TaskMoveCompile::new(
                sender_opt,
                targets,
                dependencies,
                false,
            ))
        }
        _ => bail!("Bad task_move_compile"),
    }
}

pub fn parse_genesis_option(s: &str) -> Result<GenesisOption> {
    match s {
        "std" => Ok(GenesisOption::Std),
        "none" => Ok(GenesisOption::None),
        _ => bail!("unrecognized genesis option"),
    }
}

pub fn parse_config(yaml: &Yaml) -> Result<Config> {
    match yaml {
        Yaml::Hash(map) => {
            let genesis = match map.get(&Yaml::String("genesis".to_owned())) {
                Some(Yaml::String(s)) => parse_genesis_option(s)?,
                Some(Yaml::Null) => GenesisOption::None,
                Some(_) => bail!("Bad genesis option"),
                None => GenesisOption::Std,
            };
            Ok(Config { genesis })
        }
        _ => bail!("Bad config"),
    }
}

pub fn parse_move_testing_flow(yaml: &[Yaml]) -> Result<MoveTestingFlow> {
    let mut config = None;
    let mut tasks: Vec<MoveTask> = vec![];

    let mut process_top_level_task = |yaml: &Yaml| match yaml {
        Yaml::Hash(map) => {
            if map.len() != 1 {
                bail!("Bad top level task -- Have you forgotten to start a new yaml doc?");
            }
            let (name, task) = map.iter().next().unwrap();
            let name = match name {
                Yaml::String(name) => name,
                _ => bail!("Bad top level task"),
            };

            let task: MoveTask = match name.as_str() {
                "module" => parse_task_vm_publish_module_top_level(task)?.into(),
                "script" => parse_task_vm_execute_script_top_level(task)?.into(),
                "call" => parse_task_vm_execute_function_top_level(task)?.into(),
                "show-storage" => TaskShowMoveStorage.into(),
                "move-compile" => parse_task_move_compile(task)?.into(),
                _ => bail!("Bad top level task"),
            };

            tasks.push(task.into());
            Ok(())
        }
        Yaml::String(s) if s == "show-storage" => {
            tasks.push(TaskShowMoveStorage.into());
            Ok(())
        }
        _ => bail!("Bad top level task"),
    };

    let mut it = yaml.iter();
    if let Some(yaml) = it.next() {
        match yaml {
            Yaml::Hash(map) => match map.get(&Yaml::String("config".to_owned())) {
                Some(yaml) => config = Some(parse_config(yaml)?),
                _ => process_top_level_task(yaml)?,
            },
            _ => process_top_level_task(yaml)?,
        }

        for yaml in it {
            process_top_level_task(yaml)?;
        }
    }

    Ok(MoveTestingFlow {
        config: config.unwrap_or_else(Config::default),
        tasks,
    })
}
