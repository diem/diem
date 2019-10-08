// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{client_proxy::ClientProxy, commands::*};
use transaction_builder::get_transaction_name;
use types::account_config::get_account_resource_or_default;

use std::collections::HashMap;

/// Major command for query operations.
pub struct QueryCommand {
    pub sub_commands: Vec<Box<dyn Command>>,
    pub command_map: HashMap<&'static str, usize>,
}

impl QueryCommand {
    pub fn new() -> QueryCommand {
        let commands: Vec<Box<dyn Command>> = vec![
            Box::new(QueryCommandGetBalance {}),
            Box::new(QueryCommandGetSeqNum {}),
            Box::new(QueryCommandGetLatestAccountState {}),
            Box::new(QueryCommandGetTxnByAccountSeq {}),
            Box::new(QueryCommandGetTxnByRange {}),
            Box::new(QueryCommandGetEvent {}),
        ];
        let mut commands_map = HashMap::new();
        for (i, cmd) in commands.iter().enumerate() {
            for alias in cmd.get_aliases() {
                if commands_map.insert(alias, i) != None {
                    panic!("Duplicate alias {}", alias);
                }
            }
        }
        QueryCommand {
            sub_commands: commands,
            command_map: commands_map,
        }
    }
    fn print_helper(&self) {
        println!("usage: query <arg>\n\nUse the following sub-commands for this command:\n");
        for cmd in &self.sub_commands {
            println!(
                "{:<35}{}",
                cmd.get_aliases().join(" | "),
                cmd.get_description()
            );
        }
        println!("\n");
    }
}

impl Command for QueryCommand {
    fn get_aliases(&self) -> Vec<&'static str> {
        vec!["query", "q"]
    }
    fn get_description(&self) -> &'static str {
        "Query operations"
    }
    fn execute(&self, client: &mut ClientProxy, params: &[&str]) {
        match self.command_map.get(&params[1]) {
            Some(&idx) => self.sub_commands[idx].execute(client, &params[1..]),
            _ => self.print_usage(&params),
        }
    }
    fn print_usage(&self, params: &[&str]) {
        if params.len() > 1 {
            match self.command_map.get(&params[1]) {
                Some(&idx) => self.sub_commands[idx].print_usage(&params),
                _ => self.print_helper(),
            }
        } else {
            self.print_helper()
        }
    }
}

/// Sub commands to query balance for the account specified.
pub struct QueryCommandGetBalance {}

impl Command for QueryCommandGetBalance {
    fn get_aliases(&self) -> Vec<&'static str> {
        vec!["balance", "b"]
    }
    fn get_params_help(&self) -> &'static str {
        "<account_ref_id>\n<account_address>"
    }
    fn get_description(&self) -> &'static str {
        "Get the current balance of an account"
    }
    fn execute(&self, client: &mut ClientProxy, params: &[&str]) {
        if params.len() != 2 {
            println!("Invalid number of arguments for balance query");
            return;
        }
        match client.get_balance(&params) {
            Ok(balance) => println!("Balance is: {}", balance),
            Err(e) => report_error("Failed to get balance", e),
        }
    }
    fn print_usage(&self, _params: &[&str]) {
        print_sub_command_usage(
            "query balance",
            self.get_description(),
            self.get_params_help(),
        );
    }
}

/// Sub command to get the latest sequence number from validator for the account specified.
pub struct QueryCommandGetSeqNum {}

impl Command for QueryCommandGetSeqNum {
    fn get_aliases(&self) -> Vec<&'static str> {
        vec!["sequence", "s"]
    }
    fn get_params_help(&self) -> &'static str {
        "<account_ref_id>\n<account_address> [reset_sequence_number=true|false]"
    }
    fn get_description(&self) -> &'static str {
        "Get the current sequence number for an account, \
         and reset current sequence number in CLI (optional, default is false)"
    }
    fn execute(&self, client: &mut ClientProxy, params: &[&str]) {
        println!(">> Getting current sequence number");
        match client.get_sequence_number(&params) {
            Ok(sn) => println!("Sequence number is: {}", sn),
            Err(e) => report_error("Error getting sequence number", e),
        }
    }
    fn print_usage(&self, _params: &[&str]) {
        print_sub_command_usage(
            "query sequence",
            self.get_description(),
            self.get_params_help(),
        );
    }
}

/// Command to query latest account state from validator.
pub struct QueryCommandGetLatestAccountState {}

impl Command for QueryCommandGetLatestAccountState {
    fn get_aliases(&self) -> Vec<&'static str> {
        vec!["account_state", "as"]
    }
    fn get_params_help(&self) -> &'static str {
        "<account_ref_id>\n<account_address>"
    }
    fn get_description(&self) -> &'static str {
        "Get the latest state for an account"
    }
    fn execute(&self, client: &mut ClientProxy, params: &[&str]) {
        println!(">> Getting latest account state");
        match client.get_latest_account_state(&params) {
            Ok((acc, version)) => match get_account_resource_or_default(&acc) {
                Ok(_) => println!(
                    "Latest account state is: \n \
                     Account: {:#?}\n \
                     State: {:#?}\n \
                     Blockchain Version: {}\n",
                    client
                        .get_account_address_from_parameter(params[1])
                        .expect("Unable to parse account parameter"),
                    acc,
                    version,
                ),
                Err(e) => report_error("Error converting account blob to account resource", e),
            },
            Err(e) => report_error("Error getting latest account state", e),
        }
    }
    fn print_usage(&self, _params: &[&str]) {
        print_sub_command_usage(
            "query account_state",
            self.get_description(),
            self.get_params_help(),
        );
    }
}

/// Sub command  to get transaction by account and sequence number from validator.
pub struct QueryCommandGetTxnByAccountSeq {}

impl Command for QueryCommandGetTxnByAccountSeq {
    fn get_aliases(&self) -> Vec<&'static str> {
        vec!["txn_acc_seq", "ts"]
    }
    fn get_params_help(&self) -> &'static str {
        "<account_ref_id>\n<account_address> <sequence_number> <fetch_events=true|false>"
    }
    fn get_description(&self) -> &'static str {
        "Get the committed transaction by account and sequence number.  \
         Optionally also fetch events emitted by this transaction."
    }
    fn execute(&self, client: &mut ClientProxy, params: &[&str]) {
        println!(">> Getting committed transaction by account and sequence number");
        match client.get_committed_txn_by_acc_seq(&params) {
            Ok(txn_and_events) => {
                match txn_and_events {
                    Some((comm_txn, events)) => {
                        println!(
                            "Committed transaction: {}",
                            comm_txn.format_for_client(get_transaction_name)
                        );
                        if let Some(events_inner) = &events {
                            println!("Events: ");
                            for event in events_inner {
                                println!("{}", event);
                            }
                        }
                    }
                    None => println!("Transaction not available"),
                };
            }
            Err(e) => report_error(
                "Error getting committed transaction by account and sequence number",
                e,
            ),
        }
    }
    fn print_usage(&self, _params: &[&str]) {
        print_sub_command_usage(
            "query txn_acc_seq",
            self.get_description(),
            self.get_params_help(),
        );
    }
}

/// Sub command to query transactions by range from validator.
pub struct QueryCommandGetTxnByRange {}

impl Command for QueryCommandGetTxnByRange {
    fn get_aliases(&self) -> Vec<&'static str> {
        vec!["txn_range", "tr"]
    }
    fn get_params_help(&self) -> &'static str {
        "<start_version> <limit> <fetch_events=true|false>"
    }
    fn get_description(&self) -> &'static str {
        "Get the committed transactions by version range. \
         Optionally also fetch events emitted by these transactions."
    }
    fn execute(&self, client: &mut ClientProxy, params: &[&str]) {
        println!(">> Getting committed transaction by range");
        match client.get_committed_txn_by_range(&params) {
            Ok(comm_txns_and_events) => {
                // Note that this should never panic because we shouldn't return items
                // if the version wasn't able to be parsed in the first place
                let mut cur_version = params[1].parse::<u64>().expect("Unable to parse version");
                for (txn, opt_events) in comm_txns_and_events {
                    println!(
                        "Transaction at version {}: {}",
                        cur_version,
                        txn.format_for_client(get_transaction_name)
                    );
                    if let Some(events) = opt_events {
                        if events.is_empty() {
                            println!("No events returned");
                        } else {
                            for event in events {
                                println!("{}", event);
                            }
                        }
                    }
                    cur_version += 1;
                }
            }
            Err(e) => report_error("Error getting committed transactions by range", e),
        }
    }
    fn print_usage(&self, _params: &[&str]) {
        println!(
            "usage: query txn_range {}\n\n{}\n",
            self.get_params_help(),
            self.get_description()
        );
    }
}

/// Sub command to query events from validator.
pub struct QueryCommandGetEvent {}

impl Command for QueryCommandGetEvent {
    fn get_aliases(&self) -> Vec<&'static str> {
        vec!["event", "ev"]
    }
    fn get_params_help(&self) -> &'static str {
        "<account_ref_id>\n<account_address> <sent|received> <start_sequence_number> <ascending=true|false> <limit>"
    }
    fn get_description(&self) -> &'static str {
        "Get events by account and event type (sent|received)."
    }
    fn execute(&self, client: &mut ClientProxy, params: &[&str]) {
        println!(">> Getting events by account and event type.");
        match client.get_events_by_account_and_type(&params) {
            Ok((events, last_event_state)) => {
                if events.is_empty() {
                    println!("No events returned");
                } else {
                    for event in events {
                        println!("{}", event);
                    }
                }
                println!("Last event state: {:#?}", last_event_state);
            }
            Err(e) => report_error("Error getting events by access path", e),
        }
    }
    fn print_usage(&self, _params: &[&str]) {
        print_sub_command_usage(
            "query event",
            self.get_description(),
            self.get_params_help(),
        );
    }
}
