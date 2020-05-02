// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    client_proxy::ClientProxy,
    commands::{report_error, subcommand_execute, Command},
};

/// Major command for query operations.
pub struct QueryCommand {}

impl Command for QueryCommand {
    fn get_aliases(&self) -> Vec<&'static str> {
        vec!["query", "q"]
    }
    fn get_description(&self) -> &'static str {
        "Query operations"
    }
    fn execute(&self, client: &mut ClientProxy, params: &[&str]) {
        let commands: Vec<Box<dyn Command>> = vec![
            Box::new(QueryCommandGetBalance {}),
            Box::new(QueryCommandGetSeqNum {}),
            Box::new(QueryCommandGetLatestAccountState {}),
            Box::new(QueryCommandGetTxnByAccountSeq {}),
            Box::new(QueryCommandGetTxnByRange {}),
            Box::new(QueryCommandGetEvent {}),
        ];

        subcommand_execute(&params[0], commands, client, &params[1..]);
    }
}

/// Sub commands to query balance for the account specified.
pub struct QueryCommandGetBalance {}

impl Command for QueryCommandGetBalance {
    fn get_aliases(&self) -> Vec<&'static str> {
        vec!["balance", "b"]
    }
    fn get_params_help(&self) -> &'static str {
        "<account_ref_id>|<account_address>"
    }
    fn get_description(&self) -> &'static str {
        "Get the current balances of an account"
    }
    fn execute(&self, client: &mut ClientProxy, params: &[&str]) {
        if params.len() != 2 {
            println!("Invalid number of arguments for balance query");
            return;
        }
        match client.get_balances(&params) {
            Ok(balances) => balances
                .iter()
                .for_each(|balance| println!("Balance is: {}", balance)),
            Err(e) => report_error("Failed to get balances", e),
        }
    }
}

/// Sub command to get the latest sequence number from validator for the account specified.
pub struct QueryCommandGetSeqNum {}

impl Command for QueryCommandGetSeqNum {
    fn get_aliases(&self) -> Vec<&'static str> {
        vec!["sequence", "s"]
    }
    fn get_params_help(&self) -> &'static str {
        "<account_ref_id>|<account_address> [reset_sequence_number=true|false]"
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
}

/// Command to query latest account state from validator.
pub struct QueryCommandGetLatestAccountState {}

impl Command for QueryCommandGetLatestAccountState {
    fn get_aliases(&self) -> Vec<&'static str> {
        vec!["account_state", "as"]
    }
    fn get_params_help(&self) -> &'static str {
        "<account_ref_id>|<account_address>"
    }
    fn get_description(&self) -> &'static str {
        "Get the latest state for an account"
    }
    fn execute(&self, client: &mut ClientProxy, params: &[&str]) {
        println!(">> Getting latest account state");
        match client.get_latest_account_state(&params) {
            Ok((acc, version)) => println!(
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
            Err(e) => report_error("Error getting latest account state", e),
        }
    }
}

/// Sub command  to get transaction by account and sequence number from validator.
pub struct QueryCommandGetTxnByAccountSeq {}

impl Command for QueryCommandGetTxnByAccountSeq {
    fn get_aliases(&self) -> Vec<&'static str> {
        vec!["txn_acc_seq", "ts"]
    }
    fn get_params_help(&self) -> &'static str {
        "<account_ref_id>|<account_address> <sequence_number> <fetch_events=true|false>"
    }
    fn get_description(&self) -> &'static str {
        "Get the committed transaction by account and sequence number.  \
         Optionally also fetch events emitted by this transaction."
    }
    fn execute(&self, client: &mut ClientProxy, params: &[&str]) {
        println!(">> Getting committed transaction by account and sequence number");
        match client.get_committed_txn_by_acc_seq(&params) {
            Ok(txn_view) => {
                match txn_view {
                    Some(txn_view) => {
                        println!("Committed transaction: {:#?}", txn_view);
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
                for txn_view in comm_txns_and_events {
                    println!("Transaction at version {}: {:#?}", cur_version, txn_view,);
                    cur_version += 1;
                }
            }
            Err(e) => report_error("Error getting committed transactions by range", e),
        }
    }
}

/// Sub command to query events from validator.
pub struct QueryCommandGetEvent {}

impl Command for QueryCommandGetEvent {
    fn get_aliases(&self) -> Vec<&'static str> {
        vec!["event", "ev"]
    }
    fn get_params_help(&self) -> &'static str {
        "<account_ref_id>|<account_address> <sent|received> <start_sequence_number> <limit>"
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
                        println!("{:?}", event);
                    }
                }
                println!("Last event state: {:#?}", last_event_state);
            }
            Err(e) => report_error("Error getting events by access path", e),
        }
    }
}
