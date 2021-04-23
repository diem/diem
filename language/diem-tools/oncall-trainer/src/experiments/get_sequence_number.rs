// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::experiments::Experiment;
use anyhow::Result;
use cli::client_proxy::ClientProxy;
use diem_types::account_config::testnet_dd_account_address;

pub(crate) struct GetSequenceNumber();

impl Experiment for GetSequenceNumber {
    fn name(&self) -> &'static str {
        "get_sequence_number"
    }

    fn setup_states(&self, client: &mut ClientProxy) -> Result<()> {
        let account = client.create_next_account(false)?;
        for _ in 0..10 {
            client.mint_coins(
                &["mintb", format!("{}", account.index).as_str(), "10", "XUS"],
                true,
            )?;
        }
        Ok(())
    }
    fn description(&self) -> &'static str {
        "This tasks asks you to get the sequence number for the testnet_dd_account_address (0xDD) account. You could use either the cli tool to query the state of an account or use `transaction-replay` tool to annotate the account state. For educational purpose, I would highly recommend trying out the `transaction-replay` tool to see what's stored under a specific account.\n\nPlease input the sequence number for 0xDD account in the prompt!"
    }
    fn hint(&self) -> &'static str {
        "`transaction-replay` binary should have `annotate-account` mode. With this command you will be able to print out all resources stored under an account, including the DiemAccount resource, where you can find the sequence number, balance, etc of an arbitrary account."
    }

    fn check(&self, client: &mut ClientProxy, input: &str) -> Result<bool> {
        if let Ok(seq) = input.parse::<u64>() {
            let seq_expected = client
                .get_latest_account(&[
                    "as",
                    format!("{:?}", testnet_dd_account_address()).as_str(),
                ])?
                .unwrap()
                .sequence_number;

            Ok(seq_expected == seq)
        } else {
            println!("Expects integer as an input");
            Ok(false)
        }
    }
    fn reset_states(&self, _client: &mut ClientProxy) -> Result<()> {
        Ok(())
    }
}
