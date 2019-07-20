// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Constants for the binary format.
//!
//! Definition for the constants of the binary format, used by the serializer and the deserializer.
//! This module also offers helpers for the serialization and deserialization of certain
//! integer indexes.
//!
//! We use LEB128 for integer compression. LEB128 is a representation from the DWARF3 spec,
//! http://dwarfstd.org/Dwarf3Std.php or https://en.wikipedia.org/wiki/LEB128.
//! It's used to compress mostly indexes into the main binary tables.
use byteorder::ReadBytesExt;
use failure::*;
use std::{io::Cursor, mem::size_of};

/// Constant values for the binary format header.
///
/// The binary header is magic +  version info + table count.
pub enum BinaryConstants {}
impl BinaryConstants {
    /// The blob that must start a binary.
    pub const LIBRA_MAGIC_SIZE: usize = 8;
    pub const LIBRA_MAGIC: [u8; BinaryConstants::LIBRA_MAGIC_SIZE] =
        [b'L', b'I', b'B', b'R', b'A', b'V', b'M', b'\n'];
    /// The `LIBRA_MAGIC` size, 1 byte for major version, 1 byte for minor version and 1 byte
    /// for table count.
    pub const HEADER_SIZE: usize = BinaryConstants::LIBRA_MAGIC_SIZE + 3;
    /// A (Table Type, Start Offset, Byte Count) size, which is 1 byte for the type and
    /// 4 bytes for the offset/count.
    pub const TABLE_HEADER_SIZE: u32 = size_of::<u32>() as u32 * 2 + 1;
}

/// Constants for table types in the binary.
///
/// The binary contains a subset of those tables. A table specification is a tuple (table type,
/// start offset, byte count) for a given table.
#[rustfmt::skip]
#[allow(non_camel_case_types)]
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TableType {
    MODULE_HANDLES          = 0x1,
    STRUCT_HANDLES          = 0x2,
    FUNCTION_HANDLES        = 0x3,
    ADDRESS_POOL            = 0x4,
    STRING_POOL             = 0x5,
    BYTE_ARRAY_POOL         = 0x6,
    MAIN                    = 0x7,
    STRUCT_DEFS             = 0x8,
    FIELD_DEFS              = 0x9,
    FUNCTION_DEFS           = 0xA,
    TYPE_SIGNATURES         = 0xB,
    FUNCTION_SIGNATURES     = 0xC,
    LOCALS_SIGNATURES       = 0xD,
}

/// Constants for signature kinds (type, function, locals). Those values start a signature blob.
#[rustfmt::skip]
#[allow(non_camel_case_types)]
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum SignatureType {
    TYPE_SIGNATURE          = 0x1,
    FUNCTION_SIGNATURE      = 0x2,
    LOCAL_SIGNATURE         = 0x3,
}

/// Constants for signature blob values.
#[rustfmt::skip]
#[allow(non_camel_case_types)]
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum SerializedType {
    BOOL                    = 0x1,
    INTEGER                 = 0x2,
    STRING                  = 0x3,
    ADDRESS                 = 0x4,
    REFERENCE               = 0x5,
    MUTABLE_REFERENCE       = 0x6,
    STRUCT                  = 0x7,
    BYTEARRAY               = 0x8,
    TYPE_PARAMETER          = 0x9,
}

#[rustfmt::skip]
#[allow(non_camel_case_types)]
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum SerializedKind {
    RESOURCE                = 0x1,
    COPYABLE                = 0x2,
}

/// List of opcodes constants.
#[rustfmt::skip]
#[allow(non_camel_case_types)]
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum Opcodes {
    POP                     = 0x01,
    RET                     = 0x02,
    BR_TRUE                 = 0x03,
    BR_FALSE                = 0x04,
    BRANCH                  = 0x05,
    LD_CONST                = 0x06,
    LD_ADDR                 = 0x07,
    LD_STR                  = 0x08,
    LD_TRUE                 = 0x09,
    LD_FALSE                = 0x0A,
    COPY_LOC                = 0x0B,
    MOVE_LOC                = 0x0C,
    ST_LOC                  = 0x0D,
    LD_REF_LOC              = 0x0E,
    LD_REF_FIELD            = 0x0F,
    LD_BYTEARRAY            = 0x10,
    CALL                    = 0x11,
    PACK                    = 0x12,
    UNPACK                  = 0x13,
    READ_REF                = 0x14,
    WRITE_REF               = 0x15,
    ADD                     = 0x16,
    SUB                     = 0x17,
    MUL                     = 0x18,
    MOD                     = 0x19,
    DIV                     = 0x1A,
    BIT_OR                  = 0x1B,
    BIT_AND                 = 0x1C,
    XOR                     = 0x1D,
    OR                      = 0x1E,
    AND                     = 0x1F,
    NOT                     = 0x20,
    EQ                      = 0x21,
    NEQ                     = 0x22,
    LT                      = 0x23,
    GT                      = 0x24,
    LE                      = 0x25,
    GE                      = 0x26,
    ABORT                   = 0x27,
    GET_TXN_GAS_UNIT_PRICE  = 0x28,
    GET_TXN_MAX_GAS_UNITS   = 0x29,
    GET_GAS_REMAINING       = 0x2A,
    GET_TXN_SENDER          = 0x2B,
    EXISTS                  = 0x2C,
    BORROW_REF              = 0x2D,
    RELEASE_REF             = 0x2E,
    MOVE_FROM               = 0x2F,
    MOVE_TO                 = 0x30,
    CREATE_ACCOUNT          = 0x31,
    EMIT_EVENT              = 0x32,
    GET_TXN_SEQUENCE_NUMBER = 0x33,
    GET_TXN_PUBLIC_KEY      = 0x34,
    FREEZE_REF              = 0x35,
}

/// Take a `Vec<u8>` and a value to write to that vector and applies LEB128 logic to
/// compress the u16.
pub fn write_u16_as_uleb128(binary: &mut Vec<u8>, value: u16) {
    write_u32_as_uleb128(binary, u32::from(value));
}

/// Take a `Vec<u8>` and a value to write to that vector and applies LEB128 logic to
/// compress the u32.
pub fn write_u32_as_uleb128(binary: &mut Vec<u8>, value: u32) {
    let mut val = value;
    loop {
        let v: u8 = (val & 0x7f) as u8;
        if u32::from(v) != val {
            binary.push(v | 0x80);
            val >>= 7;
        } else {
            binary.push(v);
            break;
        }
    }
}

/// Write a `u16` in Little Endian format.
pub fn write_u16(binary: &mut Vec<u8>, value: u16) {
    binary.extend(&value.to_le_bytes());
}

/// Write a `u32` in Little Endian format.
pub fn write_u32(binary: &mut Vec<u8>, value: u32) {
    binary.extend(&value.to_le_bytes());
}

/// Write a `u64` in Little Endian format.
pub fn write_u64(binary: &mut Vec<u8>, value: u64) {
    binary.extend(&value.to_le_bytes());
}

/// Reads a `u16` in ULEB128 format from a `binary`.
///
/// Takes a `&mut Cursor<&[u8]>` and returns a pair:
///
/// u16 - value read
///
/// Return an error on an invalid representation.
pub fn read_uleb128_as_u16(cursor: &mut Cursor<&[u8]>) -> Result<u16> {
    let mut value: u16 = 0;
    let mut shift: u8 = 0;
    while let Ok(byte) = cursor.read_u8() {
        let val = byte & 0x7f;
        value |= u16::from(val) << shift;
        if val == byte {
            return Ok(value);
        }
        shift += 7;
        if shift > 14 {
            break;
        }
    }
    bail!("invalid ULEB128 representation for u16")
}

/// Reads a `u32` in ULEB128 format from a `binary`.
///
/// Takes a `&mut Cursor<&[u8]>` and returns a pair:
///
/// u32 - value read
///
/// Return an error on an invalid representation.
pub fn read_uleb128_as_u32(cursor: &mut Cursor<&[u8]>) -> Result<u32> {
    let mut value: u32 = 0;
    let mut shift: u8 = 0;
    while let Ok(byte) = cursor.read_u8() {
        let val = byte & 0x7f;
        value |= u32::from(val) << shift;
        if val == byte {
            return Ok(value);
        }
        shift += 7;
        if shift > 28 {
            break;
        }
    }
    bail!("invalid ULEB128 representation for u32")
}
