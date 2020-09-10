// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use libra_types::{
    mempool_status::{MempoolStatus, MempoolStatusCode},
    vm_status::{StatusCode, StatusType},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Custom JSON RPC server error codes
/// Ranges from -32000 to -32099 - see `https://www.jsonrpc.org/specification#error_object` for details
pub enum ServerCode {
    DefaultServerError = -32000,

    // VM errors - see `vm_status.rs` for specs
    VmValidationError = -32001,
    VmVerificationError = -32002,
    VmInvariantViolationError = -32003,
    VmDeserializationError = -32004,
    VmExecutionError = -32005,
    VmUnknownError = -32006,

    // Mempool errors - see `MempoolStatusCode` for specs
    MempoolInvalidSeqNumber = -32007,
    MempoolIsFull = -32008,
    MempoolTooManyTransactions = -32009,
    MempoolInvalidUpdate = -32010,
    MempoolVmError = -32011,
    MempoolUnknownError = -32012,
}

/// JSON RPC server error codes for invalid request
pub enum InvalidRequestCode {
    InvalidRequest = -32600,
    MethodNotFound = -32601,
    InvalidParams = -32602,
    // -32603 is internal error
    InvalidFormat = -32604,
    ParseError = -32700,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ErrorData {
    StatusCode(StatusCode),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JsonRpcError {
    pub code: i16,
    pub message: String,
    pub data: Option<ErrorData>,
}

impl std::error::Error for JsonRpcError {}

impl std::fmt::Display for JsonRpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<serde_json::error::Error> for JsonRpcError {
    fn from(err: serde_json::error::Error) -> Self {
        JsonRpcError::internal_error(err.to_string())
    }
}

impl From<anyhow::Error> for JsonRpcError {
    fn from(err: anyhow::Error) -> Self {
        JsonRpcError::internal_error(err.to_string())
    }
}

impl JsonRpcError {
    pub fn serialize(self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }

    pub fn invalid_request() -> Self {
        Self::invalid_request_with_data(Option::None)
    }

    pub fn invalid_request_with_data(data: Option<ErrorData>) -> Self {
        Self {
            code: InvalidRequestCode::InvalidRequest as i16,
            message: "Invalid Request".to_string(),
            data,
        }
    }

    pub fn invalid_request_with_msg(msg: String) -> Self {
        Self {
            code: InvalidRequestCode::InvalidRequest as i16,
            message: format!("Invalid Request: {}", msg),
            data: None,
        }
    }

    pub fn invalid_format() -> Self {
        Self {
            code: InvalidRequestCode::InvalidFormat as i16,
            message: "Invalid request format".to_string(),
            data: None,
        }
    }

    pub fn invalid_params(data: Option<ErrorData>) -> Self {
        Self {
            code: InvalidRequestCode::InvalidParams as i16,
            message: "Invalid params".to_string(),
            data,
        }
    }

    pub fn invalid_params_size(msg: String) -> Self {
        Self {
            code: InvalidRequestCode::InvalidParams as i16,
            message: format!("Invalid params: {}", msg),
            data: None,
        }
    }

    pub fn invalid_param(index: usize, name: &str, type_info: &str) -> Self {
        Self {
            code: InvalidRequestCode::InvalidParams as i16,
            message: format!(
                "Invalid param {}(params[{}]): should be {}",
                name, index, type_info
            ),
            data: None,
        }
    }

    pub fn method_not_found() -> Self {
        Self {
            code: InvalidRequestCode::MethodNotFound as i16,
            message: "Method not found".to_string(),
            data: None,
        }
    }

    pub fn internal_error(message: String) -> Self {
        Self {
            code: ServerCode::DefaultServerError as i16,
            message: format!("Server error: {}", message),
            data: None,
        }
    }

    pub fn mempool_error(error: MempoolStatus) -> Result<Self> {
        let code = match error.code {
            MempoolStatusCode::InvalidSeqNumber => ServerCode::MempoolInvalidSeqNumber,
            MempoolStatusCode::MempoolIsFull => ServerCode::MempoolIsFull,
            MempoolStatusCode::TooManyTransactions => ServerCode::MempoolTooManyTransactions,
            MempoolStatusCode::InvalidUpdate => ServerCode::MempoolInvalidUpdate,
            MempoolStatusCode::VmError => ServerCode::MempoolVmError,
            MempoolStatusCode::UnknownStatus => ServerCode::MempoolUnknownError,
            MempoolStatusCode::Accepted => {
                return Err(anyhow::format_err!(
                    "[JSON RPC] cannot create mempool error for mempool accepted status"
                ))
            }
        };

        Ok(Self {
            code: code as i16,
            message: format!(
                "Server error: Mempool submission error: {:?}",
                error.message
            ),
            data: None,
        })
    }

    pub fn vm_status(error: StatusCode) -> Self {
        // map VM status to custom server code
        let vm_status_type = error.status_type();
        let code = match vm_status_type {
            StatusType::Validation => ServerCode::VmValidationError,
            StatusType::Verification => ServerCode::VmVerificationError,
            StatusType::InvariantViolation => ServerCode::VmInvariantViolationError,
            StatusType::Deserialization => ServerCode::VmDeserializationError,
            StatusType::Execution => ServerCode::VmExecutionError,
            StatusType::Unknown => ServerCode::VmUnknownError,
        };

        Self {
            code: code as i16,
            message: format!("Server error: VM {} error: {:?}", vm_status_type, error),
            data: Some(ErrorData::StatusCode(error)),
        }
    }

    pub fn as_status_code(&self) -> Option<StatusCode> {
        if let Some(ErrorData::StatusCode(data)) = &self.data {
            return Some(*data);
        }
        None
    }
}
