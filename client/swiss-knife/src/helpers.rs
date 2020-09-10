// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_types::{
    account_address::AccountAddress,
    account_config::{from_currency_code_string, type_tag_for_currency_code},
};
use move_core_types::language_storage::TypeTag;
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;
use std::{fmt::Display, io::Read};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Response {
    pub error_message: String,
    pub data: serde_json::Value,
}

pub fn exit_success_with_data<T: serde::Serialize>(data: T) {
    let data = serde_json::to_value(data)
        .map_err(|err| exit_with_error(format!("json serialization failure : {}", err)))
        .unwrap();
    let response = Response {
        error_message: "".into(),
        data,
    };
    print!(
        "{}",
        to_string_pretty(&response)
            .map_err(|err| exit_with_error(format!("json serialization failure : {}", err)))
            .unwrap()
    );
    std::process::exit(0);
}

pub fn exit_with_error<T: Display>(error_message: T) {
    let response = Response {
        error_message: format!("{}", error_message),
        data: "".into(),
    };
    print!(
        "{}",
        to_string_pretty(&response)
            .map_err(|err| println!("json serialization failure : {}", err))
            .unwrap()
    );
    std::process::exit(1);
}

pub fn hex_decode(data: &str) -> Vec<u8> {
    hex::decode(data)
        .map_err(|err| exit_with_error(format!("Failed to decode hex data {} : {}", data, err)))
        .unwrap()
}

pub fn read_stdin() -> String {
    let mut buffer = String::new();
    std::io::stdin()
        .read_to_string(&mut buffer)
        .map_err(|err| exit_with_error(format!("Failed to read from stdin : {}", err)))
        .unwrap();
    buffer
}

pub fn coin_tag_parser(coin_tag: &str) -> TypeTag {
    type_tag_for_currency_code(
        from_currency_code_string(&coin_tag)
            .map_err(|err| {
                exit_with_error(format!("Failed to parse coin_tag {} : {}", coin_tag, err))
            })
            .unwrap(),
    )
}

pub fn account_address_parser(address: &str) -> AccountAddress {
    AccountAddress::from_hex_literal(address)
        .map_err(|err| {
            exit_with_error(format!(
                "Failed to parse address as an AccountAddress {} : {}",
                address, err
            ))
        })
        .unwrap()
}
