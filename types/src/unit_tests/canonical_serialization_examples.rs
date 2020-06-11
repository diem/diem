// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! These tests verify the behavior of LCS against some known test vectors with various types.

use crate::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_config::LBR_NAME,
    transaction::{ChangeSet, RawTransaction, Script, TransactionArgument, TransactionPayload},
    write_set::{WriteOp, WriteSet, WriteSetMut},
};
use lcs::to_bytes;
use std::time::Duration;

#[test]
fn test_access_path_canonical_serialization_example() {
    let account_address = AccountAddress::new([
        0x9a, 0x1a, 0xd0, 0x97, 0x42, 0xd1, 0xff, 0xc6, 0x2e, 0x65, 0x9e, 0x9a, 0x77, 0x97, 0x80,
        0x8b,
    ]);
    let input = AccessPath::new(
        account_address,
        vec![
            0x01, 0x21, 0x7d, 0xa6, 0xc6, 0xb3, 0xe1, 0x9f, 0x18, 0x25, 0xcf, 0xb2, 0x67, 0x6d,
            0xae, 0xcc, 0xe3, 0xbf, 0x3d, 0xe0, 0x3c, 0xf2, 0x66, 0x47, 0xc7, 0x8d, 0xf0, 0x0b,
            0x37, 0x1b, 0x25, 0xcc, 0x97,
        ],
    );

    let expected_output = vec![
        0x9A, 0x1A, 0xD0, 0x97, 0x42, 0xD1, 0xFF, 0xC6, 0x2E, 0x65, 0x9E, 0x9A, 0x77, 0x97, 0x80,
        0x8B, 0x21, 0x01, 0x21, 0x7D, 0xA6, 0xC6, 0xB3, 0xE1, 0x9F, 0x18, 0x25, 0xCF, 0xB2, 0x67,
        0x6D, 0xAE, 0xCC, 0xE3, 0xBF, 0x3D, 0xE0, 0x3C, 0xF2, 0x66, 0x47, 0xC7, 0x8D, 0xF0, 0x0B,
        0x37, 0x1B, 0x25, 0xCC, 0x97,
    ];

    let actual_output = to_bytes(&input).unwrap();
    assert_eq!(expected_output, actual_output);
}

#[test]
fn test_account_address_canonical_serialization_example() {
    let input = AccountAddress::new([
        0xca, 0x82, 0x0b, 0xf9, 0x30, 0x5e, 0xb9, 0x7d, 0x0d, 0x78, 0x4f, 0x71, 0xb3, 0x95, 0x54,
        0x57,
    ]);

    let expected_output: Vec<u8> = vec![
        0xCA, 0x82, 0x0B, 0xF9, 0x30, 0x5E, 0xB9, 0x7D, 0x0D, 0x78, 0x4F, 0x71, 0xB3, 0x95, 0x54,
        0x57,
    ];

    let actual_output = to_bytes(&input).unwrap();
    assert_eq!(expected_output, actual_output);
}

#[test]
fn test_program_canonical_serialization_example() {
    let input = get_common_program();

    let expected_output: Vec<u8> = vec![
        0x04, 0x6D, 0x6F, 0x76, 0x65, 0x00, 0x01, 0x01, 0xEF, 0xBE, 0xAD, 0xDE, 0x0D, 0xD0, 0xFE,
        0xCA,
    ];

    let actual_output = to_bytes(&input).unwrap();
    assert_eq!(expected_output, actual_output);
}

#[test]
fn test_raw_transaction_with_a_program_canonical_serialization_example() {
    let input = RawTransaction::new_script(
        AccountAddress::new([
            0x3a, 0x24, 0xa6, 0x1e, 0x05, 0xd1, 0x29, 0xca, 0xce, 0x9e, 0x0e, 0xfc, 0x8b, 0xc9,
            0xe3, 0x38,
        ]),
        32,
        get_common_program(),
        10000,
        20000,
        LBR_NAME.to_owned(),
        Duration::from_secs(86400),
    );

    let expected_output = vec![
        58, 36, 166, 30, 5, 209, 41, 202, 206, 158, 14, 252, 139, 201, 227, 56, 32, 0, 0, 0, 0, 0,
        0, 0, 1, 4, 109, 111, 118, 101, 0, 1, 1, 239, 190, 173, 222, 13, 208, 254, 202, 16, 39, 0,
        0, 0, 0, 0, 0, 32, 78, 0, 0, 0, 0, 0, 0, 3, 76, 66, 82, 128, 81, 1, 0, 0, 0, 0, 0,
    ];

    let actual_output = to_bytes(&input).unwrap();
    assert_eq!(expected_output, actual_output);
}

#[test]
fn test_raw_transaction_with_a_write_set_canonical_serialization_example() {
    let input = RawTransaction::new_write_set(
        AccountAddress::new([
            0xc3, 0x39, 0x8a, 0x59, 0x9a, 0x6f, 0x3b, 0x9f, 0x30, 0xb6, 0x35, 0xaf, 0x29, 0xf2,
            0xba, 0x04,
        ]),
        32,
        get_common_write_set(),
    );

    let expected_output = vec![
        195, 57, 138, 89, 154, 111, 59, 159, 48, 182, 53, 175, 41, 242, 186, 4, 32, 0, 0, 0, 0, 0,
        0, 0, 0, 2, 167, 29, 118, 250, 162, 210, 213, 195, 34, 78, 195, 212, 29, 235, 41, 57, 33,
        1, 33, 125, 166, 198, 179, 225, 159, 24, 37, 207, 178, 103, 109, 174, 204, 227, 191, 61,
        224, 60, 242, 102, 71, 199, 141, 240, 11, 55, 27, 37, 204, 151, 0, 196, 198, 63, 128, 199,
        75, 17, 38, 62, 66, 30, 191, 132, 134, 164, 227, 9, 1, 33, 125, 166, 198, 179, 225, 159,
        24, 1, 4, 202, 254, 208, 13, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 76, 66,
        82, 255, 255, 255, 255, 255, 255, 255, 255,
    ];
    let actual_output = to_bytes(&input).unwrap();
    assert_eq!(expected_output, actual_output);
}

#[test]
fn test_transaction_argument_address_canonical_serialization_example() {
    let input = TransactionArgument::Address(AccountAddress::new([
        0x2c, 0x25, 0x99, 0x17, 0x85, 0x34, 0x3b, 0x23, 0xae, 0x07, 0x3a, 0x50, 0xe5, 0xfd, 0x80,
        0x9a,
    ]));

    let expected_output: Vec<u8> = vec![
        0x03, 0x2C, 0x25, 0x99, 0x17, 0x85, 0x34, 0x3B, 0x23, 0xAE, 0x07, 0x3A, 0x50, 0xE5, 0xFD,
        0x80, 0x9A,
    ];

    let actual_output = to_bytes(&input).unwrap();

    assert_eq!(expected_output, actual_output);
}

#[test]
fn test_transaction_argument_byte_array_canonical_serialization_example() {
    let input = TransactionArgument::U8Vector(vec![0xCA, 0xFE, 0xD0, 0x0D]);

    let expected_output: Vec<u8> = vec![0x04, 0x04, 0xCA, 0xFE, 0xD0, 0x0D];

    let actual_output = to_bytes(&input).unwrap();
    assert_eq!(expected_output, actual_output);
}

#[test]
fn test_transaction_argument_u64_canonical_serialization_example() {
    let input = TransactionArgument::U64(9_213_671_392_124_193_148);
    let expected_output: Vec<u8> = vec![0x01, 0x7C, 0xC9, 0xBD, 0xA4, 0x50, 0x89, 0xDD, 0x7F];

    let actual_output = to_bytes(&input).unwrap();
    assert_eq!(expected_output, actual_output);
}

#[test]
fn test_transaction_payload_with_a_program_canonical_serialization_example() {
    let input = TransactionPayload::Script(get_common_program());

    let expected_output = vec![
        0x01, 0x04, 0x6D, 0x6F, 0x76, 0x65, 0x00, 0x01, 0x01, 0xEF, 0xBE, 0xAD, 0xDE, 0x0D, 0xD0,
        0xFE, 0xCA,
    ];

    let actual_output = to_bytes(&input).unwrap();
    assert_eq!(expected_output, actual_output);
}

#[test]
fn test_transaction_payload_with_a_write_set_canonical_serialization_example() {
    let input = TransactionPayload::WriteSet(ChangeSet::new(get_common_write_set(), vec![]));

    let expected_output = vec![
        0, 2, 167, 29, 118, 250, 162, 210, 213, 195, 34, 78, 195, 212, 29, 235, 41, 57, 33, 1, 33,
        125, 166, 198, 179, 225, 159, 24, 37, 207, 178, 103, 109, 174, 204, 227, 191, 61, 224, 60,
        242, 102, 71, 199, 141, 240, 11, 55, 27, 37, 204, 151, 0, 196, 198, 63, 128, 199, 75, 17,
        38, 62, 66, 30, 191, 132, 134, 164, 227, 9, 1, 33, 125, 166, 198, 179, 225, 159, 24, 1, 4,
        202, 254, 208, 13, 0,
    ];

    let actual_output = to_bytes(&input).unwrap();
    assert_eq!(expected_output, actual_output);
}

#[test]
fn test_write_op_delete_canonical_serialization_example() {
    let input = WriteOp::Deletion;
    let expected_output = vec![0x00];

    let actual_output = to_bytes(&input).unwrap();
    assert_eq!(expected_output, actual_output);
}

#[test]
fn test_write_op_value_canonical_serialization_example() {
    let input = WriteOp::Value(vec![0xca, 0xfe, 0xd0, 0x0d]);
    let expected_output = vec![0x01, 0x04, 0xCA, 0xFE, 0xD0, 0x0D];

    let actual_output = to_bytes(&input).unwrap();
    assert_eq!(expected_output, actual_output);
}

#[test]
fn test_write_set_canonical_serialization_example() {
    let input = get_common_write_set();

    let expected_output = vec![
        0x02, 0xA7, 0x1D, 0x76, 0xFA, 0xA2, 0xD2, 0xD5, 0xC3, 0x22, 0x4E, 0xC3, 0xD4, 0x1D, 0xEB,
        0x29, 0x39, 0x21, 0x01, 0x21, 0x7D, 0xA6, 0xC6, 0xB3, 0xE1, 0x9F, 0x18, 0x25, 0xCF, 0xB2,
        0x67, 0x6D, 0xAE, 0xCC, 0xE3, 0xBF, 0x3D, 0xE0, 0x3C, 0xF2, 0x66, 0x47, 0xC7, 0x8D, 0xF0,
        0x0B, 0x37, 0x1B, 0x25, 0xCC, 0x97, 0x00, 0xC4, 0xC6, 0x3F, 0x80, 0xC7, 0x4B, 0x11, 0x26,
        0x3E, 0x42, 0x1E, 0xBF, 0x84, 0x86, 0xA4, 0xE3, 0x09, 0x01, 0x21, 0x7D, 0xA6, 0xC6, 0xB3,
        0xE1, 0x9F, 0x18, 0x01, 0x04, 0xCA, 0xFE, 0xD0, 0x0D,
    ];
    let actual_output = to_bytes(&input).unwrap();
    assert_eq!(expected_output, actual_output);
}

fn get_common_program() -> Script {
    Script::new(
        b"move".to_vec(),
        vec![],
        vec![TransactionArgument::U64(0xcafe_d00d_dead_beef)],
    )
}

fn get_common_write_set() -> WriteSet {
    WriteSetMut::new(vec![
        (
            AccessPath::new(
                AccountAddress::new([
                    0xa7, 0x1d, 0x76, 0xfa, 0xa2, 0xd2, 0xd5, 0xc3, 0x22, 0x4e, 0xc3, 0xd4, 0x1d,
                    0xeb, 0x29, 0x39,
                ]),
                vec![
                    0x01, 0x21, 0x7d, 0xa6, 0xc6, 0xb3, 0xe1, 0x9f, 0x18, 0x25, 0xcf, 0xb2, 0x67,
                    0x6d, 0xae, 0xcc, 0xe3, 0xbf, 0x3d, 0xe0, 0x3c, 0xf2, 0x66, 0x47, 0xc7, 0x8d,
                    0xf0, 0x0b, 0x37, 0x1b, 0x25, 0xcc, 0x97,
                ],
            ),
            WriteOp::Deletion,
        ),
        (
            AccessPath::new(
                AccountAddress::new([
                    0xc4, 0xc6, 0x3f, 0x80, 0xc7, 0x4b, 0x11, 0x26, 0x3e, 0x42, 0x1e, 0xbf, 0x84,
                    0x86, 0xa4, 0xe3,
                ]),
                vec![0x01, 0x21, 0x7d, 0xa6, 0xc6, 0xb3, 0xe1, 0x9f, 0x18],
            ),
            WriteOp::Value(vec![0xca, 0xfe, 0xd0, 0x0d]),
        ),
    ])
    .freeze()
    .unwrap()
}
