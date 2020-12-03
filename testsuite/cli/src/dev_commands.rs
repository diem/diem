// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    client_proxy::ClientProxy,
    commands::{subcommand_execute, Command},
};
use chrono::{DateTime, Utc};
use diem_types::waypoint::Waypoint;
use std::time::{Duration, UNIX_EPOCH};

/// Major command for account related operations.
pub struct DevCommand {}

impl Command for DevCommand {
    fn get_aliases(&self) -> Vec<&'static str> {
        vec!["dev", "d"]
    }
    fn get_description(&self) -> &'static str {
        "Local Move development"
    }
    fn execute(&self, client: &mut ClientProxy, params: &[&str]) {
        let commands: Vec<Box<dyn Command>> = vec![
            Box::new(DevCommandCompile {}),
            Box::new(DevCommandPublish {}),
            Box::new(DevCommandExecute {}),
            Box::new(DevCommandUpgradeStdlib {}),
            Box::new(DevCommandGenWaypoint {}),
            Box::new(DevCommandChangeDiemVersion {}),
            Box::new(DevCommandEnableCustomScript {}),
            Box::new(AddToScriptAllowList {}),
        ];
        subcommand_execute(&params[0], commands, client, &params[1..]);
    }
}

/// Sub command to compile a Move program
pub struct DevCommandCompile {}

impl Command for DevCommandCompile {
    fn get_aliases(&self) -> Vec<&'static str> {
        vec!["compile", "c"]
    }
    fn get_params_help(&self) -> &'static str {
        "<sender_account_address>|<sender_account_ref_id> <file_path> <dependency_source_files...>"
    }
    fn get_description(&self) -> &'static str {
        "Compile Move program"
    }
    fn execute(&self, client: &mut ClientProxy, params: &[&str]) {
        if params.len() < 3 {
            println!("Invalid number of arguments for compilation");
            return;
        }
        println!(">> Compiling program");
        match client.compile_program(params) {
            Ok(paths) => {
                println!("Successfully compiled a program at:");
                for p in paths {
                    println!("  {}", p);
                }
            }
            Err(e) => println!("{}", e),
        }
    }
}

/// Sub command to publish a Move resource
pub struct DevCommandPublish {}

impl Command for DevCommandPublish {
    fn get_aliases(&self) -> Vec<&'static str> {
        vec!["publish", "p"]
    }

    fn get_params_help(&self) -> &'static str {
        "<sender_account_address>|<sender_account_ref_id> <compiled_module_path>"
    }

    fn get_description(&self) -> &'static str {
        "Publish Move module on-chain"
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

/// Sub command to execute a custom Move script
pub struct DevCommandExecute {}

impl Command for DevCommandExecute {
    fn get_aliases(&self) -> Vec<&'static str> {
        vec!["execute", "e"]
    }

    fn get_params_help(&self) -> &'static str {
        "<sender_account_address>|<sender_account_ref_id> <compiled_module_path> [parameters]"
    }

    fn get_description(&self) -> &'static str {
        "Execute custom Move script"
    }

    fn execute(&self, client: &mut ClientProxy, params: &[&str]) {
        if params.len() < 3 {
            println!("Invalid number of arguments to execute script");
            return;
        }
        match client.execute_script(params) {
            Ok(_) => println!("Successfully finished execution"),
            Err(e) => println!("{}", e),
        }
    }
}

pub struct DevCommandEnableCustomScript {}

impl Command for DevCommandEnableCustomScript {
    fn get_aliases(&self) -> Vec<&'static str> {
        vec!["enable_custom_script", "s"]
    }

    fn get_params_help(&self) -> &'static str {
        ""
    }

    fn get_description(&self) -> &'static str {
        "Allow executing arbitrary script in the network. This disables script hash verification."
    }

    fn execute(&self, client: &mut ClientProxy, params: &[&str]) {
        if params.len() != 1 {
            println!("Invalid number of arguments");
            return;
        }
        match client.enable_custom_script(params, true) {
            Ok(_) => println!("Successfully finished execution"),
            Err(e) => println!("{}", e),
        }
    }
}

pub struct AddToScriptAllowList {}

impl Command for AddToScriptAllowList {
    fn get_aliases(&self) -> Vec<&'static str> {
        vec!["add_to_script_allow_list", "a"]
    }

    fn get_params_help(&self) -> &'static str {
        "<hash>"
    }

    fn get_description(&self) -> &'static str {
        "Add a script hash to the allow list. This enables script hash verification."
    }

    fn execute(&self, client: &mut ClientProxy, params: &[&str]) {
        if params.len() != 2 {
            println!("Invalid number of arguments");
            return;
        }
        match client.add_to_script_allow_list(params, true) {
            Ok(_) => println!("Successfully finished execution"),
            Err(e) => println!("{}", e),
        }
    }
}

pub struct DevCommandChangeDiemVersion {}

impl Command for DevCommandChangeDiemVersion {
    fn get_aliases(&self) -> Vec<&'static str> {
        vec!["change_diem_version", "v"]
    }

    fn get_params_help(&self) -> &'static str {
        "<new_diem_version>"
    }

    fn get_description(&self) -> &'static str {
        "Change the diem_version stored on chain"
    }

    fn execute(&self, client: &mut ClientProxy, params: &[&str]) {
        if params.len() != 2 {
            println!("Invalid number of arguments");
            return;
        }
        match client.change_diem_version(params, true) {
            Ok(_) => println!("Successfully finished execution"),
            Err(e) => println!("{}", e),
        }
    }
}

pub struct DevCommandUpgradeStdlib {}

impl Command for DevCommandUpgradeStdlib {
    fn get_aliases(&self) -> Vec<&'static str> {
        vec!["upgrade_stdlib", "u"]
    }

    fn get_params_help(&self) -> &'static str {
        ""
    }

    fn get_description(&self) -> &'static str {
        "Upgrade the move stdlib used for the blockchain"
    }

    fn execute(&self, client: &mut ClientProxy, params: &[&str]) {
        if params.len() != 1 {
            println!("Invalid number of arguments");
            return;
        }
        match client.upgrade_stdlib(params, true) {
            Ok(_) => println!("Successfully finished execution"),
            Err(e) => println!("{}", e),
        }
    }
}

pub struct DevCommandGenWaypoint {}

impl Command for DevCommandGenWaypoint {
    fn get_aliases(&self) -> Vec<&'static str> {
        vec!["gen_waypoint", "g"]
    }

    fn get_params_help(&self) -> &'static str {
        ""
    }

    fn get_description(&self) -> &'static str {
        "Generate a waypoint for the latest epoch change LedgerInfo"
    }

    fn execute(&self, client: &mut ClientProxy, params: &[&str]) {
        if params.len() != 1 {
            println!("No parameters required for waypoint generation");
            return;
        }
        println!("Retrieving the uptodate ledger info...");
        if let Err(e) = client.test_validator_connection() {
            println!("Failed to get uptodate ledger info connection: {}", e);
            return;
        }

        let latest_epoch_change_li = match client.latest_epoch_change_li() {
            Some(li) => li,
            None => {
                println!("No epoch change LedgerInfo found");
                return;
            }
        };
        let li_time_str = DateTime::<Utc>::from(
            UNIX_EPOCH
                + Duration::from_micros(latest_epoch_change_li.ledger_info().timestamp_usecs()),
        );
        match Waypoint::new_epoch_boundary(latest_epoch_change_li.ledger_info()) {
            Err(e) => println!("Failed to generate a waypoint: {}", e),
            Ok(waypoint) => println!(
                "Waypoint (end of epoch {}, time {}): {}",
                latest_epoch_change_li.ledger_info().epoch(),
                li_time_str,
                waypoint
            ),
        }
    }
}
