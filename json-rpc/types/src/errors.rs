// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use diem_types::{
    mempool_status::{MempoolStatus, MempoolStatusCode},
    vm_status::{StatusCode, StatusType},
};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// list of server internal errors
pub static INTERNAL_ERRORS: &[i16; 7] = &[
    ServerCode::DefaultServerError as i16,
    ServerCode::VmInvariantViolationError as i16,
    ServerCode::VmExecutionError as i16,
    ServerCode::VmUnknownError as i16,
    ServerCode::MempoolIsFull as i16,
    ServerCode::MempoolTooManyTransactions as i16,
    ServerCode::MempoolUnknownError as i16,
];

pub fn is_internal_error(err_code: &i16) -> bool {
    for code in INTERNAL_ERRORS {
        if code == err_code {
            return true;
        }
    }
    false
}

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
#[repr(i16)]
#[derive(FromPrimitive)]
pub enum InvalidRequestCode {
    ParseError = -32700,
    InvalidRequest = -32600,
    MethodNotFound = -32601,
    InvalidParams = -32602,
    // -32603 is internal error
    InvalidFormat = -32604,
}

impl InvalidRequestCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            InvalidRequestCode::ParseError => "parse_error",
            InvalidRequestCode::InvalidRequest => "invalid_request",
            InvalidRequestCode::MethodNotFound => "method_not_found",
            InvalidRequestCode::InvalidParams => "invalid_params",
            InvalidRequestCode::InvalidFormat => "invalid_format",
        }
    }
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

impl From<bcs::Error> for JsonRpcError {
    fn from(err: bcs::Error) -> Self {
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

    pub fn invalid_params(method_name: &str) -> Self {
        Self {
            code: InvalidRequestCode::InvalidParams as i16,
            message: format!("Invalid params for method '{}'", method_name),
            data: None,
        }
    }

    pub fn invalid_params_size(msg: String) -> Self {
        Self {
            code: InvalidRequestCode::InvalidParams as i16,
            message: format!("Invalid params: {}", msg),
            data: None,
        }
    }

    pub fn invalid_param(msg: &str) -> Self {
        Self {
            code: InvalidRequestCode::InvalidParams as i16,
            message: format!("Invalid param {}", msg),
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

    pub fn code_as_str(&self) -> &'static str {
        match InvalidRequestCode::from_i16(self.code) {
            Some(code) => code.as_str(),
            None => "unexpected_code",
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

#[cfg(test)]
mod tests {
    use crate::errors::{is_internal_error, JsonRpcError, ServerCode, INTERNAL_ERRORS};
    use diem_types::{
        mempool_status::{MempoolStatus, MempoolStatusCode},
        vm_status::StatusCode,
    };

    #[test]
    fn test_vm_status() {
        assert_map_vm_code(
            StatusCode::UNKNOWN_VALIDATION_STATUS,
            ServerCode::VmValidationError,
        );

        let err = JsonRpcError::vm_status(StatusCode::UNKNOWN_VERIFICATION_ERROR);
        assert_eq!(err.code, ServerCode::VmVerificationError as i16);
        assert_map_vm_code(
            StatusCode::UNKNOWN_VERIFICATION_ERROR,
            ServerCode::VmVerificationError,
        );

        assert_map_vm_code(
            StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR,
            ServerCode::VmInvariantViolationError,
        );

        assert_map_vm_code(
            StatusCode::UNKNOWN_BINARY_ERROR,
            ServerCode::VmDeserializationError,
        );
        assert_map_vm_code(
            StatusCode::UNKNOWN_RUNTIME_STATUS,
            ServerCode::VmExecutionError,
        );
        assert_map_vm_code(StatusCode::UNKNOWN_STATUS, ServerCode::VmUnknownError);
    }

    fn assert_map_vm_code(from: StatusCode, to: ServerCode) {
        let err = JsonRpcError::vm_status(from);
        assert_eq!(err.code, to as i16);
    }

    #[test]
    fn test_mempool_error() {
        let err = JsonRpcError::mempool_error(MempoolStatus {
            code: MempoolStatusCode::Accepted,
            message: "error msg".to_string(),
        });
        assert!(err.is_err());

        assert_map_code(
            MempoolStatusCode::InvalidSeqNumber,
            ServerCode::MempoolInvalidSeqNumber,
        );
        assert_map_code(MempoolStatusCode::MempoolIsFull, ServerCode::MempoolIsFull);
        assert_map_code(
            MempoolStatusCode::TooManyTransactions,
            ServerCode::MempoolTooManyTransactions,
        );
        assert_map_code(
            MempoolStatusCode::InvalidUpdate,
            ServerCode::MempoolInvalidUpdate,
        );
        assert_map_code(MempoolStatusCode::VmError, ServerCode::MempoolVmError);
        assert_map_code(
            MempoolStatusCode::UnknownStatus,
            ServerCode::MempoolUnknownError,
        );
    }

    #[test]
    fn test_code_as_str() {
        let result = JsonRpcError::method_not_found().code_as_str();
        assert_eq!(result, "method_not_found")
    }

    #[test]
    fn test_is_internal_error() {
        for code in INTERNAL_ERRORS {
            assert!(is_internal_error(code));
        }
        assert!(!is_internal_error(
            &(ServerCode::MempoolInvalidUpdate as i16)
        ));
        assert!(!is_internal_error(&(ServerCode::VmValidationError as i16)));
    }

    fn assert_map_code(from: MempoolStatusCode, to: ServerCode) {
        let err = JsonRpcError::mempool_error(MempoolStatus {
            code: from,
            message: "error msg".to_string(),
        })
        .unwrap();
        assert_eq!(err.code, to as i16);
    }
}
