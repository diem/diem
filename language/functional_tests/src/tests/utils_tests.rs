// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::global::Config as GlobalConfig,
    errors::*,
    utils::{build_transactions, split_input},
};

fn parse_input(input: &str) -> Result<()> {
    let (config, _, transactions) = split_input(&input)?;
    let config = GlobalConfig::build(&config)?;
    build_transactions(&config, &transactions)?;
    Ok(())
}

#[test]
fn parse_input_no_transactions() {
    parse_input("").unwrap_err();
}

#[test]
fn parse_input_no_transactions_with_config() {
    parse_input("//! no-run: verifier").unwrap_err();
}

#[rustfmt::skip]
#[test]
fn parse_input_nothing_before_first_empty_transaction() {
    parse_input(r"
        //! new-transaction
        main() {}
    ").unwrap();
}

#[rustfmt::skip]
#[test]
fn parse_input_config_before_first_empty_transaction() {
    parse_input(r"
        //! no-run: runtime
        //! new-transaction
        main() {}
    ").unwrap_err();
}

#[rustfmt::skip]
#[test]
fn parse_input_empty_transaction() {
    parse_input(r"
        main() {}

        //! new-transaction

        //! new-transaction
        main() {}
    ").unwrap_err();
}

#[rustfmt::skip]
#[test]
fn parse_input_empty_transaction_with_config() {
    parse_input(r"
        main() {}

        //! new-transaction
        //! sender: default

        //! new-transaction
        main() {}
    ").unwrap_err();
}
