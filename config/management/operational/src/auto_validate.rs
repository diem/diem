// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{validate_transaction::ValidateTransaction, TransactionContext};
use libra_management::error::Error;
use std::{thread::sleep, time};
use structopt::StructOpt;

/// The default amount of time (in seconds) to allow for a transaction to be validated.
const TIMEOUT_SECS: u64 = 30;
/// The default sleep interval (in seconds) between validation checks.
const SLEEP_INTERVAL_SECS: u64 = 1;

#[derive(Clone, Debug, StructOpt)]
pub struct AutoValidate {
    #[structopt(long, help = "Disables auto validation")]
    disable_validate: bool,
    #[structopt(
        long,
        help = "The sleep duration between validation checks (in seconds)"
    )]
    sleep_interval: Option<u64>,
    #[structopt(long, help = "The timeout for automatic validation (in seconds)")]
    validate_timeout: Option<u64>,
}

impl AutoValidate {
    pub fn execute(
        &mut self,
        json_server: String,
        transaction_context: TransactionContext,
    ) -> Result<TransactionContext, Error> {
        // If validation is disabled return the transaction context unmodified
        if self.disable_validate {
            return Ok(transaction_context);
        }

        // Get the timeout to wait
        let validate_timeout = if let Some(validate_timeout) = self.validate_timeout {
            validate_timeout
        } else {
            TIMEOUT_SECS
        };

        // Get the sleep interval
        let sleep_interval = if let Some(sleep_interval) = self.sleep_interval {
            sleep_interval
        } else {
            SLEEP_INTERVAL_SECS
        };

        // Create the validate transaction command to run
        let validate_transaction = &ValidateTransaction::new(
            json_server,
            transaction_context.address,
            transaction_context.sequence_number,
        );

        // Loop until we get a successful result, or hit the timeout
        let mut time_slept = 0;
        while time_slept < validate_timeout {
            let validation_result = validate_transaction.execute()?;
            if validation_result.execution_result.is_some() {
                // The transaction was executed, return the context.
                return Ok(validation_result);
            }

            sleep(time::Duration::from_secs(sleep_interval));
            time_slept += sleep_interval;
        }

        // Tried to find the execution result, but the transaction still hasn't been executed...
        Ok(transaction_context)
    }
}
