// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{file_format::SignatureToken, IndexKind, SignatureTokenKind};
use failure::Fail;
use std::{fmt, iter::FromIterator};
use types::{
    account_address::AccountAddress,
    language_storage::ModuleId,
    transaction::TransactionStatus,
    vm_error::{VMStatus, VMValidationStatus, VMVerificationError, VMVerificationStatus},
};

// We may want to eventually move this into the VM runtime since it is a semantic decision that
// need to be made by the VM. But for now, this will reside here.
pub fn vm_result_to_transaction_status<T>(result: &VMResult<T>) -> TransactionStatus {
    // The decision as to whether or not a transaction should be dropped should be able to be
    // determined solely by the VMStatus. This then means that we can audit/verify any decisions
    // made by the VM externally on whether or not to discard or keep the transaction output by
    // inspecting the contained VMStatus.
    let vm_status = vm_status_of_result(result);
    vm_status.into()
}

#[derive(Debug)]
pub struct VMRuntimeError {
    pub loc: Location,
    pub err: VMErrorKind,
}

// TODO: Fill in the details for Locations. Ideally it should be a unique handle into a function and
// a pc.
#[derive(Debug, Default)]
pub struct Location {}

#[derive(Debug, PartialEq)]
pub enum VMErrorKind {
    ArithmeticError,
    TypeError,
    Aborted(u64),
    OutOfGasError,
    GlobalRefAlreadyReleased,
    MissingReleaseRef,
    GlobalAlreadyBorrowed,
    MissingData,
    DuplicateModuleName,
    DataFormatError,
    InvalidData,
    RemoteDataError,
    CannotWriteExistingResource,
    ValueSerializerError,
    ValueDeserializerError,
    CodeSerializerError(BinaryError),
    CodeDeserializerError(BinaryError),
    Verification(Vec<VerificationStatus>),
    ExecutionStackOverflow,
    CallStackOverflow,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum VerificationStatus {
    /// A verification error was detected in a transaction script.
    Script(VerificationError),
    /// A verification error was detected in a module. The first element is the index of the module
    /// in the transaction.
    Module(u16, VerificationError),
    /// A verification error was detected in a dependency of a module.
    Dependency(ModuleId, VerificationError),
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct VerificationError {
    /// Where the violation occurred.
    pub kind: IndexKind,
    /// The index where the violation occurred.
    pub idx: usize,
    /// The actual violation that occurred.
    pub err: VMStaticViolation,
}

impl fmt::Display for VerificationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "at '{}' index {}: {}", self.kind, self.idx, self.err)
    }
}

#[derive(Clone, Debug, Eq, Fail, Ord, PartialEq, PartialOrd)]
pub enum VMStaticViolation {
    #[fail(
        display = "Index out of bounds for '{}' (expected 0..{}, found {})",
        _0, _1, _2
    )]
    IndexOutOfBounds(IndexKind, usize, usize),

    #[fail(
        display = "Index out of bounds for '{}' at code offset {} (expected 0..{}, found {})",
        _0, _1, _2, _3
    )]
    CodeUnitIndexOutOfBounds(IndexKind, usize, usize, usize),

    #[fail(
        display = "Range out of bounds for '{}' (expected 0..{}, found {}..{})",
        _0, _1, _2, _3
    )]
    RangeOutOfBounds(IndexKind, usize, usize, usize),

    #[fail(display = "Module must have at least one module handle")]
    NoModuleHandles,

    #[fail(display = "Module address does not match sender")]
    ModuleAddressDoesNotMatchSender,

    #[fail(
        display = "Invalid signature token {:?}: '{} of {}' is invalid",
        _0, _1, _2
    )]
    InvalidSignatureToken(SignatureToken, SignatureTokenKind, SignatureTokenKind),

    #[fail(display = "Duplicate element")]
    DuplicateElement,

    #[fail(display = "Invalid module handle")]
    InvalidModuleHandle,

    #[fail(display = "Unimplemented struct or function handle")]
    UnimplementedHandle,

    #[fail(display = "Inconsistent fields in struct definition")]
    InconsistentFields,

    #[fail(display = "Unused fields")]
    UnusedFields,

    #[fail(display = "Field definition has invalid type: {} ({:?})", _1, _0)]
    InvalidFieldDefReference(SignatureToken, SignatureTokenKind),

    #[fail(display = "Recursive struct definition")]
    RecursiveStructDef,

    #[fail(display = "Resource field in non-resource struct")]
    InvalidResourceField,

    #[fail(display = "Invalid fall through")]
    InvalidFallThrough,

    #[fail(display = "Failure to perform join at block {}", _0)]
    JoinFailure(usize),

    #[fail(display = "Negative stack size at block {} and offset {}", _0, _1)]
    NegativeStackSizeInsideBlock(usize, usize),

    #[fail(display = "Positive stack size at end of block {}", _0)]
    PositiveStackSizeAtBlockEnd(usize),

    #[fail(display = "Invalid signature for main function in script")]
    InvalidMainFunctionSignature,

    #[fail(display = "Lookup of struct or function handle failed in module dependency")]
    LookupFailed,

    #[fail(display = "Visibility mismatch for function handle in module dependency")]
    VisibilityMismatch,

    #[fail(display = "Type of function in module dependency could not be resolved")]
    TypeResolutionFailure,

    #[fail(display = "Type mismatch for struct or function handle in module dependency")]
    TypeMismatch,

    #[fail(display = "Missing module dependency")]
    MissingDependency,

    #[fail(display = "Unable to verify Pop at offset {}", _0)]
    PopReferenceError(usize),

    #[fail(display = "Unable to verify Pop at offset {}", _0)]
    PopResourceError(usize),

    #[fail(display = "Unable to verify ReleaseRef at offset {}", _0)]
    ReleaseRefTypeMismatchError(usize),

    #[fail(display = "Unable to verify BrTrue/BrFalse at offset {}", _0)]
    BrTypeMismatchError(usize),

    #[fail(display = "Unable to verify Abort at offset {}", _0)]
    AbortTypeMismatchError(usize),

    #[fail(display = "Unable to verify StLoc at offset {}", _0)]
    StLocTypeMismatchError(usize),

    #[fail(display = "Unable to verify StLoc at offset {}", _0)]
    StLocUnsafeToDestroyError(usize),

    #[fail(display = "Unable to verify Ret at offset {}", _0)]
    RetUnsafeToDestroyError(usize),

    #[fail(display = "Unable to verify Ret at offset {}", _0)]
    RetTypeMismatchError(usize),

    #[fail(display = "Unable to verify FreezeRef at offset {}", _0)]
    FreezeRefTypeMismatchError(usize),

    #[fail(display = "Unable to verify FreezeRef at offset {}", _0)]
    FreezeRefExistsMutableBorrowError(usize),

    #[fail(display = "Unable to verify BorrowField at offset {}", _0)]
    BorrowFieldTypeMismatchError(usize),

    #[fail(display = "Unable to verify BorrowField at offset {}", _0)]
    BorrowFieldBadFieldError(usize),

    #[fail(display = "Unable to verify BorrowField at offset {}", _0)]
    BorrowFieldExistsMutableBorrowError(usize),

    #[fail(display = "Unable to verify CopyLoc at offset {}", _0)]
    CopyLocUnavailableError(usize),

    #[fail(display = "Unable to verify CopyLoc at offset {}", _0)]
    CopyLocResourceError(usize),

    #[fail(display = "Unable to verify CopyLoc at offset {}", _0)]
    CopyLocExistsBorrowError(usize),

    #[fail(display = "Unable to verify MoveLoc at offset {}", _0)]
    MoveLocUnavailableError(usize),

    #[fail(display = "Unable to verify MoveLoc at offset {}", _0)]
    MoveLocExistsBorrowError(usize),

    #[fail(display = "Unable to verify BorrowLoc at offset {}", _0)]
    BorrowLocReferenceError(usize),

    #[fail(display = "Unable to verify BorrowLoc at offset {}", _0)]
    BorrowLocUnavailableError(usize),

    #[fail(display = "Unable to verify BorrowLoc at offset {}", _0)]
    BorrowLocExistsBorrowError(usize),

    #[fail(display = "Unable to verify Call at offset {}", _0)]
    CallTypeMismatchError(usize),

    #[fail(display = "Unable to verify BorrowLoc at offset {}", _0)]
    CallBorrowedMutableReferenceError(usize),

    #[fail(display = "Unable to verify Pack at offset {}", _0)]
    PackTypeMismatchError(usize),

    #[fail(display = "Unable to verify Unpack at offset {}", _0)]
    UnpackTypeMismatchError(usize),

    #[fail(display = "Unable to verify ReadRef at offset {}", _0)]
    ReadRefTypeMismatchError(usize),

    #[fail(display = "Unable to verify ReadRef at offset {}", _0)]
    ReadRefResourceError(usize),

    #[fail(display = "Unable to verify ReadRef at offset {}", _0)]
    ReadRefExistsMutableBorrowError(usize),

    #[fail(display = "Unable to verify WriteRef at offset {}", _0)]
    WriteRefTypeMismatchError(usize),

    #[fail(display = "Unable to verify WriteRef at offset {}", _0)]
    WriteRefResourceError(usize),

    #[fail(display = "Unable to verify WriteRef at offset {}", _0)]
    WriteRefExistsBorrowError(usize),

    #[fail(display = "Unable to verify WriteRef at offset {}", _0)]
    WriteRefNoMutableReferenceError(usize),

    #[fail(display = "Unable to verify integer operation at offset {}", _0)]
    IntegerOpTypeMismatchError(usize),

    #[fail(display = "Unable to verify boolean operation at offset {}", _0)]
    BooleanOpTypeMismatchError(usize),

    #[fail(display = "Unable to verify equality operation at offset {}", _0)]
    EqualityOpTypeMismatchError(usize),

    #[fail(display = "Unable to verify Exists at offset {}", _0)]
    ExistsResourceTypeMismatchError(usize),

    #[fail(display = "Unable to verify Exists at offset {}", _0)]
    ExistsNoResourceError(usize),

    #[fail(display = "Unable to verify BorrowGlobal at offset {}", _0)]
    BorrowGlobalTypeMismatchError(usize),

    #[fail(display = "Unable to verify BorrowGlobal at offset {}", _0)]
    BorrowGlobalNoResourceError(usize),

    #[fail(display = "Unable to verify MoveFrom at offset {}", _0)]
    MoveFromTypeMismatchError(usize),

    #[fail(display = "Unable to verify MoveFrom at offset {}", _0)]
    MoveFromNoResourceError(usize),

    #[fail(display = "Unable to verify MoveToSender at offset {}", _0)]
    MoveToSenderTypeMismatchError(usize),

    #[fail(display = "Unable to verify MoveToSender at offset {}", _0)]
    MoveToSenderNoResourceError(usize),

    #[fail(display = "Unable to verify MoveToSender at offset {}", _0)]
    CreateAccountTypeMismatchError(usize),

    #[fail(display = "Illegal global operation at offset {}", _0)]
    GlobalReferenceError(usize),

    #[fail(display = "Missing acquires resource annotaiton at offset {}", _0)]
    MissingAcquiresResourceAnnotationError(usize),

    #[fail(display = "Extraneous acquires resource annotaiton")]
    ExtraneousAcquiresResourceAnnotationError,

    #[fail(display = "Duplicate acquires resource annotaiton")]
    DuplicateAcquiresResourceAnnotationError,

    #[fail(
        display = "Duplicate acquires resource annotaiton. The struct is not a nominal resource."
    )]
    InvalidAcquiresResourceAnnotationError,

    #[fail(display = "The kind of the type actual does not satisfy the constraint.")]
    ConstraintKindMismatch,

    #[fail(display = "Expected {} type actuals got {}", _0, _1)]
    NumberOfTypeActualsMismatch(usize, usize),
}

#[derive(Clone, Debug, Eq, Fail, Ord, PartialEq, PartialOrd)]
pub enum VMInvariantViolation {
    #[fail(
        display = "Index out of bounds for '{}' (expected 0..{}, found {})",
        _0, _1, _2
    )]
    IndexOutOfBounds(IndexKind, usize, usize),
    #[fail(
        display = "Range out of bounds for '{}' (expected 0..{}, found {}..{})",
        _0, _1, _2, _3
    )]
    RangeOutOfBounds(IndexKind, usize, usize, usize),
    #[fail(display = "Try to pop an empty value stack")]
    EmptyValueStack,
    #[fail(display = "Try to pop an empty call stack")]
    EmptyCallStack,
    #[fail(display = "Program Counter Overflows")]
    ProgramCounterOverflow,
    #[fail(display = "Linker can't find the destination code")]
    LinkerError,
    #[fail(display = "Owned value has multiple references")]
    LocalReferenceError,
    #[fail(display = "Failed to get response from storage")]
    StorageError,
    #[fail(display = "Internal runtime type error due to incorrect bytecode verification")]
    InternalTypeError,
    #[fail(display = "Event key is not 32 byte")]
    EventKeyMismatch,
}

/// Error codes that can be emitted by the prologue. These have special significance to the VM when
/// they are raised during the prologue. However, they can also be raised by user code during
/// execution of a transaction script. They have no significance to the VM in that case.
pub const EBAD_SIGNATURE: u64 = 1; // signature on transaction is invalid
pub const EBAD_ACCOUNT_AUTHENTICATION_KEY: u64 = 2; // auth key in transaction is invalid
pub const ESEQUENCE_NUMBER_TOO_OLD: u64 = 3; // transaction sequence number is too old
pub const ESEQUENCE_NUMBER_TOO_NEW: u64 = 4; // transaction sequence number is too new
pub const EACCOUNT_DOES_NOT_EXIST: u64 = 5; // transaction sender's account does not exist
pub const ECANT_PAY_GAS_DEPOSIT: u64 = 6; // insufficient balance to pay for gas deposit

/// Generic error codes. These codes don't have any special meaning for the VM, but they are useful
/// conventions for debugging
pub const EINSUFFICIENT_BALANCE: u64 = 10; // withdrawing more than an account contains
pub const EASSERT_ERROR: u64 = 42; // catch-all error code for assert failures

pub type VMRuntimeResult<T> = ::std::result::Result<T, VMRuntimeError>;
pub type VMResult<T> = ::std::result::Result<VMRuntimeResult<T>, VMInvariantViolation>;

impl Location {
    pub fn new() -> Self {
        Location {}
    }
}

pub type BinaryLoaderResult<T> = ::std::result::Result<T, BinaryError>;

// TODO: This is an initial set of errors that needs to be expanded.
//       Also it's not clear whether we should fold this into other error types
#[derive(Clone, Debug, Eq, Fail, PartialEq)]
pub enum BinaryError {
    #[fail(display = "Malformed binary")]
    Malformed,
    #[fail(display = "Bad magic")]
    BadMagic,
    #[fail(display = "Unknown version")]
    UnknownVersion,
    #[fail(display = "Unknown table type")]
    UnknownTableType,
    #[fail(display = "Unknown signature type")]
    UnknownSignatureType,
    #[fail(display = "Unexpected signature type")]
    UnexpectedSignatureType,
    #[fail(display = "Unknown serialized type")]
    UnknownSerializedType,
    #[fail(display = "Unknown opcode")]
    UnknownOpcode,
    #[fail(display = "Wrong table header format (offset or count)")]
    BadHeaderTable,
    #[fail(display = "Duplicate table type")]
    DuplicateTable,
}

#[macro_export]
macro_rules! try_runtime {
    ($e:expr) => {
        match $e {
            Ok(Ok(t)) => t,
            Ok(Err(e)) => return Ok(Err(e)),
            Err(e) => return Err(e),
        }
    };
}

#[macro_export]
macro_rules! assert_ok {
    ($e:expr) => {
        assert!(match $e {
            Ok(Ok(t)) => true,
            Ok(Err(e)) => {
                println!("Unexpected Runtime Error: {:?}", e);
                false
            }
            Err(e) => {
                println!("Unexpected ICE: {:?}", e);
                false
            }
        })
    };
}

////////////////////////////////////////////////////////////////////////////
/// Conversion functions from internal VM statuses into external VM statuses
////////////////////////////////////////////////////////////////////////////

pub fn to_vm_status<'a, T, E>(result: &'a ::std::result::Result<T, E>) -> VMStatus
where
    VMStatus: From<&'a E>,
    E: 'a,
{
    use types::vm_error::ExecutionStatus;
    match result {
        Ok(_) => VMStatus::Execution(ExecutionStatus::Executed),
        Err(err) => err.into(),
    }
}

pub fn vm_status_of_result<T>(result: &VMResult<T>) -> VMStatus {
    match result {
        Ok(runtime_result) => to_vm_status(runtime_result),
        Err(err) => err.into(),
    }
}

// FUTURE: At the moment we can't pass transaction metadata or the signed transaction due to
// restrictions in the two places that this function is called. We therefore just pass through what
// we need at the moment---the sender address---but we may want/need to pass more data later on.
pub fn convert_prologue_runtime_error(
    err: &VMRuntimeError,
    txn_sender: &AccountAddress,
) -> VMStatus {
    use VMErrorKind::*;
    match err.err {
        // Invalid authentication key
        Aborted(EBAD_ACCOUNT_AUTHENTICATION_KEY) => {
            VMStatus::Validation(VMValidationStatus::InvalidAuthKey)
        }
        // Sequence number too old
        Aborted(ESEQUENCE_NUMBER_TOO_OLD) => {
            VMStatus::Validation(VMValidationStatus::SequenceNumberTooOld)
        }
        // Sequence number too new
        Aborted(ESEQUENCE_NUMBER_TOO_NEW) => {
            VMStatus::Validation(VMValidationStatus::SequenceNumberTooNew)
        }
        // Sequence number too new
        Aborted(EACCOUNT_DOES_NOT_EXIST) => {
            let error_msg = format!("sender address: {}", txn_sender);
            VMStatus::Validation(VMValidationStatus::SendingAccountDoesNotExist(error_msg))
        }
        // Can't pay for transaction gas deposit/fee
        Aborted(ECANT_PAY_GAS_DEPOSIT) => {
            VMStatus::Validation(VMValidationStatus::InsufficientBalanceForTransactionFee)
        }
        _ => err.into(),
    }
}

///////////////////////////////////////////////////////////////////
/// Conversion from internal VM statuses into external VM statuses
///////////////////////////////////////////////////////////////////

impl From<&BinaryError> for VMStatus {
    fn from(error: &BinaryError) -> Self {
        use types::vm_error::BinaryError as VMBinaryError;
        let bin_err = match error {
            BinaryError::Malformed => VMBinaryError::Malformed,
            BinaryError::BadMagic => VMBinaryError::BadMagic,
            BinaryError::UnknownVersion => VMBinaryError::UnknownVersion,
            BinaryError::UnknownTableType => VMBinaryError::UnknownTableType,
            BinaryError::UnknownSignatureType => VMBinaryError::UnknownSignatureType,
            BinaryError::UnknownSerializedType => VMBinaryError::UnknownSerializedType,
            BinaryError::UnknownOpcode => VMBinaryError::UnknownOpcode,
            BinaryError::BadHeaderTable => VMBinaryError::BadHeaderTable,
            BinaryError::DuplicateTable => VMBinaryError::DuplicateTable,
            BinaryError::UnexpectedSignatureType => VMBinaryError::UnexpectedSignatureType,
        };
        VMStatus::Deserialization(bin_err)
    }
}

impl From<&VMInvariantViolation> for VMStatus {
    fn from(error: &VMInvariantViolation) -> Self {
        use types::vm_error::VMInvariantViolationError;
        let err = match error {
            VMInvariantViolation::IndexOutOfBounds(_, _, _) => {
                VMInvariantViolationError::OutOfBoundsIndex
            }
            VMInvariantViolation::RangeOutOfBounds(_, _, _, _) => {
                VMInvariantViolationError::OutOfBoundsRange
            }
            VMInvariantViolation::EmptyValueStack => VMInvariantViolationError::EmptyValueStack,
            VMInvariantViolation::EmptyCallStack => VMInvariantViolationError::EmptyCallStack,
            VMInvariantViolation::ProgramCounterOverflow => VMInvariantViolationError::PCOverflow,
            VMInvariantViolation::LinkerError => VMInvariantViolationError::LinkerError,
            VMInvariantViolation::LocalReferenceError => {
                VMInvariantViolationError::LocalReferenceError
            }
            VMInvariantViolation::StorageError => VMInvariantViolationError::StorageError,
            VMInvariantViolation::InternalTypeError => VMInvariantViolationError::InternalTypeError,
            VMInvariantViolation::EventKeyMismatch => VMInvariantViolationError::EventKeyMismatch,
        };
        VMStatus::InvariantViolation(err)
    }
}

impl From<&VerificationError> for VMVerificationError {
    fn from(error: &VerificationError) -> Self {
        let message = format!("{}", error);
        match error.err {
            VMStaticViolation::IndexOutOfBounds(_, _, _) => {
                VMVerificationError::IndexOutOfBounds(message)
            }
            VMStaticViolation::CodeUnitIndexOutOfBounds(_, _, _, _) => {
                VMVerificationError::CodeUnitIndexOutOfBounds(message)
            }
            VMStaticViolation::RangeOutOfBounds(_, _, _, _) => {
                VMVerificationError::RangeOutOfBounds(message)
            }
            VMStaticViolation::NoModuleHandles => VMVerificationError::NoModuleHandles(message),
            VMStaticViolation::ModuleAddressDoesNotMatchSender => {
                VMVerificationError::ModuleAddressDoesNotMatchSender(message)
            }
            VMStaticViolation::InvalidSignatureToken(_, _, _) => {
                VMVerificationError::InvalidSignatureToken(message)
            }
            VMStaticViolation::DuplicateElement => VMVerificationError::DuplicateElement(message),
            VMStaticViolation::InvalidModuleHandle => {
                VMVerificationError::InvalidModuleHandle(message)
            }
            VMStaticViolation::UnimplementedHandle => {
                VMVerificationError::UnimplementedHandle(message)
            }
            VMStaticViolation::InconsistentFields => {
                VMVerificationError::InconsistentFields(message)
            }
            VMStaticViolation::UnusedFields => VMVerificationError::UnusedFields(message),
            VMStaticViolation::InvalidFieldDefReference(_, _) => {
                VMVerificationError::InvalidFieldDefReference(message)
            }
            VMStaticViolation::RecursiveStructDef => {
                VMVerificationError::RecursiveStructDefinition(message)
            }
            VMStaticViolation::InvalidResourceField => {
                VMVerificationError::InvalidResourceField(message)
            }
            VMStaticViolation::InvalidFallThrough => {
                VMVerificationError::InvalidFallThrough(message)
            }
            VMStaticViolation::JoinFailure(_) => VMVerificationError::JoinFailure(message),
            VMStaticViolation::NegativeStackSizeInsideBlock(_, _) => {
                VMVerificationError::NegativeStackSizeWithinBlock(message)
            }
            VMStaticViolation::PositiveStackSizeAtBlockEnd(_) => {
                VMVerificationError::UnbalancedStack(message)
            }
            VMStaticViolation::InvalidMainFunctionSignature => {
                VMVerificationError::InvalidMainFunctionSignature(message)
            }
            VMStaticViolation::LookupFailed => VMVerificationError::LookupFailed(message),
            VMStaticViolation::VisibilityMismatch => {
                VMVerificationError::VisibilityMismatch(message)
            }
            VMStaticViolation::TypeResolutionFailure => {
                VMVerificationError::TypeResolutionFailure(message)
            }
            VMStaticViolation::TypeMismatch => VMVerificationError::TypeMismatch(message),
            VMStaticViolation::MissingDependency => VMVerificationError::MissingDependency(message),
            VMStaticViolation::PopReferenceError(_) => {
                VMVerificationError::PopReferenceError(message)
            }
            VMStaticViolation::PopResourceError(_) => {
                VMVerificationError::PopResourceError(message)
            }
            VMStaticViolation::ReleaseRefTypeMismatchError(_) => {
                VMVerificationError::ReleaseRefTypeMismatchError(message)
            }
            VMStaticViolation::BrTypeMismatchError(_) => {
                VMVerificationError::BrTypeMismatchError(message)
            }
            VMStaticViolation::AbortTypeMismatchError(_) => {
                VMVerificationError::AbortTypeMismatchError(message)
            }
            VMStaticViolation::StLocTypeMismatchError(_) => {
                VMVerificationError::StLocTypeMismatchError(message)
            }
            VMStaticViolation::StLocUnsafeToDestroyError(_) => {
                VMVerificationError::StLocUnsafeToDestroyError(message)
            }
            VMStaticViolation::RetUnsafeToDestroyError(_) => {
                VMVerificationError::RetUnsafeToDestroyError(message)
            }
            VMStaticViolation::RetTypeMismatchError(_) => {
                VMVerificationError::RetTypeMismatchError(message)
            }
            VMStaticViolation::FreezeRefTypeMismatchError(_) => {
                VMVerificationError::FreezeRefTypeMismatchError(message)
            }
            VMStaticViolation::FreezeRefExistsMutableBorrowError(_) => {
                VMVerificationError::FreezeRefExistsMutableBorrowError(message)
            }
            VMStaticViolation::BorrowFieldTypeMismatchError(_) => {
                VMVerificationError::BorrowFieldTypeMismatchError(message)
            }
            VMStaticViolation::BorrowFieldBadFieldError(_) => {
                VMVerificationError::BorrowFieldBadFieldError(message)
            }
            VMStaticViolation::BorrowFieldExistsMutableBorrowError(_) => {
                VMVerificationError::BorrowFieldExistsMutableBorrowError(message)
            }
            VMStaticViolation::CopyLocUnavailableError(_) => {
                VMVerificationError::CopyLocUnavailableError(message)
            }
            VMStaticViolation::CopyLocResourceError(_) => {
                VMVerificationError::CopyLocResourceError(message)
            }
            VMStaticViolation::CopyLocExistsBorrowError(_) => {
                VMVerificationError::CopyLocExistsBorrowError(message)
            }
            VMStaticViolation::MoveLocUnavailableError(_) => {
                VMVerificationError::MoveLocUnavailableError(message)
            }
            VMStaticViolation::MoveLocExistsBorrowError(_) => {
                VMVerificationError::MoveLocExistsBorrowError(message)
            }
            VMStaticViolation::BorrowLocReferenceError(_) => {
                VMVerificationError::BorrowLocReferenceError(message)
            }
            VMStaticViolation::BorrowLocUnavailableError(_) => {
                VMVerificationError::BorrowLocUnavailableError(message)
            }
            VMStaticViolation::BorrowLocExistsBorrowError(_) => {
                VMVerificationError::BorrowLocExistsBorrowError(message)
            }
            VMStaticViolation::CallTypeMismatchError(_) => {
                VMVerificationError::CallTypeMismatchError(message)
            }
            VMStaticViolation::CallBorrowedMutableReferenceError(_) => {
                VMVerificationError::CallBorrowedMutableReferenceError(message)
            }
            VMStaticViolation::PackTypeMismatchError(_) => {
                VMVerificationError::PackTypeMismatchError(message)
            }
            VMStaticViolation::UnpackTypeMismatchError(_) => {
                VMVerificationError::UnpackTypeMismatchError(message)
            }
            VMStaticViolation::ReadRefTypeMismatchError(_) => {
                VMVerificationError::ReadRefTypeMismatchError(message)
            }
            VMStaticViolation::ReadRefResourceError(_) => {
                VMVerificationError::ReadRefResourceError(message)
            }
            VMStaticViolation::ReadRefExistsMutableBorrowError(_) => {
                VMVerificationError::ReadRefExistsMutableBorrowError(message)
            }
            VMStaticViolation::WriteRefTypeMismatchError(_) => {
                VMVerificationError::WriteRefTypeMismatchError(message)
            }
            VMStaticViolation::WriteRefResourceError(_) => {
                VMVerificationError::WriteRefResourceError(message)
            }
            VMStaticViolation::WriteRefExistsBorrowError(_) => {
                VMVerificationError::WriteRefExistsBorrowError(message)
            }
            VMStaticViolation::WriteRefNoMutableReferenceError(_) => {
                VMVerificationError::WriteRefNoMutableReferenceError(message)
            }
            VMStaticViolation::IntegerOpTypeMismatchError(_) => {
                VMVerificationError::IntegerOpTypeMismatchError(message)
            }
            VMStaticViolation::BooleanOpTypeMismatchError(_) => {
                VMVerificationError::BooleanOpTypeMismatchError(message)
            }
            VMStaticViolation::EqualityOpTypeMismatchError(_) => {
                VMVerificationError::EqualityOpTypeMismatchError(message)
            }
            VMStaticViolation::ExistsResourceTypeMismatchError(_) => {
                VMVerificationError::ExistsResourceTypeMismatchError(message)
            }
            VMStaticViolation::ExistsNoResourceError(_) => {
                VMVerificationError::ExistsNoResourceError(message)
            }
            VMStaticViolation::BorrowGlobalTypeMismatchError(_) => {
                VMVerificationError::BorrowGlobalTypeMismatchError(message)
            }
            VMStaticViolation::BorrowGlobalNoResourceError(_) => {
                VMVerificationError::BorrowGlobalNoResourceError(message)
            }
            VMStaticViolation::MoveFromTypeMismatchError(_) => {
                VMVerificationError::MoveFromTypeMismatchError(message)
            }
            VMStaticViolation::MoveFromNoResourceError(_) => {
                VMVerificationError::MoveFromNoResourceError(message)
            }
            VMStaticViolation::MoveToSenderTypeMismatchError(_) => {
                VMVerificationError::MoveToSenderTypeMismatchError(message)
            }
            VMStaticViolation::MoveToSenderNoResourceError(_) => {
                VMVerificationError::MoveToSenderNoResourceError(message)
            }
            VMStaticViolation::CreateAccountTypeMismatchError(_) => {
                VMVerificationError::CreateAccountTypeMismatchError(message)
            }
            VMStaticViolation::GlobalReferenceError(_) => {
                VMVerificationError::GlobalReferenceError(message)
            }
            VMStaticViolation::MissingAcquiresResourceAnnotationError(_) => {
                VMVerificationError::MissingAcquiresResourceAnnotationError(message)
            }
            VMStaticViolation::ExtraneousAcquiresResourceAnnotationError => {
                VMVerificationError::ExtraneousAcquiresResourceAnnotationError(message)
            }
            VMStaticViolation::InvalidAcquiresResourceAnnotationError => {
                VMVerificationError::InvalidAcquiresResourceAnnotationError(message)
            }
            VMStaticViolation::DuplicateAcquiresResourceAnnotationError => {
                VMVerificationError::DuplicateAcquiresResourceAnnotationError(message)
            }
            VMStaticViolation::ConstraintKindMismatch => {
                VMVerificationError::ConstraintKindMismatch(message)
            }
            VMStaticViolation::NumberOfTypeActualsMismatch(_, _) => {
                VMVerificationError::NumberOfTypeActualsMismatch(message)
            }
        }
    }
}

impl From<&VerificationStatus> for VMVerificationStatus {
    fn from(status: &VerificationStatus) -> Self {
        match status {
            VerificationStatus::Script(err) => VMVerificationStatus::Script(err.into()),
            VerificationStatus::Module(module_idx, err) => {
                VMVerificationStatus::Module(*module_idx, err.into())
            }
            VerificationStatus::Dependency(dependency_id, err) => {
                VMVerificationStatus::Dependency(dependency_id.clone(), err.into())
            }
        }
    }
}

impl<'a> FromIterator<&'a VerificationStatus> for VMStatus {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = &'a VerificationStatus>,
    {
        let status_list = iter.into_iter().map(VMVerificationStatus::from).collect();
        VMStatus::Verification(status_list)
    }
}

impl From<&VMErrorKind> for VMStatus {
    fn from(error: &VMErrorKind) -> Self {
        use types::vm_error::{ArithmeticErrorType, DynamicReferenceErrorType, ExecutionStatus};
        let err = match error {
            VMErrorKind::ArithmeticError => {
                ExecutionStatus::ArithmeticError(ArithmeticErrorType::Underflow)
            }
            VMErrorKind::Aborted(err_code) => ExecutionStatus::Aborted(*err_code),
            VMErrorKind::OutOfGasError => ExecutionStatus::OutOfGas,
            VMErrorKind::TypeError => ExecutionStatus::TypeError,
            VMErrorKind::GlobalRefAlreadyReleased => ExecutionStatus::DynamicReferenceError(
                DynamicReferenceErrorType::GlobalRefAlreadyReleased,
            ),
            VMErrorKind::MissingReleaseRef => {
                ExecutionStatus::DynamicReferenceError(DynamicReferenceErrorType::MissingReleaseRef)
            }
            VMErrorKind::GlobalAlreadyBorrowed => ExecutionStatus::DynamicReferenceError(
                DynamicReferenceErrorType::GlobalAlreadyBorrowed,
            ),
            VMErrorKind::MissingData => ExecutionStatus::MissingData,
            VMErrorKind::DataFormatError => ExecutionStatus::DataFormatError,
            VMErrorKind::InvalidData => ExecutionStatus::InvalidData,
            VMErrorKind::RemoteDataError => ExecutionStatus::RemoteDataError,
            VMErrorKind::CannotWriteExistingResource => {
                ExecutionStatus::CannotWriteExistingResource
            }
            VMErrorKind::ValueSerializerError => ExecutionStatus::ValueSerializationError,
            VMErrorKind::ValueDeserializerError => ExecutionStatus::ValueDeserializationError,
            VMErrorKind::DuplicateModuleName => ExecutionStatus::DuplicateModuleName,
            // The below errors already have top-level VMStatus variants associated with them, so
            // return those.
            VMErrorKind::CodeSerializerError(err) => return VMStatus::from(err),
            VMErrorKind::CodeDeserializerError(err) => return VMStatus::from(err),
            VMErrorKind::Verification(statuses) => return statuses.iter().collect(),
            VMErrorKind::ExecutionStackOverflow => ExecutionStatus::ExecutionStackOverflow,
            VMErrorKind::CallStackOverflow => ExecutionStatus::CallStackOverflow,
        };
        VMStatus::Execution(err)
    }
}

impl From<&VMRuntimeError> for VMStatus {
    fn from(error: &VMRuntimeError) -> Self {
        VMStatus::from(&error.err)
    }
}
