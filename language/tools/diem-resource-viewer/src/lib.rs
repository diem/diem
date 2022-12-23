// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};
use diem_types::{
    access_path::AccessPath, account_address::AccountAddress, account_state::AccountState,
    contract_event::ContractEvent,
};
use move_core_types::language_storage::StructTag;
use resource_viewer::MoveValueAnnotator;
use std::{
    collections::BTreeMap,
    fmt::{Display, Formatter},
};

use move_core_types::resolver::MoveResolver;
pub use resource_viewer::{AnnotatedMoveStruct, AnnotatedMoveValue};

pub struct DiemValueAnnotator<'a, T>(MoveValueAnnotator<'a, T>);

/// A wrapper around `MoveValueAnnotator` that adds a few diem-specific funtionalities.
#[derive(Debug)]
pub struct AnnotatedAccountStateBlob(BTreeMap<StructTag, AnnotatedMoveStruct>);

impl<'a, T: MoveResolver> DiemValueAnnotator<'a, T> {
    pub fn new(storage: &'a T) -> Self {
        Self(MoveValueAnnotator::new(storage))
    }

    pub fn view_resource(&self, tag: &StructTag, blob: &[u8]) -> Result<AnnotatedMoveStruct> {
        self.0.view_resource(tag, blob)
    }

    pub fn view_access_path(
        &self,
        access_path: AccessPath,
        blob: &[u8],
    ) -> Result<AnnotatedMoveStruct> {
        match access_path.get_struct_tag() {
            Some(tag) => self.view_resource(&tag, blob),
            None => bail!("Bad resource access path"),
        }
    }

    pub fn view_contract_event(&self, event: &ContractEvent) -> Result<AnnotatedMoveValue> {
        self.0.view_value(event.type_tag(), event.event_data())
    }

    pub fn view_account_state(&self, state: &AccountState) -> Result<AnnotatedAccountStateBlob> {
        let mut output = BTreeMap::new();
        for (k, v) in state.iter() {
            let tag = match AccessPath::new(AccountAddress::random(), k.to_vec()).get_struct_tag() {
                Some(t) => t,
                None => {
                    println!("Uncached AccessPath: {:?}", k);
                    continue;
                }
            };
            let value = self.view_resource(&tag, v)?;
            output.insert(tag, value);
        }
        Ok(AnnotatedAccountStateBlob(output))
    }
}

impl Display for AnnotatedAccountStateBlob {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        writeln!(f, "{{")?;
        for v in self.0.values() {
            write!(f, "{}", v)?;
            writeln!(f, ",")?;
        }
        writeln!(f, "}}")
    }
}
