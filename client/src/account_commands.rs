// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{client_proxy::ClientProxy, commands::*};

use std::collections::HashMap;

/// Major command for account related operations.
pub struct AccountCommand {
    pub sub_commands: Vec<Box<dyn Command>>,
    pub command_map: HashMap<&'static str, usize>,
}

impl AccountCommand {
    pub fn new() -> AccountCommand {
        let commands: Vec<Box<dyn Command>> = vec![
            Box::new(AccountCommandCreate {}),
            Box::new(AccountCommandListAccounts {}),
            Box::new(AccountCommandRecoverWallet {}),
            Box::new(AccountCommandWriteRecovery {}),
            Box::new(AccountCommandMint {}),
        ];
        let mut commands_map = HashMap::new();
        for (i, cmd) in commands.iter().enumerate() {
            for alias in cmd.get_aliases() {
                if commands_map.insert(alias, i) != None {
                    panic!("Duplicate alias {}", alias);
                }
            }
        }
        AccountCommand {
            sub_commands: commands,
            command_map: commands_map,
        }
    }
    fn print_helper(&self) {
        println!("usage: account <arg>\n\nUse the following sub-commands for this command:\n");
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

impl Command for AccountCommand {
    fn get_aliases(&self) -> Vec<&'static str> {
        vec!["account", "a"]
    }
    fn get_description(&self) -> &'static str {
        "Account operations"
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

/// Sub command to create a random account. The account will not be saved on chain.
pub struct AccountCommandCreate {}

impl Command for AccountCommandCreate {
    fn get_aliases(&self) -> Vec<&'static str> {
        vec!["create", "c"]
    }
    fn get_description(&self) -> &'static str {
        "Create an account. Returns reference ID to use in other operations"
    }
    fn execute(&self, client: &mut ClientProxy, _params: &[&str]) {
        println!(">> Creating/retrieving next account from wallet");
        match client.create_next_account(true) {
            Ok(account_data) => println!(
                "Created/retrieved account #{} address {}",
                account_data.index,
                hex::encode(account_data.address)
            ),
            Err(e) => report_error("Error creating account", e),
        }
    }
    fn print_usage(&self, _params: &[&str]) {
        println!("usage: account create\n\n{}\n", self.get_description());
    }
}

/// Sub command to recover wallet from the file specified.
pub struct AccountCommandRecoverWallet {}

impl Command for AccountCommandRecoverWallet {
    fn get_aliases(&self) -> Vec<&'static str> {
        vec!["recover", "r"]
    }
    fn get_params_help(&self) -> &'static str {
        "<file_path>"
    }
    fn get_description(&self) -> &'static str {
        "Recover Libra wallet from the file path"
    }
    fn execute(&self, client: &mut ClientProxy, params: &[&str]) {
        println!(">> Recovering Wallet");
        match client.recover_wallet_accounts(&params) {
            Ok(account_data) => {
                println!(
                    "Wallet recovered and the first {} child accounts were derived",
                    account_data.len()
                );
                for data in account_data {
                    println!("#{} address {}", data.index, hex::encode(data.address));
                }
            }
            Err(e) => report_error("Error recovering Libra wallet", e),
        }
    }
    fn print_usage(&self, _params: &[&str]) {
        println!(
            "usage: account recover {}\n\n{}\n",
            self.get_params_help(),
            self.get_description()
        );
    }
}

/// Sub command to backup wallet to the file specified.
pub struct AccountCommandWriteRecovery {}

impl Command for AccountCommandWriteRecovery {
    fn get_aliases(&self) -> Vec<&'static str> {
        vec!["write", "w"]
    }
    fn get_params_help(&self) -> &'static str {
        "<file_path>"
    }
    fn get_description(&self) -> &'static str {
        "Save Libra wallet mnemonic recovery seed to disk"
    }
    fn execute(&self, client: &mut ClientProxy, params: &[&str]) {
        println!(">> Saving Libra wallet mnemonic recovery seed to disk");
        match client.write_recovery(&params) {
            Ok(_) => println!("Saved mnemonic seed to disk"),
            Err(e) => report_error("Error writing mnemonic recovery seed to file", e),
        }
    }
    fn print_usage(&self, _params: &[&str]) {
        println!(
            "usage: account write {}\n\n{}\n",
            self.get_params_help(),
            self.get_description()
        );
    }
}

/// Sub command to list all accounts information.
pub struct AccountCommandListAccounts {}

impl Command for AccountCommandListAccounts {
    fn get_aliases(&self) -> Vec<&'static str> {
        vec!["list", "la"]
    }
    fn get_description(&self) -> &'static str {
        "Print all accounts that were created or loaded"
    }
    fn execute(&self, client: &mut ClientProxy, _params: &[&str]) {
        client.print_all_accounts();
    }
    fn print_usage(&self, _params: &[&str]) {
        println!("usage: account list \n\n{}\n", self.get_description());
    }
}

/// Sub command to mint account.
pub struct AccountCommandMint {}

impl Command for AccountCommandMint {
    fn get_aliases(&self) -> Vec<&'static str> {
        vec!["mint", "mintb", "m", "mb"]
    }
    fn get_params_help(&self) -> &'static str {
        "<receiver_account_ref_id>\n<receiver_account_address> <number_of_coins>"
    }
    fn get_description(&self) -> &'static str {
        "Mint coins to the account. Suffix 'b' is for blocking"
    }
    fn execute(&self, client: &mut ClientProxy, params: &[&str]) {
        if params.len() != 3 {
            println!("Invalid number of arguments for mint");
            return;
        }
        println!(">> Minting coins");
        let is_blocking = blocking_cmd(params[0]);
        match client.mint_coins(&params, is_blocking) {
            Ok(_) => {
                if is_blocking {
                    println!("Finished minting!");
                } else {
                    // If this value is updated, it must also be changed in
                    // setup_scripts/docker/mint/server.py
                    println!("Mint request submitted");
                }
            }
            Err(e) => report_error("Error minting coins", e),
        }
    }
    fn print_usage(&self, params: &[&str]) {
        if blocking_cmd(params[1]) {
            print_sub_command_usage(
                "account mintb",
                self.get_description(),
                self.get_params_help(),
            );
        } else {
            print_sub_command_usage(
                "account mint",
                self.get_description(),
                self.get_params_help(),
            );
        }
    }
}
