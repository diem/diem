// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_ir_types::location::*;
use std::{
    convert::TryFrom,
    fmt,
    hash::Hash,
    sync::atomic::{AtomicUsize, Ordering as AtomicOrdering},
};

pub mod ast_debug;
pub mod fake_natives;
pub mod remembering_unique_map;
pub mod unique_map;

//**************************************************************************************************
// Address
//**************************************************************************************************

pub const ADDRESS_LENGTH: usize = 16;

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Default, Clone, Copy)]
pub struct Address([u8; ADDRESS_LENGTH]);

impl Address {
    pub const LIBRA_CORE: Address = Address::new([0u8; ADDRESS_LENGTH]);

    pub const fn new(address: [u8; ADDRESS_LENGTH]) -> Self {
        Address(address)
    }

    pub fn to_u8(self) -> [u8; ADDRESS_LENGTH] {
        self.0
    }

    pub fn parse_str(s: &str) -> Result<Address, String> {
        let mut hex_string = String::from(&s[2..]);
        if hex_string.len() % 2 != 0 {
            hex_string.insert(0, '0');
        }

        let mut result = hex::decode(hex_string.as_str()).unwrap();
        let len = result.len();
        if len < ADDRESS_LENGTH {
            result.reverse();
            for _ in len..ADDRESS_LENGTH {
                result.push(0);
            }
            result.reverse();
        }

        assert!(result.len() >= ADDRESS_LENGTH);
        Self::try_from(&result[..]).map_err(|_| {
            format!(
                "Address is {} bytes long. The maximum size is {} bytes",
                result.len(),
                ADDRESS_LENGTH
            )
        })
    }
}

impl AsRef<[u8]> for Address {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        write!(f, "0x{:#x}", self)
    }
}

impl fmt::Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:#x}", self)
    }
}

impl fmt::LowerHex for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let encoded = hex::encode(&self.0);
        let dropped = encoded
            .chars()
            .skip_while(|c| c == &'0')
            .collect::<String>();
        if dropped.is_empty() {
            write!(f, "0")
        } else {
            write!(f, "{}", dropped)
        }
    }
}

impl TryFrom<&[u8]> for Address {
    type Error = String;

    fn try_from(bytes: &[u8]) -> Result<Address, String> {
        if bytes.len() != ADDRESS_LENGTH {
            Err(format!("The Address {:?} is of invalid length", bytes))
        } else {
            let mut addr = [0u8; ADDRESS_LENGTH];
            addr.copy_from_slice(bytes);
            Ok(Address(addr))
        }
    }
}

//**************************************************************************************************
// Name
//**************************************************************************************************

pub trait TName: Eq + Ord + Clone {
    type Key: Ord + Clone;
    type Loc: Copy;
    fn drop_loc(self) -> (Self::Loc, Self::Key);
    fn clone_drop_loc(&self) -> (Self::Loc, Self::Key);
    fn add_loc(loc: Self::Loc, key: Self::Key) -> Self;
}

pub trait Identifier {
    fn value(&self) -> &str;
    fn loc(&self) -> Loc;
}

// TODO maybe we should intern these strings somehow
pub type Name = Spanned<String>;

impl TName for Name {
    type Key = String;
    type Loc = Loc;

    fn drop_loc(self) -> (Loc, String) {
        (self.loc, self.value)
    }

    fn clone_drop_loc(&self) -> (Loc, String) {
        (self.loc, self.value.clone())
    }

    fn add_loc(loc: Loc, key: String) -> Self {
        sp(loc, key)
    }
}

//**************************************************************************************************
// Counter
//**************************************************************************************************

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Counter(usize);

impl Counter {
    pub fn next() -> u64 {
        static COUNTER_NEXT: AtomicUsize = AtomicUsize::new(0);

        COUNTER_NEXT.fetch_add(1, AtomicOrdering::AcqRel) as u64
    }
}

//**************************************************************************************************
// Display
//**************************************************************************************************

pub fn format_delim<T: fmt::Display, I: IntoIterator<Item = T>>(items: I, delim: &str) -> String {
    items
        .into_iter()
        .map(|item| format!("{}", item))
        .collect::<Vec<_>>()
        .join(delim)
}

pub fn format_comma<T: fmt::Display, I: IntoIterator<Item = T>>(items: I) -> String {
    format_delim(items, ", ")
}
