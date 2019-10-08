// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_commands::AccountCommand, client_proxy::ClientProxy, dev_commands::DevCommand,
    query_commands::QueryCommand, transfer_commands::TransferCommand,
};

use failure::prelude::*;
use metrics::counters::*;
use std::{collections::HashMap, sync::Arc};
use types::account_address::ADDRESS_LENGTH;

/// Print the error and bump up error counter.
pub fn report_error(msg: &str, e: Error) {
    println!("[ERROR] {}: {}", msg, pretty_format_error(e));
    COUNTER_CLIENT_ERRORS.inc();
}

fn pretty_format_error(e: Error) -> String {
    if let Some(grpc_error) = e.downcast_ref::<grpcio::Error>() {
        if let grpcio::Error::RpcFailure(grpc_rpc_failure) = grpc_error {
            if grpc_rpc_failure.status == grpcio::RpcStatusCode::UNAVAILABLE
                || grpc_rpc_failure.status == grpcio::RpcStatusCode::DEADLINE_EXCEEDED
            {
                return "Server unavailable, please retry and/or check \
                        if host passed to the client is running"
                    .to_string();
            }
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

/// Returns all the commands available, as well as the reverse index from the aliases to the
/// commands.
pub fn get_commands(
    include_dev: bool,
) -> (
    Vec<Arc<dyn Command>>,
    HashMap<&'static str, Arc<dyn Command>>,
) {
    let mut commands: Vec<Arc<dyn Command>> = vec![
        Arc::new(AccountCommand::new()),
        Arc::new(QueryCommand::new()),
        Arc::new(TransferCommand {}),
    ];
    if include_dev {
        commands.push(Arc::new(DevCommand::new()));
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

/// helper to print sub command usage
pub fn print_sub_command_usage(command: &str, description: &str, param: &str) {
    println!("usage: {} <options>\n\n{}\n", command, description);
    println!("Use the following options for this command:\n\n{}\n", param);
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
    /// print usage msg
    fn print_usage(&self, params: &[&str]);
}
