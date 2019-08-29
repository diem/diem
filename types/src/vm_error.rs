// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![allow(clippy::unit_arg)]

use crate::language_storage::ModuleId;
use failure::prelude::*;
#[cfg(any(test, feature = "testing"))]
use proptest::{collection::vec, prelude::*};
#[cfg(any(test, feature = "testing"))]
use proptest_derive::Arbitrary;
use proto_conv::{FromProto, IntoProto};

// We want conversions here so that we don't need to be dealing with the unknown default values
// that we want in the protobuf.
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
#[cfg_attr(any(test, feature = "testing"), derive(Arbitrary))]
#[cfg_attr(any(test, feature = "testing"), proptest(no_params))]
pub enum VMValidationStatus {
    InvalidSignature,
    InvalidAuthKey,
    SequenceNumberTooOld,
    SequenceNumberTooNew,
    InsufficientBalanceForTransactionFee,
    TransactionExpired,
    SendingAccountDoesNotExist(String),
    RejectedWriteSet,
    InvalidWriteSet,
    ExceededMaxTransactionSize(String),
    UnknownScript,
    UnknownModule,
    MaxGasUnitsExceedsMaxGasUnitsBound(String),
    MaxGasUnitsBelowMinTransactionGasUnits(String),
    GasUnitPriceBelowMinBound(String),
    GasUnitPriceAboveMaxBound(String),
}

// TODO: Add string parameters to all the other types as well
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
#[cfg_attr(any(test, feature = "testing"), derive(Arbitrary))]
#[cfg_attr(any(test, feature = "testing"), proptest(no_params))]
pub enum VMVerificationError {
    IndexOutOfBounds(String),
    CodeUnitIndexOutOfBounds(String),
    RangeOutOfBounds(String),
    NoModuleHandles(String),
    ModuleAddressDoesNotMatchSender(String),
    InvalidSignatureToken(String),
    InvalidFieldDefReference(String),
    RecursiveStructDefinition(String),
    InvalidResourceField(String),
    InvalidFallThrough(String),
    JoinFailure(String),
    NegativeStackSizeWithinBlock(String),
    UnbalancedStack(String),
    InvalidMainFunctionSignature(String),
    DuplicateElement(String),
    InvalidModuleHandle(String),
    UnimplementedHandle(String),
    InconsistentFields(String),
    UnusedFields(String),
    LookupFailed(String),
    VisibilityMismatch(String),
    TypeResolutionFailure(String),
    TypeMismatch(String),
    MissingDependency(String),
    PopReferenceError(String),
    PopResourceError(String),
    ReleaseRefTypeMismatchError(String),
    BrTypeMismatchError(String),
    AbortTypeMismatchError(String),
    StLocTypeMismatchError(String),
    StLocUnsafeToDestroyError(String),
    RetUnsafeToDestroyError(String),
    RetTypeMismatchError(String),
    FreezeRefTypeMismatchError(String),
    FreezeRefExistsMutableBorrowError(String),
    BorrowFieldTypeMismatchError(String),
    BorrowFieldBadFieldError(String),
    BorrowFieldExistsMutableBorrowError(String),
    CopyLocUnavailableError(String),
    CopyLocResourceError(String),
    CopyLocExistsBorrowError(String),
    MoveLocUnavailableError(String),
    MoveLocExistsBorrowError(String),
    BorrowLocReferenceError(String),
    BorrowLocUnavailableError(String),
    BorrowLocExistsBorrowError(String),
    CallTypeMismatchError(String),
    CallBorrowedMutableReferenceError(String),
    PackTypeMismatchError(String),
    UnpackTypeMismatchError(String),
    ReadRefTypeMismatchError(String),
    ReadRefResourceError(String),
    ReadRefExistsMutableBorrowError(String),
    WriteRefTypeMismatchError(String),
    WriteRefResourceError(String),
    WriteRefExistsBorrowError(String),
    WriteRefNoMutableReferenceError(String),
    IntegerOpTypeMismatchError(String),
    BooleanOpTypeMismatchError(String),
    EqualityOpTypeMismatchError(String),
    ExistsResourceTypeMismatchError(String),
    ExistsNoResourceError(String),
    BorrowGlobalTypeMismatchError(String),
    BorrowGlobalNoResourceError(String),
    MoveFromTypeMismatchError(String),
    MoveFromNoResourceError(String),
    MoveToSenderTypeMismatchError(String),
    MoveToSenderNoResourceError(String),
    CreateAccountTypeMismatchError(String),
    GlobalReferenceError(String),
    MissingAcquiresResourceAnnotationError(String),
    ExtraneousAcquiresResourceAnnotationError(String),
    DuplicateAcquiresResourceAnnotationError(String),
    InvalidAcquiresResourceAnnotationError(String),
    ConstraintKindMismatch(String),
    NumberOfTypeActualsMismatch(String),
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
#[cfg_attr(any(test, feature = "testing"), derive(Arbitrary))]
#[cfg_attr(any(test, feature = "testing"), proptest(no_params))]
pub enum VMVerificationStatus {
    /// Verification error in a transaction script.
    Script(VMVerificationError),
    /// Verification error in a module -- the first element is the index of the module with the
    /// error.
    Module(u16, VMVerificationError),
    /// Verification error in a dependent module.
    Dependency(ModuleId, VMVerificationError),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[cfg_attr(any(test, feature = "testing"), derive(Arbitrary))]
pub enum VMInvariantViolationError {
    OutOfBoundsIndex,
    OutOfBoundsRange,
    EmptyValueStack,
    EmptyCallStack,
    PCOverflow,
    LinkerError,
    LocalReferenceError,
    StorageError,
    InternalTypeError,
    EventKeyMismatch,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[cfg_attr(any(test, feature = "testing"), derive(Arbitrary))]
pub enum BinaryError {
    Malformed,
    BadMagic,
    UnknownVersion,
    UnknownTableType,
    UnknownSignatureType,
    UnknownSerializedType,
    UnknownOpcode,
    BadHeaderTable,
    UnexpectedSignatureType,
    DuplicateTable,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[cfg_attr(any(test, feature = "testing"), derive(Arbitrary))]
pub enum DynamicReferenceErrorType {
    MoveOfBorrowedResource,
    GlobalRefAlreadyReleased,
    MissingReleaseRef,
    GlobalAlreadyBorrowed,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[cfg_attr(any(test, feature = "testing"), derive(Arbitrary))]
pub enum ArithmeticErrorType {
    Underflow,
    Overflow,
    DivisionByZero,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[cfg_attr(any(test, feature = "testing"), derive(Arbitrary))]
pub enum ExecutionStatus {
    Executed,
    OutOfGas,
    ResourceDoesNotExist,
    ResourceAlreadyExists,
    EvictedAccountAccess,
    AccountAddressAlreadyExists,
    TypeError,
    MissingData,
    DataFormatError,
    InvalidData,
    RemoteDataError,
    CannotWriteExistingResource,
    ValueSerializationError,
    ValueDeserializationError,
    Aborted(u64),
    ArithmeticError(ArithmeticErrorType),
    DynamicReferenceError(DynamicReferenceErrorType),
    DuplicateModuleName,
    ExecutionStackOverflow,
    CallStackOverflow,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
#[cfg_attr(any(test, feature = "testing"), derive(Arbitrary))]
#[cfg_attr(any(test, feature = "testing"), proptest(no_params))]
pub enum VMStatus {
    Validation(VMValidationStatus),
    InvariantViolation(VMInvariantViolationError),
    Deserialization(BinaryError),
    Execution(ExecutionStatus),
    // As of version 0.9.3, proptest's union (enum) strategies are quadratic time in the number of
    // variants: https://github.com/AltSysrq/proptest/issues/143
    //
    // In particular, if a variant is picked, so are variants for each previous variant (which
    // follows enum definition order for proptest-derive).
    //
    // VerificationStatus is by far the most expensive enum variant to generate since it has a
    // vector of statuses. If it were listed out earlier, it would be generated in a lot more
    // cases than necessary. Move VerificationStatus to the end so that the cost of generating
    // the value tree is only paid when VerificationStatus is generated.
    //
    // Also reduce the size of the vector because VMVerificationStatus is slow to generate
    // for the exact same reason.
    #[cfg_attr(
        any(test, feature = "testing"),
        proptest(
            strategy = "vec(any::<VMVerificationStatus>(), 0..16).prop_map(VMStatus::Verification)"
        )
    )]
    Verification(Vec<VMVerificationStatus>),
}

#[derive(Debug, Fail, Eq, PartialEq)]
pub enum DecodingError {
    #[fail(display = "Module index {} greater than max possible value 65535", _0)]
    ModuleIndexTooBig(u32),
    #[fail(display = "Unknown Validation Status Encountered")]
    UnknownValidationStatusEncountered,
    #[fail(display = "Unknown Verification Error Encountered")]
    UnknownVerificationErrorEncountered,
    #[fail(display = "Unknown Invariant Violation Error Encountered")]
    UnknownInvariantViolationErrorEncountered,
    #[fail(display = "Unknown Transaction Binary Decoding Error Encountered")]
    UnknownBinaryErrorEncountered,
    #[fail(display = "Unknown Reference Error Type Encountered")]
    UnknownDynamicReferenceErrorTypeEncountered,
    #[fail(display = "Unknown Arithmetic Error Type Encountered")]
    UnknownArithmeticErrorTypeEncountered,
    #[fail(display = "Unknown Runtime Status Encountered")]
    UnknownRuntimeStatusEncountered,
    #[fail(display = "Unknown/Invalid VM Status Encountered")]
    InvalidVMStatusEncountered,
}

//***********************************
// Decoding/Encoding to Protobuffers
//***********************************
impl IntoProto for VMValidationStatus {
    type ProtoType = crate::proto::vm_errors::VMValidationStatus;

    fn into_proto(self) -> Self::ProtoType {
        use crate::proto::vm_errors::VMValidationStatusCode as ProtoCode;
        let mut validation_status = Self::ProtoType::new();
        validation_status.set_message("none".to_string());
        match self {
            VMValidationStatus::InvalidSignature => {
                validation_status.set_code(ProtoCode::InvalidSignature)
            }
            VMValidationStatus::InvalidAuthKey => {
                validation_status.set_code(ProtoCode::InvalidAuthKey)
            }
            VMValidationStatus::SequenceNumberTooOld => {
                validation_status.set_code(ProtoCode::SequenceNumberTooOld)
            }
            VMValidationStatus::SequenceNumberTooNew => {
                validation_status.set_code(ProtoCode::SequenceNumberTooNew)
            }
            VMValidationStatus::InsufficientBalanceForTransactionFee => {
                validation_status.set_code(ProtoCode::InsufficientBalanceForTransactionFee)
            }
            VMValidationStatus::TransactionExpired => {
                validation_status.set_code(ProtoCode::TransactionExpired)
            }
            VMValidationStatus::SendingAccountDoesNotExist(msg) => {
                validation_status.set_message(msg);
                validation_status.set_code(ProtoCode::SendingAccountDoesNotExist)
            }
            VMValidationStatus::RejectedWriteSet => {
                validation_status.set_code(ProtoCode::RejectedWriteSet)
            }
            VMValidationStatus::InvalidWriteSet => {
                validation_status.set_code(ProtoCode::InvalidWriteSet)
            }
            VMValidationStatus::ExceededMaxTransactionSize(msg) => {
                validation_status.set_message(msg);
                validation_status.set_code(ProtoCode::ExceededMaxTransactionSize)
            }
            VMValidationStatus::UnknownScript => {
                validation_status.set_code(ProtoCode::UnknownScript)
            }
            VMValidationStatus::UnknownModule => {
                validation_status.set_code(ProtoCode::UnknownModule)
            }
            VMValidationStatus::MaxGasUnitsExceedsMaxGasUnitsBound(msg) => {
                validation_status.set_message(msg);
                validation_status.set_code(ProtoCode::MaxGasUnitsExceedsMaxGasUnitsBound)
            }
            VMValidationStatus::MaxGasUnitsBelowMinTransactionGasUnits(msg) => {
                validation_status.set_message(msg);
                validation_status.set_code(ProtoCode::MaxGasUnitsBelowMinTransactionGasUnits)
            }
            VMValidationStatus::GasUnitPriceBelowMinBound(msg) => {
                validation_status.set_message(msg);
                validation_status.set_code(ProtoCode::GasUnitPriceBelowMinBound)
            }
            VMValidationStatus::GasUnitPriceAboveMaxBound(msg) => {
                validation_status.set_message(msg);
                validation_status.set_code(ProtoCode::GasUnitPriceAboveMaxBound)
            }
        }
        validation_status
    }
}

impl FromProto for VMValidationStatus {
    type ProtoType = crate::proto::vm_errors::VMValidationStatus;

    fn from_proto(mut proto_validation_status: Self::ProtoType) -> Result<VMValidationStatus> {
        use crate::proto::vm_errors::VMValidationStatusCode as ProtoStatus;
        match proto_validation_status.get_code() {
            ProtoStatus::InvalidSignature => Ok(VMValidationStatus::InvalidSignature),
            ProtoStatus::InvalidAuthKey => Ok(VMValidationStatus::InvalidAuthKey),
            ProtoStatus::SequenceNumberTooOld => Ok(VMValidationStatus::SequenceNumberTooOld),
            ProtoStatus::SequenceNumberTooNew => Ok(VMValidationStatus::SequenceNumberTooNew),
            ProtoStatus::InsufficientBalanceForTransactionFee => {
                Ok(VMValidationStatus::InsufficientBalanceForTransactionFee)
            }
            ProtoStatus::TransactionExpired => Ok(VMValidationStatus::TransactionExpired),
            ProtoStatus::SendingAccountDoesNotExist => {
                let msg = proto_validation_status.take_message();
                Ok(VMValidationStatus::SendingAccountDoesNotExist(msg))
            }
            ProtoStatus::RejectedWriteSet => Ok(VMValidationStatus::RejectedWriteSet),
            ProtoStatus::InvalidWriteSet => Ok(VMValidationStatus::InvalidWriteSet),
            ProtoStatus::ExceededMaxTransactionSize => {
                let msg = proto_validation_status.take_message();
                Ok(VMValidationStatus::ExceededMaxTransactionSize(msg))
            }
            ProtoStatus::UnknownScript => Ok(VMValidationStatus::UnknownScript),
            ProtoStatus::UnknownModule => Ok(VMValidationStatus::UnknownModule),
            ProtoStatus::MaxGasUnitsExceedsMaxGasUnitsBound => {
                let msg = proto_validation_status.take_message();
                Ok(VMValidationStatus::MaxGasUnitsExceedsMaxGasUnitsBound(msg))
            }
            ProtoStatus::MaxGasUnitsBelowMinTransactionGasUnits => {
                let msg = proto_validation_status.take_message();
                Ok(VMValidationStatus::MaxGasUnitsBelowMinTransactionGasUnits(
                    msg,
                ))
            }
            ProtoStatus::GasUnitPriceBelowMinBound => {
                let msg = proto_validation_status.take_message();
                Ok(VMValidationStatus::GasUnitPriceBelowMinBound(msg))
            }
            ProtoStatus::GasUnitPriceAboveMaxBound => {
                let msg = proto_validation_status.take_message();
                Ok(VMValidationStatus::GasUnitPriceAboveMaxBound(msg))
            }
            ProtoStatus::UnknownValidationStatus => {
                bail_err!(DecodingError::UnknownValidationStatusEncountered)
            }
        }
    }
}

impl IntoProto for VMVerificationError {
    type ProtoType = (crate::proto::vm_errors::VMVerificationErrorKind, String);

    fn into_proto(self) -> Self::ProtoType {
        use crate::proto::vm_errors::VMVerificationErrorKind as ProtoKind;
        match self {
            VMVerificationError::IndexOutOfBounds(message) => {
                (ProtoKind::IndexOutOfBounds, message)
            }
            VMVerificationError::CodeUnitIndexOutOfBounds(message) => {
                (ProtoKind::CodeUnitIndexOutOfBounds, message)
            }
            VMVerificationError::RangeOutOfBounds(message) => {
                (ProtoKind::RangeOutOfBounds, message)
            }
            VMVerificationError::NoModuleHandles(message) => (ProtoKind::NoModuleHandles, message),
            VMVerificationError::ModuleAddressDoesNotMatchSender(message) => {
                (ProtoKind::ModuleAddressDoesNotMatchSender, message)
            }
            VMVerificationError::InvalidSignatureToken(message) => {
                (ProtoKind::InvalidSignatureToken, message)
            }
            VMVerificationError::InvalidFieldDefReference(message) => {
                (ProtoKind::InvalidFieldDefReference, message)
            }
            VMVerificationError::RecursiveStructDefinition(message) => {
                (ProtoKind::RecursiveStructDefinition, message)
            }
            VMVerificationError::InvalidResourceField(message) => {
                (ProtoKind::InvalidResourceField, message)
            }
            VMVerificationError::InvalidFallThrough(message) => {
                (ProtoKind::InvalidFallThrough, message)
            }
            VMVerificationError::JoinFailure(message) => (ProtoKind::JoinFailure, message),
            VMVerificationError::NegativeStackSizeWithinBlock(message) => {
                (ProtoKind::NegativeStackSizeWithinBlock, message)
            }
            VMVerificationError::UnbalancedStack(message) => (ProtoKind::UnbalancedStack, message),
            VMVerificationError::InvalidMainFunctionSignature(message) => {
                (ProtoKind::InvalidMainFunctionSignature, message)
            }
            VMVerificationError::DuplicateElement(message) => {
                (ProtoKind::DuplicateElement, message)
            }
            VMVerificationError::InvalidModuleHandle(message) => {
                (ProtoKind::InvalidModuleHandle, message)
            }
            VMVerificationError::UnimplementedHandle(message) => {
                (ProtoKind::UnimplementedHandle, message)
            }
            VMVerificationError::InconsistentFields(message) => {
                (ProtoKind::InconsistentFields, message)
            }
            VMVerificationError::UnusedFields(message) => (ProtoKind::UnusedFields, message),
            VMVerificationError::LookupFailed(message) => (ProtoKind::LookupFailed, message),
            VMVerificationError::VisibilityMismatch(message) => {
                (ProtoKind::VisibilityMismatch, message)
            }
            VMVerificationError::TypeResolutionFailure(message) => {
                (ProtoKind::TypeResolutionFailure, message)
            }
            VMVerificationError::TypeMismatch(message) => (ProtoKind::TypeMismatch, message),
            VMVerificationError::MissingDependency(message) => {
                (ProtoKind::MissingDependency, message)
            }
            VMVerificationError::PopReferenceError(message) => {
                (ProtoKind::PopReferenceError, message)
            }
            VMVerificationError::PopResourceError(message) => {
                (ProtoKind::PopResourceError, message)
            }
            VMVerificationError::ReleaseRefTypeMismatchError(message) => {
                (ProtoKind::ReleaseRefTypeMismatchError, message)
            }
            VMVerificationError::BrTypeMismatchError(message) => {
                (ProtoKind::BrTypeMismatchError, message)
            }
            VMVerificationError::AbortTypeMismatchError(message) => {
                (ProtoKind::AbortTypeMismatchError, message)
            }
            VMVerificationError::StLocTypeMismatchError(message) => {
                (ProtoKind::StLocTypeMismatchError, message)
            }
            VMVerificationError::StLocUnsafeToDestroyError(message) => {
                (ProtoKind::StLocUnsafeToDestroyError, message)
            }
            VMVerificationError::RetUnsafeToDestroyError(message) => {
                (ProtoKind::RetUnsafeToDestroyError, message)
            }
            VMVerificationError::RetTypeMismatchError(message) => {
                (ProtoKind::RetTypeMismatchError, message)
            }
            VMVerificationError::FreezeRefTypeMismatchError(message) => {
                (ProtoKind::FreezeRefTypeMismatchError, message)
            }
            VMVerificationError::FreezeRefExistsMutableBorrowError(message) => {
                (ProtoKind::FreezeRefExistsMutableBorrowError, message)
            }
            VMVerificationError::BorrowFieldTypeMismatchError(message) => {
                (ProtoKind::BorrowFieldTypeMismatchError, message)
            }
            VMVerificationError::BorrowFieldBadFieldError(message) => {
                (ProtoKind::BorrowFieldBadFieldError, message)
            }
            VMVerificationError::BorrowFieldExistsMutableBorrowError(message) => {
                (ProtoKind::BorrowFieldExistsMutableBorrowError, message)
            }
            VMVerificationError::CopyLocUnavailableError(message) => {
                (ProtoKind::CopyLocUnavailableError, message)
            }
            VMVerificationError::CopyLocResourceError(message) => {
                (ProtoKind::CopyLocResourceError, message)
            }
            VMVerificationError::CopyLocExistsBorrowError(message) => {
                (ProtoKind::CopyLocExistsBorrowError, message)
            }
            VMVerificationError::MoveLocUnavailableError(message) => {
                (ProtoKind::MoveLocUnavailableError, message)
            }
            VMVerificationError::MoveLocExistsBorrowError(message) => {
                (ProtoKind::MoveLocExistsBorrowError, message)
            }
            VMVerificationError::BorrowLocReferenceError(message) => {
                (ProtoKind::BorrowLocReferenceError, message)
            }
            VMVerificationError::BorrowLocUnavailableError(message) => {
                (ProtoKind::BorrowLocUnavailableError, message)
            }
            VMVerificationError::BorrowLocExistsBorrowError(message) => {
                (ProtoKind::BorrowLocExistsBorrowError, message)
            }
            VMVerificationError::CallTypeMismatchError(message) => {
                (ProtoKind::CallTypeMismatchError, message)
            }
            VMVerificationError::CallBorrowedMutableReferenceError(message) => {
                (ProtoKind::CallBorrowedMutableReferenceError, message)
            }
            VMVerificationError::PackTypeMismatchError(message) => {
                (ProtoKind::PackTypeMismatchError, message)
            }
            VMVerificationError::UnpackTypeMismatchError(message) => {
                (ProtoKind::UnpackTypeMismatchError, message)
            }
            VMVerificationError::ReadRefTypeMismatchError(message) => {
                (ProtoKind::ReadRefTypeMismatchError, message)
            }
            VMVerificationError::ReadRefResourceError(message) => {
                (ProtoKind::ReadRefResourceError, message)
            }
            VMVerificationError::ReadRefExistsMutableBorrowError(message) => {
                (ProtoKind::ReadRefExistsMutableBorrowError, message)
            }
            VMVerificationError::WriteRefTypeMismatchError(message) => {
                (ProtoKind::WriteRefTypeMismatchError, message)
            }
            VMVerificationError::WriteRefResourceError(message) => {
                (ProtoKind::WriteRefResourceError, message)
            }
            VMVerificationError::WriteRefExistsBorrowError(message) => {
                (ProtoKind::WriteRefExistsBorrowError, message)
            }
            VMVerificationError::WriteRefNoMutableReferenceError(message) => {
                (ProtoKind::WriteRefNoMutableReferenceError, message)
            }
            VMVerificationError::IntegerOpTypeMismatchError(message) => {
                (ProtoKind::IntegerOpTypeMismatchError, message)
            }
            VMVerificationError::BooleanOpTypeMismatchError(message) => {
                (ProtoKind::BooleanOpTypeMismatchError, message)
            }
            VMVerificationError::EqualityOpTypeMismatchError(message) => {
                (ProtoKind::EqualityOpTypeMismatchError, message)
            }
            VMVerificationError::ExistsResourceTypeMismatchError(message) => {
                (ProtoKind::ExistsResourceTypeMismatchError, message)
            }
            VMVerificationError::ExistsNoResourceError(message) => {
                (ProtoKind::ExistsNoResourceError, message)
            }
            VMVerificationError::BorrowGlobalTypeMismatchError(message) => {
                (ProtoKind::BorrowGlobalTypeMismatchError, message)
            }
            VMVerificationError::BorrowGlobalNoResourceError(message) => {
                (ProtoKind::BorrowGlobalNoResourceError, message)
            }
            VMVerificationError::MoveFromTypeMismatchError(message) => {
                (ProtoKind::MoveFromTypeMismatchError, message)
            }
            VMVerificationError::MoveFromNoResourceError(message) => {
                (ProtoKind::MoveFromNoResourceError, message)
            }
            VMVerificationError::MoveToSenderTypeMismatchError(message) => {
                (ProtoKind::MoveToSenderTypeMismatchError, message)
            }
            VMVerificationError::MoveToSenderNoResourceError(message) => {
                (ProtoKind::MoveToSenderNoResourceError, message)
            }
            VMVerificationError::CreateAccountTypeMismatchError(message) => {
                (ProtoKind::CreateAccountTypeMismatchError, message)
            }
            VMVerificationError::GlobalReferenceError(message) => {
                (ProtoKind::GlobalReferenceError, message)
            }
            VMVerificationError::MissingAcquiresResourceAnnotationError(message) => {
                (ProtoKind::MissingAcquiresResourceAnnotationError, message)
            }
            VMVerificationError::ExtraneousAcquiresResourceAnnotationError(message) => (
                ProtoKind::ExtraneousAcquiresResourceAnnotationError,
                message,
            ),
            VMVerificationError::InvalidAcquiresResourceAnnotationError(message) => {
                (ProtoKind::InvalidAcquiresResourceAnnotationError, message)
            }
            VMVerificationError::DuplicateAcquiresResourceAnnotationError(message) => {
                (ProtoKind::DuplicateAcquiresResourceAnnotationError, message)
            }
            VMVerificationError::ConstraintKindMismatch(message) => {
                (ProtoKind::ConstraintKindMismatch, message)
            }
            VMVerificationError::NumberOfTypeActualsMismatch(message) => {
                (ProtoKind::NumberOfTypeActualsMismatch, message)
            }
        }
    }
}

impl FromProto for VMVerificationError {
    type ProtoType = (crate::proto::vm_errors::VMVerificationErrorKind, String);

    fn from_proto(proto_verification_error: Self::ProtoType) -> Result<VMVerificationError> {
        use crate::proto::vm_errors::VMVerificationErrorKind as ProtoKind;

        let (kind, message) = proto_verification_error;
        match kind {
            ProtoKind::IndexOutOfBounds => Ok(VMVerificationError::IndexOutOfBounds(message)),
            ProtoKind::CodeUnitIndexOutOfBounds => {
                Ok(VMVerificationError::CodeUnitIndexOutOfBounds(message))
            }
            ProtoKind::RangeOutOfBounds => Ok(VMVerificationError::RangeOutOfBounds(message)),
            ProtoKind::NoModuleHandles => Ok(VMVerificationError::NoModuleHandles(message)),
            ProtoKind::ModuleAddressDoesNotMatchSender => Ok(
                VMVerificationError::ModuleAddressDoesNotMatchSender(message),
            ),
            ProtoKind::InvalidSignatureToken => {
                Ok(VMVerificationError::InvalidSignatureToken(message))
            }
            ProtoKind::InvalidFieldDefReference => {
                Ok(VMVerificationError::InvalidFieldDefReference(message))
            }
            ProtoKind::RecursiveStructDefinition => {
                Ok(VMVerificationError::RecursiveStructDefinition(message))
            }
            ProtoKind::InvalidResourceField => {
                Ok(VMVerificationError::InvalidResourceField(message))
            }
            ProtoKind::InvalidFallThrough => Ok(VMVerificationError::InvalidFallThrough(message)),
            ProtoKind::JoinFailure => Ok(VMVerificationError::JoinFailure(message)),
            ProtoKind::NegativeStackSizeWithinBlock => {
                Ok(VMVerificationError::NegativeStackSizeWithinBlock(message))
            }
            ProtoKind::UnbalancedStack => Ok(VMVerificationError::UnbalancedStack(message)),
            ProtoKind::InvalidMainFunctionSignature => {
                Ok(VMVerificationError::InvalidMainFunctionSignature(message))
            }
            ProtoKind::DuplicateElement => Ok(VMVerificationError::DuplicateElement(message)),
            ProtoKind::InvalidModuleHandle => Ok(VMVerificationError::InvalidModuleHandle(message)),
            ProtoKind::UnimplementedHandle => Ok(VMVerificationError::UnimplementedHandle(message)),
            ProtoKind::InconsistentFields => Ok(VMVerificationError::InconsistentFields(message)),
            ProtoKind::UnusedFields => Ok(VMVerificationError::UnusedFields(message)),
            ProtoKind::LookupFailed => Ok(VMVerificationError::LookupFailed(message)),
            ProtoKind::VisibilityMismatch => Ok(VMVerificationError::VisibilityMismatch(message)),
            ProtoKind::TypeResolutionFailure => {
                Ok(VMVerificationError::TypeResolutionFailure(message))
            }
            ProtoKind::TypeMismatch => Ok(VMVerificationError::TypeMismatch(message)),
            ProtoKind::MissingDependency => Ok(VMVerificationError::MissingDependency(message)),
            ProtoKind::PopReferenceError => Ok(VMVerificationError::PopReferenceError(message)),
            ProtoKind::PopResourceError => Ok(VMVerificationError::PopResourceError(message)),
            ProtoKind::ReleaseRefTypeMismatchError => {
                Ok(VMVerificationError::ReleaseRefTypeMismatchError(message))
            }
            ProtoKind::BrTypeMismatchError => Ok(VMVerificationError::BrTypeMismatchError(message)),
            ProtoKind::AbortTypeMismatchError => {
                Ok(VMVerificationError::AbortTypeMismatchError(message))
            }
            ProtoKind::StLocTypeMismatchError => {
                Ok(VMVerificationError::StLocTypeMismatchError(message))
            }
            ProtoKind::StLocUnsafeToDestroyError => {
                Ok(VMVerificationError::StLocUnsafeToDestroyError(message))
            }
            ProtoKind::RetUnsafeToDestroyError => {
                Ok(VMVerificationError::RetUnsafeToDestroyError(message))
            }
            ProtoKind::RetTypeMismatchError => {
                Ok(VMVerificationError::RetTypeMismatchError(message))
            }
            ProtoKind::FreezeRefTypeMismatchError => {
                Ok(VMVerificationError::FreezeRefTypeMismatchError(message))
            }
            ProtoKind::FreezeRefExistsMutableBorrowError => Ok(
                VMVerificationError::FreezeRefExistsMutableBorrowError(message),
            ),
            ProtoKind::BorrowFieldTypeMismatchError => {
                Ok(VMVerificationError::BorrowFieldTypeMismatchError(message))
            }
            ProtoKind::BorrowFieldBadFieldError => {
                Ok(VMVerificationError::BorrowFieldBadFieldError(message))
            }
            ProtoKind::BorrowFieldExistsMutableBorrowError => Ok(
                VMVerificationError::BorrowFieldExistsMutableBorrowError(message),
            ),
            ProtoKind::CopyLocUnavailableError => {
                Ok(VMVerificationError::CopyLocUnavailableError(message))
            }
            ProtoKind::CopyLocResourceError => {
                Ok(VMVerificationError::CopyLocResourceError(message))
            }
            ProtoKind::CopyLocExistsBorrowError => {
                Ok(VMVerificationError::CopyLocExistsBorrowError(message))
            }
            ProtoKind::MoveLocUnavailableError => {
                Ok(VMVerificationError::MoveLocUnavailableError(message))
            }
            ProtoKind::MoveLocExistsBorrowError => {
                Ok(VMVerificationError::MoveLocExistsBorrowError(message))
            }
            ProtoKind::BorrowLocReferenceError => {
                Ok(VMVerificationError::BorrowLocReferenceError(message))
            }
            ProtoKind::BorrowLocUnavailableError => {
                Ok(VMVerificationError::BorrowLocUnavailableError(message))
            }
            ProtoKind::BorrowLocExistsBorrowError => {
                Ok(VMVerificationError::BorrowLocExistsBorrowError(message))
            }
            ProtoKind::CallTypeMismatchError => {
                Ok(VMVerificationError::CallTypeMismatchError(message))
            }
            ProtoKind::CallBorrowedMutableReferenceError => Ok(
                VMVerificationError::CallBorrowedMutableReferenceError(message),
            ),
            ProtoKind::PackTypeMismatchError => {
                Ok(VMVerificationError::PackTypeMismatchError(message))
            }
            ProtoKind::UnpackTypeMismatchError => {
                Ok(VMVerificationError::UnpackTypeMismatchError(message))
            }
            ProtoKind::ReadRefTypeMismatchError => {
                Ok(VMVerificationError::ReadRefTypeMismatchError(message))
            }
            ProtoKind::ReadRefResourceError => {
                Ok(VMVerificationError::ReadRefResourceError(message))
            }
            ProtoKind::ReadRefExistsMutableBorrowError => Ok(
                VMVerificationError::ReadRefExistsMutableBorrowError(message),
            ),
            ProtoKind::WriteRefTypeMismatchError => {
                Ok(VMVerificationError::WriteRefTypeMismatchError(message))
            }
            ProtoKind::WriteRefResourceError => {
                Ok(VMVerificationError::WriteRefResourceError(message))
            }
            ProtoKind::WriteRefExistsBorrowError => {
                Ok(VMVerificationError::WriteRefExistsBorrowError(message))
            }
            ProtoKind::WriteRefNoMutableReferenceError => Ok(
                VMVerificationError::WriteRefNoMutableReferenceError(message),
            ),
            ProtoKind::IntegerOpTypeMismatchError => {
                Ok(VMVerificationError::IntegerOpTypeMismatchError(message))
            }
            ProtoKind::BooleanOpTypeMismatchError => {
                Ok(VMVerificationError::BooleanOpTypeMismatchError(message))
            }
            ProtoKind::EqualityOpTypeMismatchError => {
                Ok(VMVerificationError::EqualityOpTypeMismatchError(message))
            }
            ProtoKind::ExistsResourceTypeMismatchError => Ok(
                VMVerificationError::ExistsResourceTypeMismatchError(message),
            ),
            ProtoKind::ExistsNoResourceError => {
                Ok(VMVerificationError::ExistsNoResourceError(message))
            }
            ProtoKind::BorrowGlobalTypeMismatchError => {
                Ok(VMVerificationError::BorrowGlobalTypeMismatchError(message))
            }
            ProtoKind::BorrowGlobalNoResourceError => {
                Ok(VMVerificationError::BorrowGlobalNoResourceError(message))
            }
            ProtoKind::MoveFromTypeMismatchError => {
                Ok(VMVerificationError::MoveFromTypeMismatchError(message))
            }
            ProtoKind::MoveFromNoResourceError => {
                Ok(VMVerificationError::MoveFromNoResourceError(message))
            }
            ProtoKind::MoveToSenderTypeMismatchError => {
                Ok(VMVerificationError::MoveToSenderTypeMismatchError(message))
            }
            ProtoKind::MoveToSenderNoResourceError => {
                Ok(VMVerificationError::MoveToSenderNoResourceError(message))
            }
            ProtoKind::CreateAccountTypeMismatchError => {
                Ok(VMVerificationError::CreateAccountTypeMismatchError(message))
            }
            ProtoKind::GlobalReferenceError => {
                Ok(VMVerificationError::GlobalReferenceError(message))
            }
            ProtoKind::MissingAcquiresResourceAnnotationError => Ok(
                VMVerificationError::MissingAcquiresResourceAnnotationError(message),
            ),
            ProtoKind::ExtraneousAcquiresResourceAnnotationError => {
                Ok(VMVerificationError::ExtraneousAcquiresResourceAnnotationError(message))
            }
            ProtoKind::DuplicateAcquiresResourceAnnotationError => {
                Ok(VMVerificationError::DuplicateAcquiresResourceAnnotationError(message))
            }
            ProtoKind::InvalidAcquiresResourceAnnotationError => Ok(
                VMVerificationError::InvalidAcquiresResourceAnnotationError(message),
            ),
            ProtoKind::ConstraintKindMismatch => {
                Ok(VMVerificationError::ConstraintKindMismatch(message))
            }
            ProtoKind::NumberOfTypeActualsMismatch => {
                Ok(VMVerificationError::NumberOfTypeActualsMismatch(message))
            }
            ProtoKind::UnknownVerificationError => {
                bail_err!(DecodingError::UnknownVerificationErrorEncountered)
            }
        }
    }
}

impl IntoProto for VMVerificationStatus {
    type ProtoType = crate::proto::vm_errors::VMVerificationStatus;

    fn into_proto(self) -> Self::ProtoType {
        use crate::proto::vm_errors::VMVerificationStatus_StatusKind as ProtoStatusKind;

        let mut proto_status = Self::ProtoType::new();

        let (kind, message) = match self {
            VMVerificationStatus::Script(error) => {
                proto_status.set_status_kind(ProtoStatusKind::SCRIPT);
                error.into_proto()
            }
            VMVerificationStatus::Module(module_idx, error) => {
                proto_status.set_status_kind(ProtoStatusKind::MODULE);
                proto_status.set_module_idx(u32::from(module_idx));
                error.into_proto()
            }
            VMVerificationStatus::Dependency(dependency_id, error) => {
                proto_status.set_status_kind(ProtoStatusKind::DEPENDENCY);
                proto_status.set_dependency_id(dependency_id.into_proto());
                error.into_proto()
            }
        };
        proto_status.set_error_kind(kind);
        proto_status.set_message(message);
        proto_status
    }
}

impl FromProto for VMVerificationStatus {
    type ProtoType = crate::proto::vm_errors::VMVerificationStatus;

    fn from_proto(mut proto_status: Self::ProtoType) -> Result<Self> {
        use crate::proto::vm_errors::VMVerificationStatus_StatusKind as ProtoStatusKind;

        let err = VMVerificationError::from_proto((
            proto_status.get_error_kind(),
            proto_status.take_message(),
        ))?;

        match proto_status.get_status_kind() {
            ProtoStatusKind::SCRIPT => Ok(VMVerificationStatus::Script(err)),
            ProtoStatusKind::MODULE => {
                let module_idx = proto_status.get_module_idx();
                if module_idx > u32::from(u16::max_value()) {
                    bail_err!(DecodingError::ModuleIndexTooBig(module_idx));
                }
                Ok(VMVerificationStatus::Module(module_idx as u16, err))
            }
            ProtoStatusKind::DEPENDENCY => {
                let dependency_id = ModuleId::from_proto(proto_status.take_dependency_id())?;
                Ok(VMVerificationStatus::Dependency(dependency_id, err))
            }
        }
    }
}

impl IntoProto for VMInvariantViolationError {
    type ProtoType = crate::proto::vm_errors::VMInvariantViolationError;

    fn into_proto(self) -> Self::ProtoType {
        use crate::proto::vm_errors::VMInvariantViolationError as ProtoStatus;
        match self {
            VMInvariantViolationError::OutOfBoundsIndex => ProtoStatus::OutOfBoundsIndex,
            VMInvariantViolationError::OutOfBoundsRange => ProtoStatus::OutOfBoundsRange,
            VMInvariantViolationError::EmptyValueStack => ProtoStatus::EmptyValueStack,
            VMInvariantViolationError::EmptyCallStack => ProtoStatus::EmptyCallStack,
            VMInvariantViolationError::PCOverflow => ProtoStatus::PCOverflow,
            VMInvariantViolationError::LinkerError => ProtoStatus::LinkerError,
            VMInvariantViolationError::LocalReferenceError => ProtoStatus::LocalReferenceError,
            VMInvariantViolationError::StorageError => ProtoStatus::StorageError,
            VMInvariantViolationError::InternalTypeError => ProtoStatus::InternalTypeError,
            VMInvariantViolationError::EventKeyMismatch => ProtoStatus::EventKeyMismatch,
        }
    }
}

impl FromProto for VMInvariantViolationError {
    type ProtoType = crate::proto::vm_errors::VMInvariantViolationError;

    fn from_proto(proto_invariant_violation: Self::ProtoType) -> Result<VMInvariantViolationError> {
        use crate::proto::vm_errors::VMInvariantViolationError as ProtoError;
        match proto_invariant_violation {
            ProtoError::OutOfBoundsIndex => Ok(VMInvariantViolationError::OutOfBoundsIndex),
            ProtoError::OutOfBoundsRange => Ok(VMInvariantViolationError::OutOfBoundsRange),
            ProtoError::EmptyValueStack => Ok(VMInvariantViolationError::EmptyValueStack),
            ProtoError::EmptyCallStack => Ok(VMInvariantViolationError::EmptyCallStack),
            ProtoError::PCOverflow => Ok(VMInvariantViolationError::PCOverflow),
            ProtoError::LinkerError => Ok(VMInvariantViolationError::LinkerError),
            ProtoError::LocalReferenceError => Ok(VMInvariantViolationError::LocalReferenceError),
            ProtoError::StorageError => Ok(VMInvariantViolationError::StorageError),
            ProtoError::InternalTypeError => Ok(VMInvariantViolationError::InternalTypeError),
            ProtoError::EventKeyMismatch => Ok(VMInvariantViolationError::EventKeyMismatch),
            ProtoError::UnknownInvariantViolationError => {
                bail_err!(DecodingError::UnknownInvariantViolationErrorEncountered)
            }
        }
    }
}

impl IntoProto for BinaryError {
    type ProtoType = crate::proto::vm_errors::BinaryError;

    fn into_proto(self) -> Self::ProtoType {
        use crate::proto::vm_errors::BinaryError as ProtoStatus;
        match self {
            BinaryError::Malformed => ProtoStatus::Malformed,
            BinaryError::BadMagic => ProtoStatus::BadMagic,
            BinaryError::UnknownVersion => ProtoStatus::UnknownVersion,
            BinaryError::UnknownTableType => ProtoStatus::UnknownTableType,
            BinaryError::UnknownSignatureType => ProtoStatus::UnknownSignatureType,
            BinaryError::UnknownSerializedType => ProtoStatus::UnknownSerializedType,
            BinaryError::UnknownOpcode => ProtoStatus::UnknownOpcode,
            BinaryError::BadHeaderTable => ProtoStatus::BadHeaderTable,
            BinaryError::UnexpectedSignatureType => ProtoStatus::UnexpectedSignatureType,
            BinaryError::DuplicateTable => ProtoStatus::DuplicateTable,
        }
    }
}

impl FromProto for BinaryError {
    type ProtoType = crate::proto::vm_errors::BinaryError;

    fn from_proto(proto_binary_error: Self::ProtoType) -> Result<BinaryError> {
        use crate::proto::vm_errors::BinaryError as ProtoError;
        match proto_binary_error {
            ProtoError::Malformed => Ok(BinaryError::Malformed),
            ProtoError::BadMagic => Ok(BinaryError::BadMagic),
            ProtoError::UnknownVersion => Ok(BinaryError::UnknownVersion),
            ProtoError::UnknownTableType => Ok(BinaryError::UnknownTableType),
            ProtoError::UnknownSignatureType => Ok(BinaryError::UnknownSignatureType),
            ProtoError::UnknownSerializedType => Ok(BinaryError::UnknownSerializedType),
            ProtoError::UnknownOpcode => Ok(BinaryError::UnknownOpcode),
            ProtoError::BadHeaderTable => Ok(BinaryError::BadHeaderTable),
            ProtoError::UnexpectedSignatureType => Ok(BinaryError::UnexpectedSignatureType),
            ProtoError::DuplicateTable => Ok(BinaryError::DuplicateTable),
            ProtoError::UnknownBinaryError => {
                bail_err!(DecodingError::UnknownBinaryErrorEncountered)
            }
        }
    }
}

impl IntoProto for DynamicReferenceErrorType {
    type ProtoType = crate::proto::vm_errors::DynamicReferenceError_DynamicReferenceErrorType;

    fn into_proto(self) -> Self::ProtoType {
        use crate::proto::vm_errors::DynamicReferenceError_DynamicReferenceErrorType as ProtoError;
        match self {
            DynamicReferenceErrorType::MoveOfBorrowedResource => ProtoError::MoveOfBorrowedResource,
            DynamicReferenceErrorType::GlobalRefAlreadyReleased => {
                ProtoError::GlobalRefAlreadyReleased
            }
            DynamicReferenceErrorType::MissingReleaseRef => ProtoError::MissingReleaseRef,
            DynamicReferenceErrorType::GlobalAlreadyBorrowed => ProtoError::GlobalAlreadyBorrowed,
        }
    }
}

impl FromProto for DynamicReferenceErrorType {
    type ProtoType = crate::proto::vm_errors::DynamicReferenceError_DynamicReferenceErrorType;

    fn from_proto(proto_ref_err_type: Self::ProtoType) -> Result<DynamicReferenceErrorType> {
        use crate::proto::vm_errors::DynamicReferenceError_DynamicReferenceErrorType as ProtoError;
        match proto_ref_err_type {
            ProtoError::MoveOfBorrowedResource => {
                Ok(DynamicReferenceErrorType::MoveOfBorrowedResource)
            }
            ProtoError::GlobalRefAlreadyReleased => {
                Ok(DynamicReferenceErrorType::GlobalRefAlreadyReleased)
            }
            ProtoError::MissingReleaseRef => Ok(DynamicReferenceErrorType::MissingReleaseRef),
            ProtoError::GlobalAlreadyBorrowed => {
                Ok(DynamicReferenceErrorType::GlobalAlreadyBorrowed)
            }
            ProtoError::UnknownDynamicReferenceError => {
                bail_err!(DecodingError::UnknownDynamicReferenceErrorTypeEncountered)
            }
        }
    }
}

impl IntoProto for ArithmeticErrorType {
    type ProtoType = crate::proto::vm_errors::ArithmeticError_ArithmeticErrorType;

    fn into_proto(self) -> Self::ProtoType {
        use crate::proto::vm_errors::ArithmeticError_ArithmeticErrorType as ProtoError;
        match self {
            ArithmeticErrorType::Underflow => ProtoError::Underflow,
            ArithmeticErrorType::Overflow => ProtoError::Overflow,
            ArithmeticErrorType::DivisionByZero => ProtoError::DivisionByZero,
        }
    }
}

impl FromProto for ArithmeticErrorType {
    type ProtoType = crate::proto::vm_errors::ArithmeticError_ArithmeticErrorType;

    fn from_proto(proto_ref_err_type: Self::ProtoType) -> Result<ArithmeticErrorType> {
        use crate::proto::vm_errors::ArithmeticError_ArithmeticErrorType as ProtoError;
        match proto_ref_err_type {
            ProtoError::Underflow => Ok(ArithmeticErrorType::Underflow),
            ProtoError::Overflow => Ok(ArithmeticErrorType::Overflow),
            ProtoError::DivisionByZero => Ok(ArithmeticErrorType::DivisionByZero),
            ProtoError::UnknownArithmeticError => {
                bail_err!(DecodingError::UnknownArithmeticErrorTypeEncountered)
            }
        }
    }
}

impl IntoProto for ExecutionStatus {
    type ProtoType = crate::proto::vm_errors::ExecutionStatus;

    fn into_proto(self) -> Self::ProtoType {
        use crate::proto::vm_errors::{
            Aborted, ArithmeticError, DynamicReferenceError, ExecutionStatus as ExecuteStatus,
            RuntimeStatus,
        };
        let mut exec_status = ExecuteStatus::new();
        match self {
            ExecutionStatus::Executed => exec_status.set_runtime_status(RuntimeStatus::Executed),
            ExecutionStatus::OutOfGas => exec_status.set_runtime_status(RuntimeStatus::OutOfGas),
            ExecutionStatus::ResourceDoesNotExist => {
                exec_status.set_runtime_status(RuntimeStatus::ResourceDoesNotExist)
            }
            ExecutionStatus::ResourceAlreadyExists => {
                exec_status.set_runtime_status(RuntimeStatus::ResourceAlreadyExists)
            }
            ExecutionStatus::EvictedAccountAccess => {
                exec_status.set_runtime_status(RuntimeStatus::EvictedAccountAccess)
            }
            ExecutionStatus::AccountAddressAlreadyExists => {
                exec_status.set_runtime_status(RuntimeStatus::AccountAddressAlreadyExists)
            }
            ExecutionStatus::TypeError => exec_status.set_runtime_status(RuntimeStatus::TypeError),
            ExecutionStatus::MissingData => {
                exec_status.set_runtime_status(RuntimeStatus::MissingData)
            }
            ExecutionStatus::DataFormatError => {
                exec_status.set_runtime_status(RuntimeStatus::DataFormatError)
            }
            ExecutionStatus::InvalidData => {
                exec_status.set_runtime_status(RuntimeStatus::InvalidData)
            }
            ExecutionStatus::RemoteDataError => {
                exec_status.set_runtime_status(RuntimeStatus::RemoteDataError)
            }
            ExecutionStatus::CannotWriteExistingResource => {
                exec_status.set_runtime_status(RuntimeStatus::CannotWriteExistingResource)
            }
            ExecutionStatus::ValueSerializationError => {
                exec_status.set_runtime_status(RuntimeStatus::ValueSerializationError)
            }
            ExecutionStatus::ValueDeserializationError => {
                exec_status.set_runtime_status(RuntimeStatus::ValueDeserializationError)
            }
            ExecutionStatus::DuplicateModuleName => {
                exec_status.set_runtime_status(RuntimeStatus::DuplicateModuleName)
            }
            ExecutionStatus::DynamicReferenceError(err_type) => {
                let mut ref_err = DynamicReferenceError::new();
                let err_code = DynamicReferenceErrorType::into_proto(err_type);
                ref_err.set_error_code(err_code);
                exec_status.set_reference_error(ref_err)
            }
            ExecutionStatus::ArithmeticError(err_type) => {
                let mut arith_err = ArithmeticError::new();
                let err_code = ArithmeticErrorType::into_proto(err_type);
                arith_err.set_error_code(err_code);
                exec_status.set_arithmetic_error(arith_err)
            }
            ExecutionStatus::Aborted(err_code) => {
                let mut aborted = Aborted::new();
                aborted.set_aborted_error_code(err_code);
                exec_status.set_aborted(aborted)
            }
            ExecutionStatus::ExecutionStackOverflow => {
                exec_status.set_runtime_status(RuntimeStatus::ExecutionStackOverflow)
            }
            ExecutionStatus::CallStackOverflow => {
                exec_status.set_runtime_status(RuntimeStatus::CallStackOverflow)
            }
        };
        exec_status
    }
}

impl FromProto for ExecutionStatus {
    type ProtoType = crate::proto::vm_errors::ExecutionStatus;

    fn from_proto(mut proto_execution_status: Self::ProtoType) -> Result<ExecutionStatus> {
        use crate::proto::vm_errors::RuntimeStatus as ProtoRuntimeStatus;
        if proto_execution_status.has_runtime_status() {
            match proto_execution_status.get_runtime_status() {
                ProtoRuntimeStatus::Executed => Ok(ExecutionStatus::Executed),
                ProtoRuntimeStatus::OutOfGas => Ok(ExecutionStatus::OutOfGas),
                ProtoRuntimeStatus::ResourceDoesNotExist => {
                    Ok(ExecutionStatus::ResourceDoesNotExist)
                }
                ProtoRuntimeStatus::ResourceAlreadyExists => {
                    Ok(ExecutionStatus::ResourceAlreadyExists)
                }
                ProtoRuntimeStatus::EvictedAccountAccess => {
                    Ok(ExecutionStatus::EvictedAccountAccess)
                }
                ProtoRuntimeStatus::AccountAddressAlreadyExists => {
                    Ok(ExecutionStatus::AccountAddressAlreadyExists)
                }
                ProtoRuntimeStatus::TypeError => Ok(ExecutionStatus::TypeError),
                ProtoRuntimeStatus::MissingData => Ok(ExecutionStatus::MissingData),
                ProtoRuntimeStatus::DataFormatError => Ok(ExecutionStatus::DataFormatError),
                ProtoRuntimeStatus::InvalidData => Ok(ExecutionStatus::InvalidData),
                ProtoRuntimeStatus::RemoteDataError => Ok(ExecutionStatus::RemoteDataError),
                ProtoRuntimeStatus::CannotWriteExistingResource => {
                    Ok(ExecutionStatus::CannotWriteExistingResource)
                }
                ProtoRuntimeStatus::ValueSerializationError => {
                    Ok(ExecutionStatus::ValueSerializationError)
                }
                ProtoRuntimeStatus::ValueDeserializationError => {
                    Ok(ExecutionStatus::ValueDeserializationError)
                }
                ProtoRuntimeStatus::DuplicateModuleName => Ok(ExecutionStatus::DuplicateModuleName),
                ProtoRuntimeStatus::UnknownRuntimeStatus => {
                    bail_err!(DecodingError::UnknownRuntimeStatusEncountered)
                }
                ProtoRuntimeStatus::ExecutionStackOverflow => {
                    Ok(ExecutionStatus::ExecutionStackOverflow)
                }
                ProtoRuntimeStatus::CallStackOverflow => Ok(ExecutionStatus::CallStackOverflow),
            }
        } else if proto_execution_status.has_arithmetic_error() {
            let err = proto_execution_status
                .take_arithmetic_error()
                .get_error_code();
            let from_proto = ArithmeticErrorType::from_proto(err)?;
            Ok(ExecutionStatus::ArithmeticError(from_proto))
        } else if proto_execution_status.has_reference_error() {
            let err = proto_execution_status
                .take_reference_error()
                .get_error_code();
            let from_proto = DynamicReferenceErrorType::from_proto(err)?;
            Ok(ExecutionStatus::DynamicReferenceError(from_proto))
        } else {
            // else it's an assertion error
            let err_code = proto_execution_status.get_aborted();
            Ok(ExecutionStatus::Aborted(err_code.aborted_error_code))
        }
    }
}

impl IntoProto for VMStatus {
    type ProtoType = crate::proto::vm_errors::VMStatus;

    fn into_proto(self) -> Self::ProtoType {
        let mut vm_status = Self::ProtoType::new();
        match self {
            VMStatus::Validation(status) => vm_status.set_validation(status.into_proto()),
            VMStatus::Verification(status_list) => {
                use crate::proto::vm_errors::VMVerificationStatusList as ProtoStatusList;

                let proto_vec: Vec<_> = status_list
                    .into_iter()
                    .map(VMVerificationStatus::into_proto)
                    .collect();
                let mut proto_status_list = ProtoStatusList::new();
                proto_status_list.set_status_list(proto_vec.into());
                vm_status.set_verification(proto_status_list);
            }
            VMStatus::InvariantViolation(err) => {
                vm_status.set_invariant_violation(err.into_proto())
            }
            VMStatus::Deserialization(err) => vm_status.set_deserialization(err.into_proto()),
            VMStatus::Execution(exec_status) => vm_status.set_execution(exec_status.into_proto()),
        };
        vm_status
    }
}

impl FromProto for VMStatus {
    type ProtoType = crate::proto::vm_errors::VMStatus;

    fn from_proto(mut vm_status: Self::ProtoType) -> Result<VMStatus> {
        if vm_status.has_validation() {
            let from_proto = VMValidationStatus::from_proto(vm_status.take_validation())?;
            Ok(VMStatus::Validation(from_proto))
        } else if vm_status.has_verification() {
            let mut proto_status_list = vm_status.take_verification();
            let proto_repeated = proto_status_list.take_status_list();
            let status_list = proto_repeated
                .into_iter()
                .map(VMVerificationStatus::from_proto)
                .collect::<Result<_>>()?;
            Ok(VMStatus::Verification(status_list))
        } else if vm_status.has_invariant_violation() {
            let from_proto =
                VMInvariantViolationError::from_proto(vm_status.get_invariant_violation())?;
            Ok(VMStatus::InvariantViolation(from_proto))
        } else if vm_status.has_deserialization() {
            let from_proto = BinaryError::from_proto(vm_status.get_deserialization())?;
            Ok(VMStatus::Deserialization(from_proto))
        } else if vm_status.has_execution() {
            let from_proto = ExecutionStatus::from_proto(vm_status.take_execution())?;
            Ok(VMStatus::Execution(from_proto))
        } else {
            bail_err!(DecodingError::InvalidVMStatusEncountered)
        }
    }
}
