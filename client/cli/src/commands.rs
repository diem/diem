// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_commands::AccountCommand, client_proxy::ClientProxy, dev_commands::DevCommand,
    query_commands::QueryCommand, transfer_commands::TransferCommand,
};
use anyhow::Error;
use libra_metrics::counters::*;
use libra_types::{
    account_address::ADDRESS_LENGTH, transaction::authenticator::AUTHENTICATION_KEY_LENGTH,
};
use std::{collections::HashMap, sync::Arc};

/// Print the error and bump up error counter.
pub fn report_error(msg: &str, e: Error) {
    println!("[ERROR] {}: {}", msg, pretty_format_error(e));
    COUNTER_CLIENT_ERRORS.inc();
}

fn pretty_format_error(e: Error) -> String {
    if let Some(grpc_error) = e.downcast_ref::<tonic::Status>() {
        match grpc_error.code() {
            tonic::Code::DeadlineExceeded | tonic::Code::Unavailable => {
                return "Server unavailable, please retry and/or check \
                        if host passed to the client is running"
                    .to_string();
            }
            _ => {}
        }
    }

    return format!("{}", e);
}

/// Check whether a command is blocking.
pub fn blocking_cmd(cmd: &str) -> bool {
    cmd.ends_with('b')
}

/// Check whether a command is debugging command.
pub fn debug_format_cmd(cmd: &str) -> bool {
    cmd.ends_with('?')
}

/// Check whether the input string is a valid libra address.
pub fn is_address(data: &str) -> bool {
    match hex::decode(data) {
        Ok(vec) => vec.len() == ADDRESS_LENGTH,
        Err(_) => false,
    }
}

/// Check whether the input string is a valid libra authentication key.
pub fn is_authentication_key(data: &str) -> bool {
    match hex::decode(data) {
        Ok(vec) => vec.len() == AUTHENTICATION_KEY_LENGTH,
        Err(_) => false,
    }
}

/// Returns all the commands available, as well as the reverse index from the aliases to the
/// commands.
pub fn get_commands(
    include_dev: bool,
) -> (
    Vec<Arc<dyn Command>>,
    HashMap<&'static str, Arc<dyn Command>>,
) {
    let mut commands: Vec<Arc<dyn Command>> = vec![
        Arc::new(AccountCommand {}),
        Arc::new(QueryCommand {}),
        Arc::new(TransferCommand {}),
    ];
    if include_dev {
        commands.push(Arc::new(DevCommand {}));
    }
    let mut alias_to_cmd = HashMap::new();
    for command in &commands {
        for alias in command.get_aliases() {
            alias_to_cmd.insert(alias, Arc::clone(command));
        }
    }
    (commands, alias_to_cmd)
}

/// Parse a cmd string, the first element in the returned vector is the command to run
pub fn parse_cmd(cmd_str: &str) -> Vec<&str> {
    cmd_str.split_ascii_whitespace().collect()
}

/// Print the help message for all sub commands.
pub fn print_subcommand_help(parent_command: &str, commands: &[Box<dyn Command>]) {
    println!(
        "usage: {} <arg>\n\nUse the following args for this command:\n",
        parent_command
    );
    for cmd in commands {
        println!(
            "{} {}\n\t{}",
            cmd.get_aliases().join(" | "),
            cmd.get_params_help(),
            cmd.get_description()
        );
    }
    println!("\n");
}

/// Execute sub command.
// TODO: Convert subcommands arrays to lazy statics
pub fn subcommand_execute(
    parent_command_name: &str,
    commands: Vec<Box<dyn Command>>,
    client: &mut ClientProxy,
    params: &[&str],
) {
    let mut commands_map = HashMap::new();
    for (i, cmd) in commands.iter().enumerate() {
        for alias in cmd.get_aliases() {
            if commands_map.insert(alias, i) != None {
                panic!("Duplicate alias {}", alias);
            }
        }
    }

    if params.is_empty() {
        print_subcommand_help(parent_command_name, &commands);
        return;
    }

    match commands_map.get(&params[0]) {
        Some(&idx) => commands[idx].execute(client, &params),
        _ => print_subcommand_help(parent_command_name, &commands),
    }
}

/// Trait to perform client operations.
pub trait Command {
    /// all commands and aliases this command support.
    fn get_aliases(&self) -> Vec<&'static str>;
    /// string that describes params.
    fn get_params_help(&self) -> &'static str {
        ""
    }
    /// string that describes what the command does.
    fn get_description(&self) -> &'static str;
    /// code to execute.
    fn execute(&self, client: &mut ClientProxy, params: &[&str]);
}
