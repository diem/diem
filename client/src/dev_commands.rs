// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{client_proxy::ClientProxy, commands::*};

/// Major command for account related operations.
pub struct DevCommand {}

impl Command for DevCommand {
    fn get_aliases(&self) -> Vec<&'static str> {
        vec!["dev"]
    }
    fn get_description(&self) -> &'static str {
        "Local move development"
    }
    fn execute(&self, client: &mut ClientProxy, params: &[&str]) {
        let commands: Vec<Box<dyn Command>> = vec![
            Box::new(DevCommandCompile {}),
            Box::new(DevCommandPublish {}),
            Box::new(SubmitTransactionFromDiskCommand {}),
        ];
        subcommand_execute(&params[0], commands, client, &params[1..]);
    }
}

/// Sub command to compile move program
pub struct DevCommandCompile {}

impl Command for DevCommandCompile {
    fn get_aliases(&self) -> Vec<&'static str> {
        vec!["compile", "c"]
    }
    fn get_params_help(&self) -> &'static str {
        "<sender_account_address>|<sender_account_ref_id> <file_path> [output_file_path (compile into tmp file by default)]"
    }
    fn get_description(&self) -> &'static str {
        "Compile move program"
    }
    fn execute(&self, client: &mut ClientProxy, params: &[&str]) {
        if params.len() < 3 || params.len() > 4 {
            println!("Invalid number of arguments for compilation");
            return;
        }
        println!(">> Compiling program");
        match client.compile_program(params) {
            Ok(path) => println!("Successfully compiled a program at {}", path),
            Err(e) => println!("{}", e),
        }
    }
}

/// Sub command to publish move resource
pub struct DevCommandPublish {}

impl Command for DevCommandPublish {
    fn get_aliases(&self) -> Vec<&'static str> {
        vec!["publish", "p"]
    }

    fn get_params_help(&self) -> &'static str {
        "<sender_account_address>|<sender_account_ref_id> <compiled_module_path>"
    }

    fn get_description(&self) -> &'static str {
        "Publish move module on-chain"
    }

    fn execute(&self, client: &mut ClientProxy, params: &[&str]) {
        if params.len() != 3 {
            println!("Invalid number of arguments to publish module");
            return;
        }
        match client.publish_module(params) {
            Ok(_) => println!("Successfully published module"),
            Err(e) => println!("{}", e),
        }
    }
}

pub struct SubmitTransactionFromDiskCommand {}

impl Command for SubmitTransactionFromDiskCommand {
    fn get_aliases(&self) -> Vec<&'static str> {
        vec!["submit", "submitb", "s", "sb"]
    }
    fn get_description(&self) -> &'static str {
        "Load a RawTransaction from file and submit to the network"
    }
    fn get_params_help(&self) -> &'static str {
        "\n\t<signer_account_address>|<signer_account_ref_id> <path_to_raw_transaction> Suffix 'b' is for blocking. "
    }
    fn execute(&self, client: &mut ClientProxy, params: &[&str]) {
        if params.len() != 3 {
            println!(
                "Invalid number of arguments for submitting transaction, got {}",
                params.len()
            );
            return;
        }
        let is_blocking = blocking_cmd(&params[0]);
        match client.submit_transaction_from_disk(params, is_blocking) {
            Ok(index_and_seq) => {
                if is_blocking {
                    println!("Finished transaction!");
                } else {
                    println!("Transaction submitted to validator");
                }
                println!(
                    "To query for transaction status, run: query txn_acc_seq {} {} \
                     <fetch_events=true|false>",
                    index_and_seq.account_index, index_and_seq.sequence_number
                );
            }
            Err(e) => report_error("Failed to perform transaction", e),
        }
    }
}
