// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_core_types::account_address::AccountAddress;
use move_symbol_pool::symbol::Symbol;
use std::{collections::BTreeMap, path::PathBuf};

pub type NamedAddress = Symbol;
pub type PackageName = Symbol;

pub type AddressDeclarations = BTreeMap<NamedAddress, Option<AccountAddress>>;
pub type DevAddressDeclarations = BTreeMap<NamedAddress, AccountAddress>;
pub type Version = (u64, u64, u64);
pub type Dependencies = BTreeMap<PackageName, Dependency>;
pub type Substitution = BTreeMap<NamedAddress, SubstOrRename>;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SourceManifest {
    pub package: PackageInfo,
    pub addresses: Option<AddressDeclarations>,
    pub dev_address_assignments: Option<DevAddressDeclarations>,
    pub build: Option<BuildInfo>,
    pub dependencies: Dependencies,
    pub dev_dependencies: Dependencies,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PackageInfo {
    pub name: PackageName,
    pub version: Version,
    pub authors: Vec<String>,
    pub license: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Dependency {
    pub local: PathBuf,
    pub subst: Option<Substitution>,
    pub version: Option<Version>,
    pub digest: Option<Vec<u8>>,
}

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct BuildInfo {
    pub language_version: Option<Version>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SubstOrRename {
    RenameFrom(NamedAddress),
    Assign(AccountAddress),
}
