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
use crate::file_format::Bytecode;
use anyhow::{bail, Result};
use byteorder::ReadBytesExt;
use std::{io::Cursor, mem::size_of};

/// Constant values for the binary format header.
///
/// The binary header is magic +  version info + table count.
pub enum BinaryConstants {}
impl BinaryConstants {
    /// The blob that must start a binary.
    pub const LIBRA_MAGIC_SIZE: usize = 4;
    pub const LIBRA_MAGIC: [u8; BinaryConstants::LIBRA_MAGIC_SIZE] = [0xA1, 0x1C, 0xEB, 0x0B];
    /// The `LIBRA_MAGIC` size, 1 byte for major version, 1 byte for minor version and 1 byte
    /// for table count.
    pub const HEADER_SIZE: usize = BinaryConstants::LIBRA_MAGIC_SIZE + 3;
    /// A (Table Type, Start Offset, Byte Count) size, which is 1 byte for the type and
    /// 4 bytes for the offset/count.
    pub const TABLE_HEADER_SIZE: u8 = size_of::<u32>() as u8 * 2 + 1;
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
    FUNCTION_INST           = 0x4,
    SIGNATURES              = 0x5,
    CONSTANT_POOL           = 0x6,
    IDENTIFIERS             = 0x7,
    ADDRESS_IDENTIFIERS     = 0x8,
    MAIN                    = 0x9,
    STRUCT_DEFS             = 0xA,
    STRUCT_DEF_INST         = 0xB,
    FUNCTION_DEFS           = 0xC,
    FIELD_HANDLE            = 0xD,
    FIELD_INST              = 0xE,
}

/// Constants for signature blob values.
#[rustfmt::skip]
#[allow(non_camel_case_types)]
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum SerializedType {
    BOOL                    = 0x1,
    U8                      = 0x2,
    U64                     = 0x3,
    U128                    = 0x4,
    ADDRESS                 = 0x5,
    REFERENCE               = 0x6,
    MUTABLE_REFERENCE       = 0x7,
    STRUCT                  = 0x8,
    TYPE_PARAMETER          = 0x9,
    VECTOR                  = 0xA,
    STRUCT_INST             = 0xB,
}

#[rustfmt::skip]
#[allow(non_camel_case_types)]
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum SerializedNominalResourceFlag {
    NOMINAL_RESOURCE        = 0x1,
    NORMAL_STRUCT           = 0x2,
}

#[rustfmt::skip]
#[allow(non_camel_case_types)]
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum SerializedKind {
    ALL                     = 0x1,
    COPYABLE                = 0x2,
    RESOURCE                = 0x3,
}

#[rustfmt::skip]
#[allow(non_camel_case_types)]
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum SerializedNativeStructFlag {
    NATIVE                  = 0x1,
    DECLARED                = 0x2,
}

/// List of opcodes constants.
#[rustfmt::skip]
#[allow(non_camel_case_types)]
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum Opcodes {
    POP                         = 0x01,
    RET                         = 0x02,
    BR_TRUE                     = 0x03,
    BR_FALSE                    = 0x04,
    BRANCH                      = 0x05,
    LD_U64                      = 0x06,
    LD_CONST                    = 0x07,
    LD_TRUE                     = 0x08,
    LD_FALSE                    = 0x09,
    COPY_LOC                    = 0x0A,
    MOVE_LOC                    = 0x0B,
    ST_LOC                      = 0x0C,
    MUT_BORROW_LOC              = 0x0D,
    IMM_BORROW_LOC              = 0x0E,
    MUT_BORROW_FIELD            = 0x0F,
    IMM_BORROW_FIELD            = 0x10,
    CALL                        = 0x11,
    PACK                        = 0x12,
    UNPACK                      = 0x13,
    READ_REF                    = 0x14,
    WRITE_REF                   = 0x15,
    ADD                         = 0x16,
    SUB                         = 0x17,
    MUL                         = 0x18,
    MOD                         = 0x19,
    DIV                         = 0x1A,
    BIT_OR                      = 0x1B,
    BIT_AND                     = 0x1C,
    XOR                         = 0x1D,
    OR                          = 0x1E,
    AND                         = 0x1F,
    NOT                         = 0x20,
    EQ                          = 0x21,
    NEQ                         = 0x22,
    LT                          = 0x23,
    GT                          = 0x24,
    LE                          = 0x25,
    GE                          = 0x26,
    ABORT                       = 0x27,
    GET_TXN_GAS_UNIT_PRICE      = 0x28,
    GET_TXN_MAX_GAS_UNITS       = 0x29,
    GET_GAS_REMAINING           = 0x2A,
    GET_TXN_SENDER              = 0x2B,
    EXISTS                      = 0x2C,
    MUT_BORROW_GLOBAL           = 0x2D,
    IMM_BORROW_GLOBAL           = 0x2E,
    MOVE_FROM                   = 0x2F,
    MOVE_TO                     = 0x30,
    GET_TXN_SEQUENCE_NUMBER     = 0x31,
    GET_TXN_PUBLIC_KEY          = 0x32,
    FREEZE_REF                  = 0x33,
    SHL                         = 0x34,
    SHR                         = 0x35,
    LD_U8                       = 0x36,
    LD_U128                     = 0x37,
    CAST_U8                     = 0x38,
    CAST_U64                    = 0x39,
    CAST_U128                   = 0x3A,
    MUT_BORROW_FIELD_GENERIC    = 0x3B,
    IMM_BORROW_FIELD_GENERIC    = 0x3C,
    CALL_GENERIC                = 0x3D,
    PACK_GENERIC                = 0x3E,
    UNPACK_GENERIC              = 0x3F,
    EXISTS_GENERIC              = 0x40,
    MUT_BORROW_GLOBAL_GENERIC   = 0x41,
    IMM_BORROW_GLOBAL_GENERIC   = 0x42,
    MOVE_FROM_GENERIC           = 0x43,
    MOVE_TO_GENERIC             = 0x44,
    NOP                         = 0x45,
}

/// Upper limit on the binary size
pub const BINARY_SIZE_LIMIT: usize = usize::max_value();

/// A wrapper for the binary vector
#[derive(Default)]
pub struct BinaryData {
    _binary: Vec<u8>,
}

/// The wrapper mirrors Vector operations but provides additional checks against overflow
impl BinaryData {
    pub fn new() -> Self {
        BinaryData {
            _binary: Vec::new(),
        }
    }

    pub fn as_inner(&self) -> &[u8] {
        &self._binary
    }

    pub fn into_inner(self) -> Vec<u8> {
        self._binary
    }

    pub fn push(&mut self, item: u8) -> Result<()> {
        if self.len().checked_add(1).is_some() {
            self._binary.push(item);
        } else {
            bail!(
                "binary size ({}) + 1 is greater than limit ({})",
                self.len(),
                BINARY_SIZE_LIMIT,
            );
        }
        Ok(())
    }

    pub fn extend(&mut self, vec: &[u8]) -> Result<()> {
        let vec_len: usize = vec.len();
        if self.len().checked_add(vec_len).is_some() {
            self._binary.extend(vec);
        } else {
            bail!(
                "binary size ({}) + {} is greater than limit ({})",
                self.len(),
                vec.len(),
                BINARY_SIZE_LIMIT,
            );
        }
        Ok(())
    }

    pub fn len(&self) -> usize {
        self._binary.len()
    }

    pub fn is_empty(&self) -> bool {
        self._binary.is_empty()
    }

    pub fn clear(&mut self) {
        self._binary.clear();
    }
}

impl From<Vec<u8>> for BinaryData {
    fn from(vec: Vec<u8>) -> Self {
        BinaryData { _binary: vec }
    }
}

/// Take a `Vec<u8>` and a value to write to that vector and applies LEB128 logic to
/// compress the u16.
pub fn write_u16_as_uleb128(binary: &mut BinaryData, value: u16) -> Result<()> {
    write_u32_as_uleb128(binary, u32::from(value))
}

/// Take a `Vec<u8>` and a value to write to that vector and applies LEB128 logic to
/// compress the u32.
pub fn write_u32_as_uleb128(binary: &mut BinaryData, value: u32) -> Result<()> {
    let mut val = value;
    loop {
        let v: u8 = (val & 0x7f) as u8;
        if u32::from(v) != val {
            binary.push(v | 0x80)?;
            val >>= 7;
        } else {
            binary.push(v)?;
            break;
        }
    }
    Ok(())
}

/// Write a `u16` in Little Endian format.
pub fn write_u16(binary: &mut BinaryData, value: u16) -> Result<()> {
    binary.extend(&value.to_le_bytes())
}

/// Write a `u32` in Little Endian format.
pub fn write_u32(binary: &mut BinaryData, value: u32) -> Result<()> {
    binary.extend(&value.to_le_bytes())
}

/// Write a `u64` in Little Endian format.
pub fn write_u64(binary: &mut BinaryData, value: u64) -> Result<()> {
    binary.extend(&value.to_le_bytes())
}

/// Write a `u128` in Little Endian format.
pub fn write_u128(binary: &mut BinaryData, value: u128) -> Result<()> {
    binary.extend(&value.to_le_bytes())
}

/// Reads a `u16` in ULEB128 format from a `binary`.
///
/// Takes a `&mut Cursor<&[u8]>` and returns a pair:
///
/// u16 - value read
///
/// Return an error on an invalid representation.
pub fn read_uleb128_as_u16(cursor: &mut Cursor<&[u8]>) -> Result<u16> {
    let mut value: u32 = 0;
    let mut shift: u8 = 0;
    while let Ok(byte) = cursor.read_u8() {
        let val = byte & 0x7f;
        value |= u32::from(val) << shift;
        if val == byte {
            if (shift > 0 && val == 0) || value > u32::from(std::u16::MAX) {
                bail!("invalid ULEB128 representation for u16");
            }
            return Ok(value as u16);
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
    let mut value: u64 = 0;
    let mut shift: u8 = 0;
    while let Ok(byte) = cursor.read_u8() {
        let val = byte & 0x7f;
        value |= u64::from(val) << shift;
        if val == byte {
            if (shift > 0 && val == 0) || value > std::u32::MAX.into() {
                bail!("invalid ULEB128 representation for u32");
            }
            return Ok(value as u32);
        }
        shift += 7;
        if shift > 28 {
            break;
        }
    }
    bail!("invalid ULEB128 representation for u32")
}

/// The encoding of the instruction is the serialized form of it, but disregarding the
/// serialization of the instruction's argument(s).
pub fn instruction_key(instruction: &Bytecode) -> u8 {
    use Bytecode::*;
    let opcode = match instruction {
        Pop => Opcodes::POP,
        Ret => Opcodes::RET,
        BrTrue(_) => Opcodes::BR_TRUE,
        BrFalse(_) => Opcodes::BR_FALSE,
        Branch(_) => Opcodes::BRANCH,
        LdU8(_) => Opcodes::LD_U8,
        LdU64(_) => Opcodes::LD_U64,
        LdU128(_) => Opcodes::LD_U128,
        CastU8 => Opcodes::CAST_U8,
        CastU64 => Opcodes::CAST_U64,
        CastU128 => Opcodes::CAST_U128,
        LdConst(_) => Opcodes::LD_CONST,
        LdTrue => Opcodes::LD_TRUE,
        LdFalse => Opcodes::LD_FALSE,
        CopyLoc(_) => Opcodes::COPY_LOC,
        MoveLoc(_) => Opcodes::MOVE_LOC,
        StLoc(_) => Opcodes::ST_LOC,
        Call(_) => Opcodes::CALL,
        CallGeneric(_) => Opcodes::CALL_GENERIC,
        Pack(_) => Opcodes::PACK,
        PackGeneric(_) => Opcodes::PACK_GENERIC,
        Unpack(_) => Opcodes::UNPACK,
        UnpackGeneric(_) => Opcodes::UNPACK_GENERIC,
        ReadRef => Opcodes::READ_REF,
        WriteRef => Opcodes::WRITE_REF,
        FreezeRef => Opcodes::FREEZE_REF,
        MutBorrowLoc(_) => Opcodes::MUT_BORROW_LOC,
        ImmBorrowLoc(_) => Opcodes::IMM_BORROW_LOC,
        MutBorrowField(_) => Opcodes::MUT_BORROW_FIELD,
        MutBorrowFieldGeneric(_) => Opcodes::MUT_BORROW_FIELD_GENERIC,
        ImmBorrowField(_) => Opcodes::IMM_BORROW_FIELD,
        ImmBorrowFieldGeneric(_) => Opcodes::IMM_BORROW_FIELD_GENERIC,
        MutBorrowGlobal(_) => Opcodes::MUT_BORROW_GLOBAL,
        MutBorrowGlobalGeneric(_) => Opcodes::MUT_BORROW_GLOBAL_GENERIC,
        ImmBorrowGlobal(_) => Opcodes::IMM_BORROW_GLOBAL,
        ImmBorrowGlobalGeneric(_) => Opcodes::IMM_BORROW_GLOBAL_GENERIC,
        Add => Opcodes::ADD,
        Sub => Opcodes::SUB,
        Mul => Opcodes::MUL,
        Mod => Opcodes::MOD,
        Div => Opcodes::DIV,
        BitOr => Opcodes::BIT_OR,
        BitAnd => Opcodes::BIT_AND,
        Xor => Opcodes::XOR,
        Shl => Opcodes::SHL,
        Shr => Opcodes::SHR,
        Or => Opcodes::OR,
        And => Opcodes::AND,
        Not => Opcodes::NOT,
        Eq => Opcodes::EQ,
        Neq => Opcodes::NEQ,
        Lt => Opcodes::LT,
        Gt => Opcodes::GT,
        Le => Opcodes::LE,
        Ge => Opcodes::GE,
        Abort => Opcodes::ABORT,
        GetTxnGasUnitPrice => Opcodes::GET_TXN_GAS_UNIT_PRICE,
        GetTxnMaxGasUnits => Opcodes::GET_TXN_MAX_GAS_UNITS,
        GetGasRemaining => Opcodes::GET_GAS_REMAINING,
        GetTxnSenderAddress => Opcodes::GET_TXN_SENDER,
        Exists(_) => Opcodes::EXISTS,
        ExistsGeneric(_) => Opcodes::EXISTS_GENERIC,
        MoveFrom(_) => Opcodes::MOVE_FROM,
        MoveFromGeneric(_) => Opcodes::MOVE_FROM_GENERIC,
        MoveToSender(_) => Opcodes::MOVE_TO,
        MoveToSenderGeneric(_) => Opcodes::MOVE_TO_GENERIC,
        GetTxnSequenceNumber => Opcodes::GET_TXN_SEQUENCE_NUMBER,
        GetTxnPublicKey => Opcodes::GET_TXN_PUBLIC_KEY,
        Nop => Opcodes::NOP,
    };
    opcode as u8
}
