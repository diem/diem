// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{CliResult, CliTypedResult};
use serde::Serialize;

#[cfg(unix)]

// Convert any successful response to Success
pub async fn to_common_success_result<T>(result: CliTypedResult<T>) -> CliResult {
    to_common_result(result.map(|_| "Success")).await
}

// For pretty printing outputs in JSON
pub async fn to_common_result<T: Serialize>(result: CliTypedResult<T>) -> CliResult {
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

impl<T> From<CliTypedResult<T>> for ResultWrapper<T> {
    fn from(result: CliTypedResult<T>) -> Self {
        match result {
            Ok(inner) => ResultWrapper::Result(inner),
            Err(inner) => ResultWrapper::Error(inner.to_string()),
        }
    }
}
