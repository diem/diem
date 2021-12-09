// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    command_line as cli,
    diagnostics::{codes::Severity, Diagnostic, Diagnostics},
};
use move_core_types::account_address::AccountAddress;
use move_ir_types::location::*;
use move_symbol_pool::Symbol;
use num_bigint::BigUint;
use petgraph::{algo::astar as petgraph_astar, graphmap::DiGraphMap};
use std::{
    collections::BTreeMap,
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

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Clone, Copy)]
#[repr(u32)]
/// Number format enum, the u32 value represents the base
pub enum NumberFormat {
    Decimal = 10,
    Hex = 16,
}

// Determines the base of the number literal, depending on the prefix
fn determine_num_text_and_base(s: &str) -> (&str, NumberFormat) {
    match s.strip_prefix("0x") {
        Some(s_hex) => (s_hex, NumberFormat::Hex),
        None => (s, NumberFormat::Decimal),
    }
}

// Parse a u8 from a decimal or hex encoding
pub fn parse_u8(s: &str) -> Result<(u8, NumberFormat), ParseIntError> {
    let (txt, base) = determine_num_text_and_base(s);
    Ok((u8::from_str_radix(txt, base as u32)?, base))
}

// Parse a u64 from a decimal or hex encoding
pub fn parse_u64(s: &str) -> Result<(u64, NumberFormat), ParseIntError> {
    let (txt, base) = determine_num_text_and_base(s);
    Ok((u64::from_str_radix(txt, base as u32)?, base))
}

// Parse a u128 from a decimal or hex encoding
pub fn parse_u128(s: &str) -> Result<(u128, NumberFormat), ParseIntError> {
    let (txt, base) = determine_num_text_and_base(s);
    Ok((u128::from_str_radix(txt, base as u32)?, base))
}

// Parse an address from a decimal or hex encoding
pub fn parse_address(s: &str) -> Option<([u8; AccountAddress::LENGTH], NumberFormat)> {
    let (txt, base) = determine_num_text_and_base(s);
    let parsed = BigUint::parse_bytes(
        txt.as_bytes(),
        match base {
            NumberFormat::Hex => 16,
            NumberFormat::Decimal => 10,
        },
    )?;
    let bytes = parsed.to_bytes_be();
    if bytes.len() > AccountAddress::LENGTH {
        return None;
    }
    let mut result = [0u8; AccountAddress::LENGTH];
    result[(AccountAddress::LENGTH - bytes.len())..].clone_from_slice(&bytes);
    Some((result, base))
}

//**************************************************************************************************
// Address
//**************************************************************************************************

/// Numerical address represents non-named address values
/// or the assigned value of a named address
#[derive(Clone, Copy)]
pub struct NumericalAddress {
    /// the number for the address
    bytes: AccountAddress,
    /// The format (e.g. decimal or hex) for displaying the number
    format: NumberFormat,
}

impl NumericalAddress {
    // bytes used for errors when an address is not known but is needed
    pub const DEFAULT_ERROR_ADDRESS: Self = NumericalAddress {
        bytes: AccountAddress::ONE,
        format: NumberFormat::Hex,
    };

    pub const fn new(bytes: [u8; AccountAddress::LENGTH], format: NumberFormat) -> Self {
        Self {
            bytes: AccountAddress::new(bytes),
            format,
        }
    }

    pub fn into_inner(self) -> AccountAddress {
        self.bytes
    }

    pub fn into_bytes(self) -> [u8; AccountAddress::LENGTH] {
        self.bytes.into_bytes()
    }

    pub fn parse_str(s: &str) -> Result<NumericalAddress, String> {
        match parse_address(s) {
            Some((n, format)) => Ok(NumericalAddress {
                bytes: AccountAddress::new(n),
                format,
            }),
            None =>
            // TODO the kind of error is in an unstable nightly API
            // But currently the only way this should fail is if the number is too long
            {
                Err(format!(
                    "Invalid address literal. The numeric value is too large. The maximum size is \
                     {} bytes",
                    AccountAddress::LENGTH
                ))
            }
        }
    }
}

impl AsRef<[u8]> for NumericalAddress {
    fn as_ref(&self) -> &[u8] {
        self.bytes.as_ref()
    }
}

impl fmt::Display for NumericalAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.format {
            NumberFormat::Decimal => {
                let n = BigUint::from_bytes_be(self.bytes.as_ref());
                write!(f, "{}", n)
            }
            NumberFormat::Hex => write!(f, "{:#X}", self),
        }
    }
}

impl fmt::Debug for NumericalAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl fmt::UpperHex for NumericalAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let encoded = hex::encode_upper(self.as_ref());
        let dropped = encoded
            .chars()
            .skip_while(|c| c == &'0')
            .collect::<String>();
        let prefix = if f.alternate() { "0x" } else { "" };
        if dropped.is_empty() {
            write!(f, "{}0", prefix)
        } else {
            write!(f, "{}{}", prefix, dropped)
        }
    }
}

pub fn parse_named_address(s: &str) -> anyhow::Result<(String, NumericalAddress)> {
    let before_after = s.split('=').collect::<Vec<_>>();

    if before_after.len() != 2 {
        anyhow::bail!(
            "Invalid named address assignment. Must be of the form <address_name>=<address>, but \
             found '{}'",
            s
        );
    }
    let name = before_after[0].parse()?;
    let addr = NumericalAddress::parse_str(before_after[1])
        .map_err(|err| anyhow::format_err!("{}", err))?;

    Ok((name, addr))
}

pub fn verify_and_create_named_address_mapping(
    named_addresses: Vec<(String, NumericalAddress)>,
) -> anyhow::Result<BTreeMap<String, NumericalAddress>> {
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

impl PartialOrd for NumericalAddress {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for NumericalAddress {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let Self {
            bytes: self_bytes,
            format: _,
        } = self;
        let Self {
            bytes: other_bytes,
            format: _,
        } = other;
        self_bytes.cmp(other_bytes)
    }
}

impl PartialEq for NumericalAddress {
    fn eq(&self, other: &Self) -> bool {
        let Self {
            bytes: self_bytes,
            format: _,
        } = self;
        let Self {
            bytes: other_bytes,
            format: _,
        } = other;
        self_bytes == other_bytes
    }
}
impl Eq for NumericalAddress {}

impl Hash for NumericalAddress {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let Self {
            bytes: self_bytes,
            format: _,
        } = self;
        self_bytes.hash(state)
    }
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
    fn value(&self) -> Symbol;
    fn loc(&self) -> Loc;
}

// TODO maybe we should intern these strings somehow
pub type Name = Spanned<Symbol>;

impl TName for Name {
    type Key = Symbol;
    type Loc = Loc;

    fn drop_loc(self) -> (Loc, Symbol) {
        (self.loc, self.value)
    }

    fn add_loc(loc: Loc, key: Symbol) -> Self {
        sp(loc, key)
    }

    fn borrow(&self) -> (&Loc, &Symbol) {
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
    named_address_mapping: BTreeMap<Symbol, NumericalAddress>,
    // TODO(tzakian): Remove the global counter and use this counter instead
    // pub counter: u64,
}

impl CompilationEnv {
    pub fn new(flags: Flags, named_address_mapping: BTreeMap<Symbol, NumericalAddress>) -> Self {
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

    pub fn named_address_mapping(&self) -> &BTreeMap<Symbol, NumericalAddress> {
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
    use once_cell::sync::Lazy;
    use std::{collections::BTreeSet, fmt};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub enum AttributePosition {
        AddressBlock,
        Module,
        Script,
        Use,
        Friend,
        Constant,
        Struct,
        Function,
        Spec,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub enum KnownAttribute {
        Testing(TestingAttribute),
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub enum TestingAttribute {
        // Can be called by other testing code, and included in compilation in test mode
        TestOnly,
        // Is a test that will be run
        Test,
        // This test is expected to fail
        ExpectedFailure,
    }

    impl fmt::Display for AttributePosition {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::AddressBlock => write!(f, "address block"),
                Self::Module => write!(f, "module"),
                Self::Script => write!(f, "script"),
                Self::Use => write!(f, "use"),
                Self::Friend => write!(f, "friend"),
                Self::Constant => write!(f, "constant"),
                Self::Struct => write!(f, "struct"),
                Self::Function => write!(f, "function"),
                Self::Spec => write!(f, "spec"),
            }
        }
    }

    impl KnownAttribute {
        pub fn resolve(attribute_str: impl AsRef<str>) -> Option<Self> {
            Some(match attribute_str.as_ref() {
                TestingAttribute::TEST => Self::Testing(TestingAttribute::Test),
                TestingAttribute::TEST_ONLY => Self::Testing(TestingAttribute::TestOnly),
                TestingAttribute::EXPECTED_FAILURE => {
                    Self::Testing(TestingAttribute::ExpectedFailure)
                }
                _ => return None,
            })
        }

        pub const fn name(&self) -> &str {
            match self {
                Self::Testing(a) => a.name(),
            }
        }

        pub fn expected_positions(&self) -> &'static BTreeSet<AttributePosition> {
            match self {
                Self::Testing(a) => a.expected_positions(),
            }
        }
    }

    impl TestingAttribute {
        pub const TEST: &'static str = "test";
        pub const EXPECTED_FAILURE: &'static str = "expected_failure";
        pub const TEST_ONLY: &'static str = "test_only";
        pub const CODE_ASSIGNMENT_NAME: &'static str = "abort_code";

        pub const fn name(&self) -> &str {
            match self {
                Self::Test => Self::TEST,
                Self::TestOnly => Self::TEST_ONLY,
                Self::ExpectedFailure => Self::EXPECTED_FAILURE,
            }
        }

        pub fn expected_positions(&self) -> &'static BTreeSet<AttributePosition> {
            static TEST_ONLY_POSITIONS: Lazy<BTreeSet<AttributePosition>> = Lazy::new(|| {
                IntoIterator::into_iter([
                    AttributePosition::AddressBlock,
                    AttributePosition::Module,
                    AttributePosition::Use,
                    AttributePosition::Friend,
                    AttributePosition::Constant,
                    AttributePosition::Struct,
                    AttributePosition::Function,
                ])
                .collect()
            });
            static TEST_POSITIONS: Lazy<BTreeSet<AttributePosition>> =
                Lazy::new(|| IntoIterator::into_iter([AttributePosition::Function]).collect());
            static EXPECTED_FAILURE_POSITIONS: Lazy<BTreeSet<AttributePosition>> =
                Lazy::new(|| IntoIterator::into_iter([AttributePosition::Function]).collect());
            match self {
                TestingAttribute::TestOnly => &*TEST_ONLY_POSITIONS,
                TestingAttribute::Test => &*TEST_POSITIONS,
                TestingAttribute::ExpectedFailure => &*EXPECTED_FAILURE_POSITIONS,
            }
        }
    }
}
