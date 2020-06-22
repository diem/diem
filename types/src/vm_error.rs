// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![allow(clippy::unit_arg)]

use anyhow::Result;
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

macro_rules! derive_status_try_from_repr {
    (
        #[repr($repr_ty:ident)]
        $( #[$metas:meta] )*
        $vis:vis enum $enum_name:ident {
            $(
                $variant:ident = $value: expr
            ),*
            $( , )?
        }
    ) => {
        #[repr($repr_ty)]
        $( #[$metas] )*
        $vis enum $enum_name {
            $(
                $variant = $value
            ),*
        }

        impl std::convert::TryFrom<$repr_ty> for $enum_name {
            type Error = &'static str;
            fn try_from(value: $repr_ty) -> Result<Self, Self::Error> {
                match value {
                    $(
                        $value => Ok($enum_name::$variant),
                    )*
                    _ => Err("invalid StatusCode"),
                }
            }
        }

        #[cfg(test)]
        const STATUS_CODE_VALUES: &'static [$repr_ty] = &[
            $($value),*
        ];
    };
}

derive_status_try_from_repr! {
#[repr(u64)]
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
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
    // Gas specifier submitted is either malformed (not a valid identifier),
    // or does not refer to an accepted gas specifier
    INVALID_GAS_SPECIFIER = 17,
    // The sending account is frozen
    SENDING_ACCOUNT_FROZEN = 18,
    // Unable to deserialize the account blob
    UNABLE_TO_DESERIALIZE_ACCOUNT = 19,
    // The currency info was unable to be found
    CURRENCY_INFO_DOES_NOT_EXIST = 20,
    // The account sender doesn't have permissions to publish modules
    INVALID_MODULE_PUBLISHER = 21,
    // The sending account has no role
    NO_ACCOUNT_ROLE = 22,

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
    UNSAFE_RET_LOCAL_OR_RESOURCE_STILL_BORROWED = 1029,
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
    NUMBER_OF_TYPE_ARGUMENTS_MISMATCH = 1076,
    LOOP_IN_INSTANTIATION_GRAPH = 1077,
    UNUSED_LOCALS_SIGNATURE = 1078,
    UNUSED_TYPE_SIGNATURE = 1079,
    // Reported when a struct has zero fields
    ZERO_SIZED_STRUCT = 1080,
    LINKER_ERROR = 1081,
    INVALID_CONSTANT_TYPE = 1082,
    MALFORMED_CONSTANT_DATA = 1083,
    EMPTY_CODE_UNIT = 1084,
    INVALID_LOOP_SPLIT = 1085,
    INVALID_LOOP_BREAK = 1086,
    INVALID_LOOP_CONTINUE = 1087,
    UNSAFE_RET_UNUSED_RESOURCES = 1088,
    TOO_MANY_LOCALS = 1089,
    MOVETO_TYPE_MISMATCH_ERROR = 1090,
    MOVETO_NO_RESOURCE_ERROR = 1091,
    GENERIC_MEMBER_OPCODE_MISMATCH = 1092,

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
    INVALID_CODE_CACHE = 2014,

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
    UNKNOWN_NOMINAL_RESOURCE = 3012,
    UNKNOWN_KIND = 3013,
    UNKNOWN_NATIVE_STRUCT_FLAG = 3014,
    BAD_ULEB_U16 = 3015,
    BAD_ULEB_U32 = 3016,
    BAD_U16 = 3017,
    BAD_U32 = 3018,
    BAD_U64 = 3019,
    BAD_U128 = 3020,
    BAD_ULEB_U8 = 3021,

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

    // A reserved status to represent an unknown vm status.
    // this is std::u64::MAX, but we can't pattern match on that, so put the hardcoded value in
    UNKNOWN_STATUS = 18446744073709551615,
}
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
    pub const VSF_LIBRA_VERSION_NOT_FOUND: u64 = 1;

    // Dynamic Reference status sub-codes
    pub const DRE_UNKNOWN_DYNAMIC_REFERENCE_ERROR: u64 = 0;
    pub const DRE_MOVE_OF_BORROWED_RESOURCE: u64 = 1;
    pub const DRE_GLOBAL_REF_ALREADY_RELEASED: u64 = 2;
    pub const DRE_MISSING_RELEASEREF: u64 = 3;
    pub const DRE_GLOBAL_ALREADY_BORROWED: u64 = 4;

    // Native Function Error sub-codes
    pub const NFE_VECTOR_ERROR_BASE: u64 = 0;
    // Failure in LCS deserialization
    pub const NFE_LCS_SERIALIZATION_FAILURE: u64 = 0x1C5;

    pub const GSE_UNABLE_TO_LOAD_MODULE: u64 = 0;
    pub const GSE_UNABLE_TO_LOAD_RESOURCE: u64 = 1;
    pub const GSE_UNABLE_TO_DESERIALIZE: u64 = 2;
}

#[test]
fn test_status_codes() {
    use std::collections::HashSet;
    // Make sure that within the 0-EXECUTION_STATUS_MAX_CODE that all of the status codes succeed
    // when they should, and fail when they should.
    for possible_major_status_code in 0..=EXECUTION_STATUS_MAX_CODE {
        if STATUS_CODE_VALUES.contains(&possible_major_status_code) {
            let status = StatusCode::try_from(possible_major_status_code);
            assert!(status.is_ok());
            let to_major_status_code = u64::from(status.unwrap());
            assert_eq!(possible_major_status_code, to_major_status_code);
        } else {
            assert!(StatusCode::try_from(possible_major_status_code).is_err())
        }
    }

    let mut seen_statuses = HashSet::new();
    let mut seen_codes = HashSet::new();
    // Now make sure that all of the error codes (including any that may be out-of-range) succeed.
    // Make sure there aren't any duplicate mappings
    for major_status_code in STATUS_CODE_VALUES.iter() {
        assert!(
            !seen_codes.contains(major_status_code),
            "Duplicate major_status_code found"
        );
        seen_codes.insert(*major_status_code);
        let status = StatusCode::try_from(*major_status_code);
        assert!(status.is_ok());
        let unwrapped_status = status.unwrap();
        assert!(
            !seen_statuses.contains(&unwrapped_status),
            "Found duplicate u64 -> Status mapping"
        );
        seen_statuses.insert(unwrapped_status);
        let to_major_status_code = u64::from(unwrapped_status);
        assert_eq!(*major_status_code, to_major_status_code);
    }
}
