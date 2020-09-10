// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_commands::AccountCommand, client_proxy::ClientProxy, counters::COUNTER_CLIENT_ERRORS,
    dev_commands::DevCommand, info_commands::InfoCommand, query_commands::QueryCommand,
    transfer_commands::TransferCommand,
};
use anyhow::Error;
use libra_types::{account_address::AccountAddress, transaction::authenticator::AuthenticationKey};
use std::{collections::HashMap, sync::Arc};

/// Print the error and bump up error counter.
pub fn report_error(msg: &str, e: Error) {
    println!("[ERROR] {}: {}", msg, e);
    COUNTER_CLIENT_ERRORS.inc();
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
    hex::decode(data).map_or(false, |vec| vec.len() == AccountAddress::LENGTH)
}

/// Check whether the input string is a valid libra authentication key.
pub fn is_authentication_key(data: &str) -> bool {
    hex::decode(data).map_or(false, |vec| vec.len() == AuthenticationKey::LENGTH)
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
        Arc::new(InfoCommand {}),
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
