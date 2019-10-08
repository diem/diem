// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{client_proxy::ClientProxy, commands::*};

use std::collections::HashMap;

/// Major command for account related operations.
pub struct DevCommand {
    pub sub_commands: Vec<Box<dyn Command>>,
    pub command_map: HashMap<&'static str, usize>,
}

impl DevCommand {
    pub fn new() -> DevCommand {
        let commands: Vec<Box<dyn Command>> = vec![
            Box::new(DevCommandCompile {}),
            Box::new(DevCommandPublish {}),
            Box::new(DevCommandExecute {}),
        ];
        let mut commands_map = HashMap::new();
        for (i, cmd) in commands.iter().enumerate() {
            for alias in cmd.get_aliases() {
                if commands_map.insert(alias, i) != None {
                    panic!("Duplicate alias {}", alias);
                }
            }
        }
        DevCommand {
            sub_commands: commands,
            command_map: commands_map,
        }
    }
    fn print_helper(&self) {
        println!("usage: dev <arg>\n\nUse the following sub-commands for this command:\n");
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

impl Command for DevCommand {
    fn get_aliases(&self) -> Vec<&'static str> {
        vec!["dev"]
    }
    fn get_description(&self) -> &'static str {
        "Local move development"
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

/// Sub command to compile move program
pub struct DevCommandCompile {}

impl Command for DevCommandCompile {
    fn get_aliases(&self) -> Vec<&'static str> {
        vec!["compile", "c"]
    }
    fn get_params_help(&self) -> &'static str {
        "<sender_account_address>\n<sender_account_ref_id> <file_path> <module|script> [output_file_path (compile into tmp file by default)]"
    }
    fn get_description(&self) -> &'static str {
        "Compile move program"
    }
    fn execute(&self, client: &mut ClientProxy, params: &[&str]) {
        if params.len() < 4 || params.len() > 5 {
            println!("Invalid number of arguments for compilation");
            return;
        }
        println!(">> Compiling program");
        match client.compile_program(params) {
            Ok(path) => println!("Successfully compiled a program at {}", path),
            Err(e) => println!("{}", e),
        }
    }
    fn print_usage(&self, _params: &[&str]) {
        print_sub_command_usage(
            "dev compile",
            self.get_description(),
            self.get_params_help(),
        );
    }
}

/// Sub command to publish move resource
pub struct DevCommandPublish {}

impl Command for DevCommandPublish {
    fn get_aliases(&self) -> Vec<&'static str> {
        vec!["publish", "p"]
    }

    fn get_params_help(&self) -> &'static str {
        "<sender_account_address>\n<sender_account_ref_id> <compiled_module_path>"
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
    fn print_usage(&self, _params: &[&str]) {
        print_sub_command_usage(
            "dev publish",
            self.get_description(),
            self.get_params_help(),
        );
    }
}

/// Sub command to execute custom move script
pub struct DevCommandExecute {}

impl Command for DevCommandExecute {
    fn get_aliases(&self) -> Vec<&'static str> {
        vec!["execute", "e"]
    }

    fn get_params_help(&self) -> &'static str {
        "<sender_account_address>\n<sender_account_ref_id> <compiled_module_path> [parameters]"
    }

    fn get_description(&self) -> &'static str {
        "Execute custom move script"
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
    fn print_usage(&self, _params: &[&str]) {
        print_sub_command_usage(
            "dev execute",
            self.get_description(),
            self.get_params_help(),
        );
    }
}
