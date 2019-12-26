// Copyright 2018 The Grin Developers
//! Common types and traits for cuckoo/cuckatoo family of solvers
use crate::error::Error;
use blake2_rfc::blake2b::blake2b;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::Cursor;

pub fn set_header_nonce(header: &[u8], nonce: Option<u32>) -> Result<[u64; 4], Error> {
    if let Some(n) = nonce {
        let len = header.len();
        let mut header = header.to_owned();
        header.truncate(len - 4); // drop last 4 bytes (u32) off the end
        header.write_u32::<LittleEndian>(n)?;
        create_siphash_keys(&header)
    } else {
        create_siphash_keys(&header)
    }
}

pub fn create_siphash_keys(header: &[u8]) -> Result<[u64; 4], Error> {
    let h = blake2b(32, &[], &header);
    let hb = h.as_bytes();
    let mut rdr = Cursor::new(hb);
    Ok([
        rdr.read_u64::<LittleEndian>()?,
        rdr.read_u64::<LittleEndian>()?,
        rdr.read_u64::<LittleEndian>()?,
        rdr.read_u64::<LittleEndian>()?,
    ])
}

/// Macro to clean up u64 unwrapping
#[macro_export]
macro_rules! to_u64 {
    ($n:expr) => {
        $n.to_u64().ok_or(ErrorKind::IntegerCast)?
    };
}

/// Macro to clean up u64 unwrapping as u32
#[macro_export]
macro_rules! to_u32 {
    ($n:expr) => {
        $n.to_u64().ok_or(ErrorKind::IntegerCast)? as u32
    };
}

/// Macro to clean up u64 unwrapping as usize
#[macro_export]
macro_rules! to_usize {
    ($n:expr) => {
        $n.to_u64().ok_or(ErrorKind::IntegerCast)? as usize
    };
}

/// Macro to clean up casting to edge type
/// TODO: this macro uses unhygenic data T
#[macro_export]
macro_rules! to_edge {
    ($n:expr) => {
        T::from($n).ok_or(ErrorKind::IntegerCast)?
    };
}
