// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    client_proxy::ClientProxy,
    commands::{blocking_cmd, report_error, Command},
};

/// Command to transfer coins between two accounts.
pub struct TransferCommand {}

impl Command for TransferCommand {
    fn get_aliases(&self) -> Vec<&'static str> {
        vec!["transfer", "transferb", "t", "tb"]
    }
    fn get_params_help(&self) -> &'static str {
        "\n\t<sender_account_address>|<sender_account_ref_id> \
         <receiver_account_address>|<receiver_account_ref_id> <number_of_coins> \
         [gas_unit_price_in_micro_libras (default=0)] [max_gas_amount_in_micro_libras (default 140000)] \
         Suffix 'b' is for blocking. "
    }
    fn get_description(&self) -> &'static str {
        "Transfer coins (in libra) from account to another."
    }
    fn execute(&self, client: &mut ClientProxy, params: &[&str]) {
        if params.len() < 4 || params.len() > 6 {
            println!("Invalid number of arguments for transfer");
            println!(
                "{} {}",
                self.get_aliases().join(" | "),
                self.get_params_help()
            );
            return;
        }

        println!(">> Transferring");
        let is_blocking = blocking_cmd(&params[0]);
        match client.transfer_coins(&params, is_blocking) {
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
