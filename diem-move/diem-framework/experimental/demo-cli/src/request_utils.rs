// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, format_err, Result};

use serde::{Deserialize, Serialize};
use std::str::FromStr;

const DIEM_ACCOUNT_RESOURCE: &str = "0x1::DiemAccount::DiemAccount";

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct AccountResource {
    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub type_: String,
    pub value: serde_json::Value,
}

pub async fn get_request(base_uri: &str, path: String) -> Result<reqwest::Response> {
    reqwest::Client::new()
        .get(format!("{}/{}", base_uri, path.as_str()).as_str())
        .send()
        .await
        .map_err(|e| format_err!("{}", e))
}

/// Account address is a hex encoded address, e.g., 0xC0FFEE
/// Resource type is a full path to the resource, e.g., "0x1::DiemAccount::DiemAccount"
/// The returned value is a key-value map with the json serialization of the object.
pub async fn get_account_resource_by_type(
    base_uri: &str,
    account_address: String,
    resource_type: &str,
) -> Result<serde_json::Value> {
    let acc_resources = get_request(base_uri, format!("accounts/{}/resources", account_address))
        .await?
        .json::<Vec<AccountResource>>()
        .await
        .map_err(|e| format_err!("{}", e))?;
    for resource_entry in acc_resources {
        if resource_entry.type_ == resource_type {
            return Ok(resource_entry.value);
        }
    }
    bail!("Resource {} not found", resource_type);
}

pub async fn get_account_seq_num(base_uri: &str, account_address: String) -> Result<u64> {
    let diem_account_resource =
        get_account_resource_by_type(base_uri, account_address, DIEM_ACCOUNT_RESOURCE).await?;
    for (k, v) in diem_account_resource
        .as_object()
        .ok_or_else(|| format_err!("Fail to parse resource {}", DIEM_ACCOUNT_RESOURCE))?
        .iter()
    {
        if k == "sequence_number" {
            return Ok(u64::from_str(v.as_str().unwrap()).unwrap());
        }
    }
    bail!("Resource sequence_number not found");
}
