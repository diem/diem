// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    command_line as cli,
    diagnostics::{codes::Severity, Diagnostic, Diagnostics},
};
use move_ir_types::location::*;
use petgraph::{algo::astar as petgraph_astar, graphmap::DiGraphMap};
use std::{
    collections::BTreeMap,
    convert::TryFrom,
    fmt,
    hash::Hash,
    num::ParseIntError,
    sync::atomic::{AtomicUsize, Ordering as AtomicOrdering},
};
use structopt::*;

pub mod ast_debug;
pub mod remembering_unique_map;
pub mod unique_map;
pub mod unique_set;

//**************************************************************************************************
// Numbers
//**************************************************************************************************

// Determines the base of the number literal, depending on the prefix
fn determine_num_text_and_base(s: &str) -> (&str, u32) {
    match s.strip_prefix("0x") {
        Some(s_hex) => (s_hex, 16),
        None => (s, 10),
    }
}

// Parse a u8 from a decimal or hex encoding
pub fn parse_u8(s: &str) -> Result<u8, ParseIntError> {
    let (txt, base) = determine_num_text_and_base(s);
    u8::from_str_radix(txt, base)
}

// Parse a u64 from a decimal or hex encoding
pub fn parse_u64(s: &str) -> Result<u64, ParseIntError> {
    let (txt, base) = determine_num_text_and_base(s);
    u64::from_str_radix(txt, base)
}

// Parse a u128 from a decimal or hex encoding
pub fn parse_u128(s: &str) -> Result<u128, ParseIntError> {
    let (txt, base) = determine_num_text_and_base(s);
    u128::from_str_radix(txt, base)
}

//**************************************************************************************************
// Address
//**************************************************************************************************

pub const ADDRESS_LENGTH: usize = 16;

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Clone, Copy)]
pub struct AddressBytes([u8; ADDRESS_LENGTH]);

impl AddressBytes {
    // bytes used for errors when an address is not known but is needed
    pub const DEFAULT_ERROR_BYTES: Self = AddressBytes([
        0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8,
    ]);

    pub const fn new(bytes: [u8; ADDRESS_LENGTH]) -> Self {
        Self(bytes)
    }

    pub fn into_bytes(self) -> [u8; ADDRESS_LENGTH] {
        self.0
    }

    pub fn parse_str(s: &str) -> Result<AddressBytes, String> {
        let decoded = match parse_u128(s) {
            Ok(n) => n.to_be_bytes(),
            Err(_) => {
                // TODO the kind of error is in an unstable nightly API
                // But currently the only way this should fail is if the number is too long
                return Err(
                    "Invalid address literal. The numeric value is too large. The maximum size is \
                     16 bytes"
                        .to_owned(),
                );
            }
        };
        Ok(AddressBytes(decoded))
    }
}

impl AsRef<[u8]> for AddressBytes {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Display for AddressBytes {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        write!(f, "0x{:#X}", self)
    }
}

impl fmt::Debug for AddressBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:#X}", self)
    }
}

impl fmt::LowerHex for AddressBytes {
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

impl TryFrom<&[u8]> for AddressBytes {
    type Error = String;

    fn try_from(bytes: &[u8]) -> Result<AddressBytes, String> {
        if bytes.len() != ADDRESS_LENGTH {
            Err(format!("The address {:?} is of invalid length", bytes))
        } else {
            let mut addr = [0u8; ADDRESS_LENGTH];
            addr.copy_from_slice(bytes);
            Ok(AddressBytes(addr))
        }
    }
}

impl fmt::UpperHex for AddressBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let encoded = hex::encode_upper(&self.0);
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

pub fn parse_named_address(s: &str) -> anyhow::Result<(String, AddressBytes)> {
    let before_after = s.split('=').collect::<Vec<_>>();

    if before_after.len() != 2 {
        anyhow::bail!("Invalid named address assignment. Must be of the form <address_name>=<address>, but found '{}'", s);
    }
    let name = before_after[0].parse()?;
    let addr =
        AddressBytes::parse_str(before_after[1]).map_err(|err| anyhow::format_err!("{}", err))?;

    Ok((name, addr))
}

pub fn verify_and_create_named_address_mapping(
    named_addresses: Vec<(String, AddressBytes)>,
) -> anyhow::Result<BTreeMap<String, AddressBytes>> {
    let mut mapping = BTreeMap::new();
    let mut invalid_mappings = BTreeMap::new();
    for (name, addr_bytes) in named_addresses {
        match mapping.insert(name.clone(), addr_bytes) {
            Some(other_addr) if other_addr != addr_bytes => {
                invalid_mappings
                    .entry(name)
                    .or_insert_with(Vec::new)
                    .push(other_addr);
            }
            None | Some(_) => (),
        }
    }

    if !invalid_mappings.is_empty() {
        let redefinitions = invalid_mappings
            .into_iter()
            .map(|(name, addr_bytes)| {
                format!(
                    "{} is assigned differing values {} and {}",
                    name,
                    addr_bytes
                        .iter()
                        .map(|x| format!("{}", x))
                        .collect::<Vec<_>>()
                        .join(","),
                    mapping[&name]
                )
            })
            .collect::<Vec<_>>();

        anyhow::bail!(
            "Redefinition of named addresses found in arguments to compiler: {}",
            redefinitions.join(", ")
        )
    }

    Ok(mapping)
}

//**************************************************************************************************
// Name
//**************************************************************************************************

pub trait TName: Eq + Ord + Clone {
    type Key: Ord + Clone;
    type Loc: Copy;
    fn drop_loc(self) -> (Self::Loc, Self::Key);
    fn add_loc(loc: Self::Loc, key: Self::Key) -> Self;
    fn borrow(&self) -> (&Self::Loc, &Self::Key);
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

    fn add_loc(loc: Loc, key: String) -> Self {
        sp(loc, key)
    }

    fn borrow(&self) -> (&Loc, &String) {
        (&self.loc, &self.value)
    }
}

//**************************************************************************************************
// Graphs
//**************************************************************************************************

pub fn shortest_cycle<'a, T: Ord + Hash>(
    dependency_graph: &DiGraphMap<&'a T, ()>,
    start: &'a T,
) -> Vec<&'a T> {
    let shortest_path = dependency_graph
        .neighbors(start)
        .fold(None, |shortest_path, neighbor| {
            let path_opt = petgraph_astar(
                dependency_graph,
                neighbor,
                |finish| finish == start,
                |_e| 1,
                |_| 0,
            );
            match (shortest_path, path_opt) {
                (p, None) | (None, p) => p,
                (Some((acc_len, acc_path)), Some((cur_len, cur_path))) => {
                    Some(if cur_len < acc_len {
                        (cur_len, cur_path)
                    } else {
                        (acc_len, acc_path)
                    })
                }
            }
        });
    let (_, mut path) = shortest_path.unwrap();
    path.insert(0, start);
    path
}

//**************************************************************************************************
// Compilation Env
//**************************************************************************************************

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilationEnv {
    flags: Flags,
    diags: Diagnostics,
    named_address_mapping: BTreeMap<String, AddressBytes>,
    // TODO(tzakian): Remove the global counter and use this counter instead
    // pub counter: u64,
}

impl CompilationEnv {
    pub fn new(flags: Flags, named_address_mapping: BTreeMap<String, AddressBytes>) -> Self {
        Self {
            flags,
            diags: Diagnostics::new(),
            named_address_mapping,
        }
    }

    pub fn add_diag(&mut self, diag: Diagnostic) {
        self.diags.add(diag)
    }

    pub fn add_diags(&mut self, diags: Diagnostics) {
        self.diags.extend(diags)
    }

    pub fn has_diags(&self) -> bool {
        !self.diags.is_empty()
    }

    pub fn count_diags(&self) -> usize {
        self.diags.len()
    }

    pub fn check_diags_at_or_above_severity(
        &mut self,
        threshold: Severity,
    ) -> Result<(), Diagnostics> {
        match self.diags.max_severity() {
            Some(max) if max >= threshold => Err(std::mem::take(&mut self.diags)),
            Some(_) | None => Ok(()),
        }
    }

    /// Should only be called after compilation is finished
    pub fn take_final_warning_diags(&mut self) -> Diagnostics {
        let final_diags = std::mem::take(&mut self.diags);
        debug_assert!(final_diags
            .max_severity()
            .map(|s| s == Severity::Warning)
            .unwrap_or(true));
        final_diags
    }

    pub fn flags(&self) -> &Flags {
        &self.flags
    }

    pub fn named_address_mapping(&self) -> &BTreeMap<String, AddressBytes> {
        &self.named_address_mapping
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

//**************************************************************************************************
// Flags
//**************************************************************************************************

#[derive(Clone, Debug, Eq, PartialEq, StructOpt)]
pub struct Flags {
    /// Compile in test mode
    #[structopt(
        short = cli::TEST_SHORT,
        long = cli::TEST,
    )]
    test: bool,

    /// If set, do not allow modules defined in source_files to shadow modules of the same id that
    /// exist in dependencies. Checking will fail in this case.
    #[structopt(
        name = "SOURCES_DO_NOT_SHADOW_DEPS",
        short = cli::NO_SHADOW_SHORT,
        long = cli::NO_SHADOW,
    )]
    no_shadow: bool,
}

impl Flags {
    pub fn empty() -> Self {
        Self {
            test: false,
            no_shadow: false,
        }
    }

    pub fn testing() -> Self {
        Self {
            test: true,
            no_shadow: false,
        }
    }

    pub fn set_sources_shadow_deps(self, sources_shadow_deps: bool) -> Self {
        Self {
            no_shadow: !sources_shadow_deps,
            ..self
        }
    }

    pub fn is_empty(&self) -> bool {
        self == &Self::empty()
    }

    pub fn is_testing(&self) -> bool {
        self.test
    }

    pub fn sources_shadow_deps(&self) -> bool {
        !self.no_shadow
    }
}

//**************************************************************************************************
// Attributes
//**************************************************************************************************

pub mod known_attributes {
    #[derive(Debug, PartialEq, Clone, PartialOrd, Eq, Ord)]
    pub enum TestingAttributes {
        // Can be called by other testing code, and included in compilation in test mode
        TestOnly,
        // Is a test that will be run
        Test,
        // This test is expected to fail
        ExpectedFailure,
    }

    impl TestingAttributes {
        pub const TEST: &'static str = "test";
        pub const EXPECTED_FAILURE: &'static str = "expected_failure";
        pub const TEST_ONLY: &'static str = "test_only";
        pub const CODE_ASSIGNMENT_NAME: &'static str = "abort_code";

        pub fn resolve(attribute_str: &str) -> Option<Self> {
            Some(match attribute_str {
                Self::TEST => Self::Test,
                Self::TEST_ONLY => Self::TestOnly,
                Self::EXPECTED_FAILURE => Self::ExpectedFailure,
                _ => return None,
            })
        }

        pub fn name(&self) -> &str {
            match self {
                Self::Test => Self::TEST,
                Self::TestOnly => Self::TEST_ONLY,
                Self::ExpectedFailure => Self::EXPECTED_FAILURE,
            }
        }
    }
}
