// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::vm_error::{
    ArithmeticErrorType, BinaryError, DynamicReferenceErrorType, ExecutionStatus,
    VMInvariantViolationError, VMStatus, VMValidationStatus, VMVerificationError,
    VMVerificationStatus,
};
use proptest::prelude::*;
use proptest_helpers::with_stack_size;
use proto_conv::test_helper::assert_protobuf_encode_decode;

proptest! {
    #[test]
    fn vm_validation_status_roundtrip(validation_status in any::<VMValidationStatus>()) {
        assert_protobuf_encode_decode(&validation_status);
    }

    #[test]
    fn vm_verification_error_roundtrip(verification_error in any::<VMVerificationError>()) {
        assert_protobuf_encode_decode(&verification_error);
    }

    #[test]
    fn vm_verification_status_roundtrip(verification_status in any::<VMVerificationStatus>()) {
        assert_protobuf_encode_decode(&verification_status);
    }

    #[test]
    fn vm_invariant_violation_roundtrip(invariant_violation in any::<VMInvariantViolationError>()) {
        assert_protobuf_encode_decode(&invariant_violation);
    }

    #[test]
    fn binary_error_roundtrip(binary_error in any::<BinaryError>()) {
        assert_protobuf_encode_decode(&binary_error);
    }

    #[test]
    fn dynamic_reference_error_roundtrip(dynamic_reference in any::<DynamicReferenceErrorType>()) {
        assert_protobuf_encode_decode(&dynamic_reference);
    }

    #[test]
    fn arithmetic_error_roundtrip(arithmetic_error in any::<ArithmeticErrorType>()) {
        assert_protobuf_encode_decode(&arithmetic_error);
    }

    #[test]
    fn execution_status_roundtrip(execution_status in any::<ExecutionStatus>()) {
        assert_protobuf_encode_decode(&execution_status);
    }
}

#[test]
fn test_vm_status_roundtrip() {
    with_stack_size(4 * 1024 * 1024, || {
        proptest!(|(vm_status in any::<VMStatus>())| {
            assert_protobuf_encode_decode(&vm_status);
        })
    })
    .unwrap();
}
