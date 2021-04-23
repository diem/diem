// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use cli::client_proxy::ClientProxy;
use gag::Gag;
use rustyline::{config::CompletionType, Config, Editor};
use std::{collections::HashMap, rc::Rc};

mod get_sequence_number;
mod get_writeset_version;

pub trait Experiment {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn setup_states(&self, client: &mut ClientProxy) -> Result<()>;
    fn hint(&self) -> &'static str;
    fn check(&self, client: &mut ClientProxy, input: &str) -> Result<bool>;
    fn reset_states(&self, client: &mut ClientProxy) -> Result<()>;
}

pub fn run_experiment(client: &mut ClientProxy, experiment: Rc<dyn Experiment>) -> Result<()> {
    println!("Running Experiment {}\n", experiment.name());
    println!("{}", experiment.description());
    {
        let _print_gag = Gag::stdout().unwrap();
        experiment.setup_states(client)?;
    }
    println!("Type 'h' for hints! ");
    println!("Ctrl-c to exit");

    let config = Config::builder()
        .history_ignore_space(true)
        .completion_type(CompletionType::List)
        .auto_add_history(true)
        .build();

    let mut rl = Editor::<()>::with_config(config);
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                if line.as_str() == "h" {
                    println!("{}", experiment.hint());
                    continue;
                }
                if experiment.check(client, line.as_str())? {
                    println!("That's right!");
                    break;
                }
            }
            Err(_) => {
                break;
            }
        }
    }

    experiment.reset_states(client)
}

pub fn experiments() -> HashMap<&'static str, Rc<dyn Experiment>> {
    let experiments: Vec<Rc<dyn Experiment>> = vec![
        Rc::new(get_sequence_number::GetSequenceNumber()),
        Rc::new(get_writeset_version::GetWriteSetVersion()),
    ];

    let mut name_to_experiment = HashMap::new();
    for exp in experiments {
        name_to_experiment.insert(exp.name(), exp);
    }

    name_to_experiment
}
