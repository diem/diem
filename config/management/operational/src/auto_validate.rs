// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{validate_transaction::ValidateTransaction, TransactionContext};
use diem_management::error::Error;
use std::{thread::sleep, time};
use structopt::StructOpt;

#[derive(Clone, Debug, StructOpt)]
pub struct AutoValidate {
    #[structopt(long, help = "Disables auto validation")]
    disable_validate: bool,
    #[structopt(
        long,
        help = "The sleep duration in seconds between validation checks",
        default_value = "1"
    )]
    sleep_interval: u64,
    #[structopt(
        long,
        help = "The timeout in seconds for automatic validation",
        default_value = "30"
    )]
    validate_timeout: u64,
}

impl AutoValidate {
    pub fn execute(
        &self,
        json_server: String,
        transaction_context: TransactionContext,
    ) -> Result<TransactionContext, Error> {
        // If validation is disabled return the transaction context unmodified
        if self.disable_validate {
            return Ok(transaction_context);
        }

        // Create the validate transaction command to run
        let validate_transaction = ValidateTransaction::new(
            json_server,
            transaction_context.address,
            transaction_context.sequence_number,
        );

        // Loop until we get a successful result, or hit the timeout
        let mut time_slept = 0;
        while time_slept < self.validate_timeout {
            let validation_result = validate_transaction.execute()?;
            if validation_result.execution_result.is_some() {
                // The transaction was executed, return the context.
                return Ok(validation_result);
            }

            sleep(time::Duration::from_secs(self.sleep_interval));
            time_slept = time_slept
                .checked_add(self.sleep_interval)
                .expect("Integer overflow/underflow detected: unexpected amount of time slept!");
        }

        // Tried to find the execution result, but the transaction still hasn't been executed...
        Ok(transaction_context)
    }
}
