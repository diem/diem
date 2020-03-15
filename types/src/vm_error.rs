// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![allow(clippy::unit_arg)]

use anyhow::{Error, Result};
#[cfg(any(test, feature = "fuzzing"))]
use proptest::prelude::*;
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{de, ser, Deserialize, Serialize};
use std::{convert::TryFrom, fmt};

/// The minimum status code for validation statuses
pub static VALIDATION_STATUS_MIN_CODE: u64 = 0;

/// The maximum status code for validation statuses
pub static VALIDATION_STATUS_MAX_CODE: u64 = 999;

/// The minimum status code for verification statuses
pub static VERIFICATION_STATUS_MIN_CODE: u64 = 1000;

/// The maximum status code for verification statuses
pub static VERIFICATION_STATUS_MAX_CODE: u64 = 1999;

/// The minimum status code for invariant violation statuses
pub static INVARIANT_VIOLATION_STATUS_MIN_CODE: u64 = 2000;

/// The maximum status code for invariant violation statuses
pub static INVARIANT_VIOLATION_STATUS_MAX_CODE: u64 = 2999;

/// The minimum status code for deserialization statuses
pub static DESERIALIZATION_STATUS_MIN_CODE: u64 = 3000;

/// The maximum status code for deserialization statuses
pub static DESERIALIZATION_STATUS_MAX_CODE: u64 = 3999;

/// The minimum status code for runtime statuses
pub static EXECUTION_STATUS_MIN_CODE: u64 = 4000;

/// The maximum status code for runtim statuses
pub static EXECUTION_STATUS_MAX_CODE: u64 = 4999;

/// A `VMStatus` is represented as a required major status that is semantic coupled with with
/// an optional sub status and message.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Serialize, Deserialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
#[cfg_attr(any(test, feature = "fuzzing"), proptest(no_params))]
pub struct VMStatus {
    /// The major status, e.g. ABORTED, OUT_OF_GAS, etc.
    pub major_status: StatusCode,

    /// The optional sub status. Used e.g. for things such as the abort code, or the arithmetic
    /// error type for an ARITHMETIC_ERROR major status.
    pub sub_status: Option<u64>,

    /// The optional message. Useful for verification errors, and for returning information in
    /// validation.
    pub message: Option<String>,
}

/// A status type is one of 5 different variants, along with a fallback variant in the case that we
/// don't recognize the status code.
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum StatusType {
    Validation,
    Verification,
    InvariantViolation,
    Deserialization,
    Execution,
    Unknown,
}

impl fmt::Display for StatusType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = match self {
            StatusType::Validation => "Validation",
            StatusType::Verification => "Verification",
            StatusType::InvariantViolation => "Invariant violation",
            StatusType::Deserialization => "Deserialization",
            StatusType::Execution => "Execution",
            StatusType::Unknown => "Unknown",
        };
        write!(f, "{}", string)
    }
}

impl fmt::Display for VMStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status_type = self.status_type();
        let mut status = format!("status {:#?} of type {}", self.major_status, status_type);

        if let Some(sub_status) = self.sub_status {
            status = format!("{} with sub status {}", status, sub_status);
        }

        if let Some(ref msg) = self.message {
            status = format!("{} and message {}", status, msg);
        }

        write!(f, "{}", status)
    }
}

impl std::error::Error for VMStatus {}

impl VMStatus {
    /// Create a new VM status with major status `major_status`.
    pub fn new(major_status: StatusCode) -> Self {
        Self {
            major_status,
            sub_status: None,
            message: None,
        }
    }

    /// Adds a sub status to the VM status.
    pub fn with_sub_status(mut self, sub_status: u64) -> Self {
        self.sub_status = Some(sub_status);
        self
    }

    /// Adds a message to the VM status.
    pub fn with_message(mut self, message: String) -> Self {
        self.message = Some(message);
        self
    }

    /// Mutates the VMStatus sub status field to be the new `sub_status` passed in.
    pub fn set_sub_status(&mut self, sub_status: u64) {
        self.sub_status = Some(sub_status);
    }

    /// Mutates the VMStatus message field to be the new `message` passed in.
    pub fn set_message(&mut self, message: String) {
        self.message = Some(message);
    }
    /// Append the message `message` to the message field of the VM status, and insert a seperator
    /// if the original message is non-empty.
    pub fn append_message_with_separator(mut self, separator: char, message: String) -> Self {
        if let Some(ref mut msg) = self.message {
            if !msg.is_empty() {
                msg.push(separator);
            }
            msg.push_str(&message);
        } else {
            self.message = Some(message);
        }
        self
    }

    /// Return the status type for this VMStatus. This is solely determined by the `major_status`
    /// field.
    pub fn status_type(&self) -> StatusType {
        let major_status_number: u64 = self.major_status.into();
        if major_status_number >= VALIDATION_STATUS_MIN_CODE
            && major_status_number <= VALIDATION_STATUS_MAX_CODE
        {
            return StatusType::Validation;
        }

        if major_status_number >= VERIFICATION_STATUS_MIN_CODE
            && major_status_number <= VERIFICATION_STATUS_MAX_CODE
        {
            return StatusType::Verification;
        }

        if major_status_number >= INVARIANT_VIOLATION_STATUS_MIN_CODE
            && major_status_number <= INVARIANT_VIOLATION_STATUS_MAX_CODE
        {
            return StatusType::InvariantViolation;
        }

        if major_status_number >= DESERIALIZATION_STATUS_MIN_CODE
            && major_status_number <= DESERIALIZATION_STATUS_MAX_CODE
        {
            return StatusType::Deserialization;
        }

        if major_status_number >= EXECUTION_STATUS_MIN_CODE
            && major_status_number <= EXECUTION_STATUS_MAX_CODE
        {
            return StatusType::Execution;
        }

        StatusType::Unknown
    }

    /// Determine if the VMStatus has status type `status_type`.
    pub fn is(&self, status_type: StatusType) -> bool {
        self.status_type() == status_type
    }

    /// Append two VMStatuses together. The major status is kept from the caller.
    pub fn append(self, other: Self) -> Self {
        let msg = format!("{}", other);
        self.append_message_with_separator('\n', msg)
    }
}

//***********************************
// Decoding/Encoding to Protobuffers
//***********************************
impl TryFrom<crate::proto::types::VmStatus> for VMStatus {
    type Error = Error;

    fn try_from(proto: crate::proto::types::VmStatus) -> Result<Self> {
        let mut status = VMStatus::new(
            StatusCode::try_from(proto.major_status).unwrap_or(StatusCode::UNKNOWN_STATUS),
        );

        if proto.has_sub_status {
            status.set_sub_status(proto.sub_status);
        }

        if proto.has_message {
            status.set_message(proto.message);
        }

        Ok(status)
    }
}

impl From<VMStatus> for crate::proto::types::VmStatus {
    fn from(status: VMStatus) -> Self {
        let mut proto_status = Self::default();

        proto_status.has_sub_status = false;
        proto_status.has_message = false;

        // Set major status
        proto_status.major_status = status.major_status.into();

        // Set minor status if there is one
        if let Some(sub_status) = status.sub_status {
            proto_status.has_sub_status = true;
            proto_status.sub_status = sub_status;
        }

        // Set info string
        if let Some(string) = status.message {
            proto_status.has_message = true;
            proto_status.message = string;
        }

        proto_status
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
#[repr(u64)]
/// We don't derive Arbitrary on this enum because it is too large and breaks proptest. It is
/// written for a subset of these in proptest_types. We test conversion between this and protobuf
/// with a hand-written test.
pub enum StatusCode {
    // The status of a transaction as determined by the prologue.
    // Validation Errors: 0-999
    // We don't want the default value to be valid
    UNKNOWN_VALIDATION_STATUS = 0,
    // The transaction has a bad signature
    INVALID_SIGNATURE = 1,
    // Bad account authentication key
    INVALID_AUTH_KEY = 2,
    // Sequence number is too old
    SEQUENCE_NUMBER_TOO_OLD = 3,
    // Sequence number is too new
    SEQUENCE_NUMBER_TOO_NEW = 4,
    // Insufficient balance to pay minimum transaction fee
    INSUFFICIENT_BALANCE_FOR_TRANSACTION_FEE = 5,
    // The transaction has expired
    TRANSACTION_EXPIRED = 6,
    // The sending account does not exist
    SENDING_ACCOUNT_DOES_NOT_EXIST = 7,
    // This write set transaction was rejected because it did not meet the
    // requirements for one.
    REJECTED_WRITE_SET = 8,
    // This write set transaction cannot be applied to the current state.
    INVALID_WRITE_SET = 9,
    // Length of program field in raw transaction exceeded max length
    EXCEEDED_MAX_TRANSACTION_SIZE = 10,
    // This script is not on our whitelist of script.
    UNKNOWN_SCRIPT = 11,
    // Transaction is trying to publish a new module.
    UNKNOWN_MODULE = 12,
    // Max gas units submitted with transaction exceeds max gas units bound
    // in VM
    MAX_GAS_UNITS_EXCEEDS_MAX_GAS_UNITS_BOUND = 13,
    // Max gas units submitted with transaction not enough to cover the
    // intrinsic cost of the transaction.
    MAX_GAS_UNITS_BELOW_MIN_TRANSACTION_GAS_UNITS = 14,
    // Gas unit price submitted with transaction is below minimum gas price
    // set in the VM.
    GAS_UNIT_PRICE_BELOW_MIN_BOUND = 15,
    // Gas unit price submitted with the transaction is above the maximum
    // gas price set in the VM.
    GAS_UNIT_PRICE_ABOVE_MAX_BOUND = 16,

    // When a code module/script is published it is verified. These are the
    // possible errors that can arise from the verification process.
    // Verification Errors: 1000-1999
    UNKNOWN_VERIFICATION_ERROR = 1000,
    INDEX_OUT_OF_BOUNDS = 1001,
    RANGE_OUT_OF_BOUNDS = 1002,
    INVALID_SIGNATURE_TOKEN = 1003,
    INVALID_FIELD_DEF = 1004,
    RECURSIVE_STRUCT_DEFINITION = 1005,
    INVALID_RESOURCE_FIELD = 1006,
    INVALID_FALL_THROUGH = 1007,
    JOIN_FAILURE = 1008,
    NEGATIVE_STACK_SIZE_WITHIN_BLOCK = 1009,
    UNBALANCED_STACK = 1010,
    INVALID_MAIN_FUNCTION_SIGNATURE = 1011,
    DUPLICATE_ELEMENT = 1012,
    INVALID_MODULE_HANDLE = 1013,
    UNIMPLEMENTED_HANDLE = 1014,
    INCONSISTENT_FIELDS = 1015,
    UNUSED_FIELD = 1016,
    LOOKUP_FAILED = 1017,
    VISIBILITY_MISMATCH = 1018,
    TYPE_RESOLUTION_FAILURE = 1019,
    TYPE_MISMATCH = 1020,
    MISSING_DEPENDENCY = 1021,
    POP_REFERENCE_ERROR = 1022,
    POP_RESOURCE_ERROR = 1023,
    RELEASEREF_TYPE_MISMATCH_ERROR = 1024,
    BR_TYPE_MISMATCH_ERROR = 1025,
    ABORT_TYPE_MISMATCH_ERROR = 1026,
    STLOC_TYPE_MISMATCH_ERROR = 1027,
    STLOC_UNSAFE_TO_DESTROY_ERROR = 1028,
    RET_UNSAFE_TO_DESTROY_ERROR = 1029,
    RET_TYPE_MISMATCH_ERROR = 1030,
    RET_BORROWED_MUTABLE_REFERENCE_ERROR = 1031,
    FREEZEREF_TYPE_MISMATCH_ERROR = 1032,
    FREEZEREF_EXISTS_MUTABLE_BORROW_ERROR = 1033,
    BORROWFIELD_TYPE_MISMATCH_ERROR = 1034,
    BORROWFIELD_BAD_FIELD_ERROR = 1035,
    BORROWFIELD_EXISTS_MUTABLE_BORROW_ERROR = 1036,
    COPYLOC_UNAVAILABLE_ERROR = 1037,
    COPYLOC_RESOURCE_ERROR = 1038,
    COPYLOC_EXISTS_BORROW_ERROR = 1039,
    MOVELOC_UNAVAILABLE_ERROR = 1040,
    MOVELOC_EXISTS_BORROW_ERROR = 1041,
    BORROWLOC_REFERENCE_ERROR = 1042,
    BORROWLOC_UNAVAILABLE_ERROR = 1043,
    BORROWLOC_EXISTS_BORROW_ERROR = 1044,
    CALL_TYPE_MISMATCH_ERROR = 1045,
    CALL_BORROWED_MUTABLE_REFERENCE_ERROR = 1046,
    PACK_TYPE_MISMATCH_ERROR = 1047,
    UNPACK_TYPE_MISMATCH_ERROR = 1048,
    READREF_TYPE_MISMATCH_ERROR = 1049,
    READREF_RESOURCE_ERROR = 1050,
    READREF_EXISTS_MUTABLE_BORROW_ERROR = 1051,
    WRITEREF_TYPE_MISMATCH_ERROR = 1052,
    WRITEREF_RESOURCE_ERROR = 1053,
    WRITEREF_EXISTS_BORROW_ERROR = 1054,
    WRITEREF_NO_MUTABLE_REFERENCE_ERROR = 1055,
    INTEGER_OP_TYPE_MISMATCH_ERROR = 1056,
    BOOLEAN_OP_TYPE_MISMATCH_ERROR = 1057,
    EQUALITY_OP_TYPE_MISMATCH_ERROR = 1058,
    EXISTS_RESOURCE_TYPE_MISMATCH_ERROR = 1059,
    BORROWGLOBAL_TYPE_MISMATCH_ERROR = 1060,
    BORROWGLOBAL_NO_RESOURCE_ERROR = 1061,
    MOVEFROM_TYPE_MISMATCH_ERROR = 1062,
    MOVEFROM_NO_RESOURCE_ERROR = 1063,
    MOVETOSENDER_TYPE_MISMATCH_ERROR = 1064,
    MOVETOSENDER_NO_RESOURCE_ERROR = 1065,
    CREATEACCOUNT_TYPE_MISMATCH_ERROR = 1066,
    // The self address of a module the transaction is publishing is not the sender address
    MODULE_ADDRESS_DOES_NOT_MATCH_SENDER = 1067,
    // The module does not have any module handles. Each module or script must have at least one
    // module handle.
    NO_MODULE_HANDLES = 1068,
    POSITIVE_STACK_SIZE_AT_BLOCK_END = 1069,
    MISSING_ACQUIRES_RESOURCE_ANNOTATION_ERROR = 1070,
    EXTRANEOUS_ACQUIRES_RESOURCE_ANNOTATION_ERROR = 1071,
    DUPLICATE_ACQUIRES_RESOURCE_ANNOTATION_ERROR = 1072,
    INVALID_ACQUIRES_RESOURCE_ANNOTATION_ERROR = 1073,
    GLOBAL_REFERENCE_ERROR = 1074,
    CONTRAINT_KIND_MISMATCH = 1075,
    NUMBER_OF_TYPE_ACTUALS_MISMATCH = 1076,
    LOOP_IN_INSTANTIATION_GRAPH = 1077,
    UNUSED_LOCALS_SIGNATURE = 1078,
    UNUSED_TYPE_SIGNATURE = 1079,
    /// Reported when a struct has zero fields
    ZERO_SIZED_STRUCT = 1080,
    LINKER_ERROR = 1081,

    // These are errors that the VM might raise if a violation of internal
    // invariants takes place.
    // Invariant Violation Errors: 2000-2999
    UNKNOWN_INVARIANT_VIOLATION_ERROR = 2000,
    OUT_OF_BOUNDS_INDEX = 2001,
    OUT_OF_BOUNDS_RANGE = 2002,
    EMPTY_VALUE_STACK = 2003,
    EMPTY_CALL_STACK = 2004,
    PC_OVERFLOW = 2005,
    VERIFICATION_ERROR = 2006,
    LOCAL_REFERENCE_ERROR = 2007,
    STORAGE_ERROR = 2008,
    INTERNAL_TYPE_ERROR = 2009,
    EVENT_KEY_MISMATCH = 2010,
    UNREACHABLE = 2011,
    VM_STARTUP_FAILURE = 2012,
    NATIVE_FUNCTION_INTERNAL_INCONSISTENCY = 2013,

    // Errors that can arise from binary decoding (deserialization)
    // Deserializtion Errors: 3000-3999
    UNKNOWN_BINARY_ERROR = 3000,
    MALFORMED = 3001,
    BAD_MAGIC = 3002,
    UNKNOWN_VERSION = 3003,
    UNKNOWN_TABLE_TYPE = 3004,
    UNKNOWN_SIGNATURE_TYPE = 3005,
    UNKNOWN_SERIALIZED_TYPE = 3006,
    UNKNOWN_OPCODE = 3007,
    BAD_HEADER_TABLE = 3008,
    UNEXPECTED_SIGNATURE_TYPE = 3009,
    DUPLICATE_TABLE = 3010,
    VERIFIER_INVARIANT_VIOLATION = 3011,

    // Errors that can arise at runtime
    // Runtime Errors: 4000-4999
    UNKNOWN_RUNTIME_STATUS = 4000,
    EXECUTED = 4001,
    OUT_OF_GAS = 4002,
    // We tried to access a resource that does not exist under the account.
    RESOURCE_DOES_NOT_EXIST = 4003,
    // We tried to create a resource under an account where that resource
    // already exists.
    RESOURCE_ALREADY_EXISTS = 4004,
    // We accessed an account that is evicted.
    EVICTED_ACCOUNT_ACCESS = 4005,
    // We tried to create an account at an address where an account already exists.
    ACCOUNT_ADDRESS_ALREADY_EXISTS = 4006,
    TYPE_ERROR = 4007,
    MISSING_DATA = 4008,
    DATA_FORMAT_ERROR = 4009,
    INVALID_DATA = 4010,
    REMOTE_DATA_ERROR = 4011,
    CANNOT_WRITE_EXISTING_RESOURCE = 4012,
    VALUE_SERIALIZATION_ERROR = 4013,
    VALUE_DESERIALIZATION_ERROR = 4014,
    // The sender is trying to publish a module named `M`, but the sender's account already
    // contains a module with this name.
    DUPLICATE_MODULE_NAME = 4015,
    ABORTED = 4016,
    ARITHMETIC_ERROR = 4017,
    DYNAMIC_REFERENCE_ERROR = 4018,
    CODE_DESERIALIZATION_ERROR = 4019,
    EXECUTION_STACK_OVERFLOW = 4020,
    CALL_STACK_OVERFLOW = 4021,
    NATIVE_FUNCTION_ERROR = 4022,
    GAS_SCHEDULE_ERROR = 4023,
    CREATE_NULL_ACCOUNT = 4024,

    // A reserved status to represent an unknown vm status.
    UNKNOWN_STATUS = std::u64::MAX,
}

// TODO(#1307)
impl ser::Serialize for StatusCode {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_u64((*self).into())
    }
}

impl<'de> de::Deserialize<'de> for StatusCode {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct StatusCodeVisitor;
        impl<'de> de::Visitor<'de> for StatusCodeVisitor {
            type Value = StatusCode;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("StatusCode as u64")
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<StatusCode, E>
            where
                E: de::Error,
            {
                Ok(StatusCode::try_from(v).unwrap_or(StatusCode::UNKNOWN_STATUS))
            }
        }

        deserializer.deserialize_u64(StatusCodeVisitor)
    }
}

impl TryFrom<u64> for StatusCode {
    type Error = &'static str;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(StatusCode::UNKNOWN_VALIDATION_STATUS),
            1 => Ok(StatusCode::INVALID_SIGNATURE),
            2 => Ok(StatusCode::INVALID_AUTH_KEY),
            3 => Ok(StatusCode::SEQUENCE_NUMBER_TOO_OLD),
            4 => Ok(StatusCode::SEQUENCE_NUMBER_TOO_NEW),
            5 => Ok(StatusCode::INSUFFICIENT_BALANCE_FOR_TRANSACTION_FEE),
            6 => Ok(StatusCode::TRANSACTION_EXPIRED),
            7 => Ok(StatusCode::SENDING_ACCOUNT_DOES_NOT_EXIST),
            8 => Ok(StatusCode::REJECTED_WRITE_SET),
            9 => Ok(StatusCode::INVALID_WRITE_SET),
            10 => Ok(StatusCode::EXCEEDED_MAX_TRANSACTION_SIZE),
            11 => Ok(StatusCode::UNKNOWN_SCRIPT),
            12 => Ok(StatusCode::UNKNOWN_MODULE),
            13 => Ok(StatusCode::MAX_GAS_UNITS_EXCEEDS_MAX_GAS_UNITS_BOUND),
            14 => Ok(StatusCode::MAX_GAS_UNITS_BELOW_MIN_TRANSACTION_GAS_UNITS),
            15 => Ok(StatusCode::GAS_UNIT_PRICE_BELOW_MIN_BOUND),
            16 => Ok(StatusCode::GAS_UNIT_PRICE_ABOVE_MAX_BOUND),
            1000 => Ok(StatusCode::UNKNOWN_VERIFICATION_ERROR),
            1001 => Ok(StatusCode::INDEX_OUT_OF_BOUNDS),
            1002 => Ok(StatusCode::RANGE_OUT_OF_BOUNDS),
            1003 => Ok(StatusCode::INVALID_SIGNATURE_TOKEN),
            1004 => Ok(StatusCode::INVALID_FIELD_DEF),
            1005 => Ok(StatusCode::RECURSIVE_STRUCT_DEFINITION),
            1006 => Ok(StatusCode::INVALID_RESOURCE_FIELD),
            1007 => Ok(StatusCode::INVALID_FALL_THROUGH),
            1008 => Ok(StatusCode::JOIN_FAILURE),
            1009 => Ok(StatusCode::NEGATIVE_STACK_SIZE_WITHIN_BLOCK),
            1010 => Ok(StatusCode::UNBALANCED_STACK),
            1011 => Ok(StatusCode::INVALID_MAIN_FUNCTION_SIGNATURE),
            1012 => Ok(StatusCode::DUPLICATE_ELEMENT),
            1013 => Ok(StatusCode::INVALID_MODULE_HANDLE),
            1014 => Ok(StatusCode::UNIMPLEMENTED_HANDLE),
            1015 => Ok(StatusCode::INCONSISTENT_FIELDS),
            1016 => Ok(StatusCode::UNUSED_FIELD),
            1017 => Ok(StatusCode::LOOKUP_FAILED),
            1018 => Ok(StatusCode::VISIBILITY_MISMATCH),
            1019 => Ok(StatusCode::TYPE_RESOLUTION_FAILURE),
            1020 => Ok(StatusCode::TYPE_MISMATCH),
            1021 => Ok(StatusCode::MISSING_DEPENDENCY),
            1022 => Ok(StatusCode::POP_REFERENCE_ERROR),
            1023 => Ok(StatusCode::POP_RESOURCE_ERROR),
            1024 => Ok(StatusCode::RELEASEREF_TYPE_MISMATCH_ERROR),
            1025 => Ok(StatusCode::BR_TYPE_MISMATCH_ERROR),
            1026 => Ok(StatusCode::ABORT_TYPE_MISMATCH_ERROR),
            1027 => Ok(StatusCode::STLOC_TYPE_MISMATCH_ERROR),
            1028 => Ok(StatusCode::STLOC_UNSAFE_TO_DESTROY_ERROR),
            1029 => Ok(StatusCode::RET_UNSAFE_TO_DESTROY_ERROR),
            1030 => Ok(StatusCode::RET_TYPE_MISMATCH_ERROR),
            1031 => Ok(StatusCode::RET_BORROWED_MUTABLE_REFERENCE_ERROR),
            1032 => Ok(StatusCode::FREEZEREF_TYPE_MISMATCH_ERROR),
            1033 => Ok(StatusCode::FREEZEREF_EXISTS_MUTABLE_BORROW_ERROR),
            1034 => Ok(StatusCode::BORROWFIELD_TYPE_MISMATCH_ERROR),
            1035 => Ok(StatusCode::BORROWFIELD_BAD_FIELD_ERROR),
            1036 => Ok(StatusCode::BORROWFIELD_EXISTS_MUTABLE_BORROW_ERROR),
            1037 => Ok(StatusCode::COPYLOC_UNAVAILABLE_ERROR),
            1038 => Ok(StatusCode::COPYLOC_RESOURCE_ERROR),
            1039 => Ok(StatusCode::COPYLOC_EXISTS_BORROW_ERROR),
            1040 => Ok(StatusCode::MOVELOC_UNAVAILABLE_ERROR),
            1041 => Ok(StatusCode::MOVELOC_EXISTS_BORROW_ERROR),
            1042 => Ok(StatusCode::BORROWLOC_REFERENCE_ERROR),
            1043 => Ok(StatusCode::BORROWLOC_UNAVAILABLE_ERROR),
            1044 => Ok(StatusCode::BORROWLOC_EXISTS_BORROW_ERROR),
            1045 => Ok(StatusCode::CALL_TYPE_MISMATCH_ERROR),
            1046 => Ok(StatusCode::CALL_BORROWED_MUTABLE_REFERENCE_ERROR),
            1047 => Ok(StatusCode::PACK_TYPE_MISMATCH_ERROR),
            1048 => Ok(StatusCode::UNPACK_TYPE_MISMATCH_ERROR),
            1049 => Ok(StatusCode::READREF_TYPE_MISMATCH_ERROR),
            1050 => Ok(StatusCode::READREF_RESOURCE_ERROR),
            1051 => Ok(StatusCode::READREF_EXISTS_MUTABLE_BORROW_ERROR),
            1052 => Ok(StatusCode::WRITEREF_TYPE_MISMATCH_ERROR),
            1053 => Ok(StatusCode::WRITEREF_RESOURCE_ERROR),
            1054 => Ok(StatusCode::WRITEREF_EXISTS_BORROW_ERROR),
            1055 => Ok(StatusCode::WRITEREF_NO_MUTABLE_REFERENCE_ERROR),
            1056 => Ok(StatusCode::INTEGER_OP_TYPE_MISMATCH_ERROR),
            1057 => Ok(StatusCode::BOOLEAN_OP_TYPE_MISMATCH_ERROR),
            1058 => Ok(StatusCode::EQUALITY_OP_TYPE_MISMATCH_ERROR),
            1059 => Ok(StatusCode::EXISTS_RESOURCE_TYPE_MISMATCH_ERROR),
            1060 => Ok(StatusCode::BORROWGLOBAL_TYPE_MISMATCH_ERROR),
            1061 => Ok(StatusCode::BORROWGLOBAL_NO_RESOURCE_ERROR),
            1062 => Ok(StatusCode::MOVEFROM_TYPE_MISMATCH_ERROR),
            1063 => Ok(StatusCode::MOVEFROM_NO_RESOURCE_ERROR),
            1064 => Ok(StatusCode::MOVETOSENDER_TYPE_MISMATCH_ERROR),
            1065 => Ok(StatusCode::MOVETOSENDER_NO_RESOURCE_ERROR),
            1066 => Ok(StatusCode::CREATEACCOUNT_TYPE_MISMATCH_ERROR),
            1067 => Ok(StatusCode::MODULE_ADDRESS_DOES_NOT_MATCH_SENDER),
            1068 => Ok(StatusCode::NO_MODULE_HANDLES),
            1069 => Ok(StatusCode::POSITIVE_STACK_SIZE_AT_BLOCK_END),
            1070 => Ok(StatusCode::MISSING_ACQUIRES_RESOURCE_ANNOTATION_ERROR),
            1071 => Ok(StatusCode::EXTRANEOUS_ACQUIRES_RESOURCE_ANNOTATION_ERROR),
            1072 => Ok(StatusCode::DUPLICATE_ACQUIRES_RESOURCE_ANNOTATION_ERROR),
            1073 => Ok(StatusCode::INVALID_ACQUIRES_RESOURCE_ANNOTATION_ERROR),
            1074 => Ok(StatusCode::GLOBAL_REFERENCE_ERROR),
            1075 => Ok(StatusCode::CONTRAINT_KIND_MISMATCH),
            1076 => Ok(StatusCode::NUMBER_OF_TYPE_ACTUALS_MISMATCH),
            1077 => Ok(StatusCode::LOOP_IN_INSTANTIATION_GRAPH),
            1078 => Ok(StatusCode::UNUSED_LOCALS_SIGNATURE),
            1079 => Ok(StatusCode::UNUSED_TYPE_SIGNATURE),
            1080 => Ok(StatusCode::ZERO_SIZED_STRUCT),
            1081 => Ok(StatusCode::LINKER_ERROR),
            2000 => Ok(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR),
            2001 => Ok(StatusCode::OUT_OF_BOUNDS_INDEX),
            2002 => Ok(StatusCode::OUT_OF_BOUNDS_RANGE),
            2003 => Ok(StatusCode::EMPTY_VALUE_STACK),
            2004 => Ok(StatusCode::EMPTY_CALL_STACK),
            2005 => Ok(StatusCode::PC_OVERFLOW),
            2006 => Ok(StatusCode::VERIFICATION_ERROR),
            2007 => Ok(StatusCode::LOCAL_REFERENCE_ERROR),
            2008 => Ok(StatusCode::STORAGE_ERROR),
            2009 => Ok(StatusCode::INTERNAL_TYPE_ERROR),
            2010 => Ok(StatusCode::EVENT_KEY_MISMATCH),
            2011 => Ok(StatusCode::UNREACHABLE),
            2012 => Ok(StatusCode::VM_STARTUP_FAILURE),
            2013 => Ok(StatusCode::NATIVE_FUNCTION_INTERNAL_INCONSISTENCY),
            3000 => Ok(StatusCode::UNKNOWN_BINARY_ERROR),
            3001 => Ok(StatusCode::MALFORMED),
            3002 => Ok(StatusCode::BAD_MAGIC),
            3003 => Ok(StatusCode::UNKNOWN_VERSION),
            3004 => Ok(StatusCode::UNKNOWN_TABLE_TYPE),
            3005 => Ok(StatusCode::UNKNOWN_SIGNATURE_TYPE),
            3006 => Ok(StatusCode::UNKNOWN_SERIALIZED_TYPE),
            3007 => Ok(StatusCode::UNKNOWN_OPCODE),
            3008 => Ok(StatusCode::BAD_HEADER_TABLE),
            3009 => Ok(StatusCode::UNEXPECTED_SIGNATURE_TYPE),
            3010 => Ok(StatusCode::DUPLICATE_TABLE),
            3011 => Ok(StatusCode::VERIFIER_INVARIANT_VIOLATION),
            4000 => Ok(StatusCode::UNKNOWN_RUNTIME_STATUS),
            4001 => Ok(StatusCode::EXECUTED),
            4002 => Ok(StatusCode::OUT_OF_GAS),
            4003 => Ok(StatusCode::RESOURCE_DOES_NOT_EXIST),
            4004 => Ok(StatusCode::RESOURCE_ALREADY_EXISTS),
            4005 => Ok(StatusCode::EVICTED_ACCOUNT_ACCESS),
            4006 => Ok(StatusCode::ACCOUNT_ADDRESS_ALREADY_EXISTS),
            4007 => Ok(StatusCode::TYPE_ERROR),
            4008 => Ok(StatusCode::MISSING_DATA),
            4009 => Ok(StatusCode::DATA_FORMAT_ERROR),
            4010 => Ok(StatusCode::INVALID_DATA),
            4011 => Ok(StatusCode::REMOTE_DATA_ERROR),
            4012 => Ok(StatusCode::CANNOT_WRITE_EXISTING_RESOURCE),
            4013 => Ok(StatusCode::VALUE_SERIALIZATION_ERROR),
            4014 => Ok(StatusCode::VALUE_DESERIALIZATION_ERROR),
            4015 => Ok(StatusCode::DUPLICATE_MODULE_NAME),
            4016 => Ok(StatusCode::ABORTED),
            4017 => Ok(StatusCode::ARITHMETIC_ERROR),
            4018 => Ok(StatusCode::DYNAMIC_REFERENCE_ERROR),
            4019 => Ok(StatusCode::CODE_DESERIALIZATION_ERROR),
            4020 => Ok(StatusCode::EXECUTION_STACK_OVERFLOW),
            4021 => Ok(StatusCode::CALL_STACK_OVERFLOW),
            4022 => Ok(StatusCode::NATIVE_FUNCTION_ERROR),
            4023 => Ok(StatusCode::GAS_SCHEDULE_ERROR),
            4024 => Ok(StatusCode::CREATE_NULL_ACCOUNT),
            std::u64::MAX => Ok(StatusCode::UNKNOWN_STATUS),
            _ => Err("invalid StatusCode"),
        }
    }
}

impl From<StatusCode> for u64 {
    fn from(status: StatusCode) -> u64 {
        status as u64
    }
}

pub mod sub_status {
    // Arithmetic sub status sub-codes
    pub const AEU_UNKNOWN_ARITHMETIC_ERROR: u64 = 0;
    pub const AEU_UNDERFLOW: u64 = 1;
    pub const AEO_OVERFLOW: u64 = 2;
    pub const AED_DIVISION_BY_ZERO: u64 = 3;

    pub const VSF_GAS_SCHEDULE_NOT_FOUND: u64 = 0;

    // Dynamic Reference status sub-codes
    pub const DRE_UNKNOWN_DYNAMIC_REFERENCE_ERROR: u64 = 0;
    pub const DRE_MOVE_OF_BORROWED_RESOURCE: u64 = 1;
    pub const DRE_GLOBAL_REF_ALREADY_RELEASED: u64 = 2;
    pub const DRE_MISSING_RELEASEREF: u64 = 3;
    pub const DRE_GLOBAL_ALREADY_BORROWED: u64 = 4;

    // Native Function Error sub-codes
    pub const NFE_VECTOR_ERROR_BASE: u64 = 0;

    pub const GSE_UNABLE_TO_LOAD_MODULE: u64 = 0;
    pub const GSE_UNABLE_TO_LOAD_RESOURCE: u64 = 1;
    pub const GSE_UNABLE_TO_DESERIALIZE: u64 = 2;
}
