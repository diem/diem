// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::common::types::CliError;
use serde::Serialize;

#[cfg(unix)]

/// Convert result to JSON output
pub async fn to_common_result<T: Serialize>(result: Result<T, CliError>) -> Result<String, String> {
    let is_err = result.is_err();
    let result: ResultWrapper<T> = result.into();
    let string = serde_json::to_string_pretty(&result).unwrap();
    if is_err {
        Err(string)
    } else {
        Ok(string)
    }
}

#[derive(Debug, Serialize)]
enum ResultWrapper<T> {
    Result(T),
    Error(String),
}

impl<T> From<Result<T, CliError>> for ResultWrapper<T> {
    fn from(result: Result<T, CliError>) -> Self {
        match result {
            Ok(inner) => ResultWrapper::Result(inner),
            Err(inner) => ResultWrapper::Error(inner.to_string()),
        }
    }
}
