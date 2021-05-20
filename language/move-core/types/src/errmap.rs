// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::language_storage::ModuleId;
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs::File,
    io::{Read, Write},
    path::Path,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDescription {
    /// The constant name of error e.g., ECANT_PAY_DEPOSIT
    pub code_name: String,
    /// The code description. This is generated from the doc comments on the constant.
    pub code_description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    /// The error category e.g., INVALID_ARGUMENT
    pub category: ErrorDescription,
    /// The error reason e.g., ECANT_PAY_DEPOSIT
    pub reason: ErrorDescription,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMapping {
    /// The set of error categories and their descriptions
    pub error_categories: BTreeMap<u64, ErrorDescription>,
    /// The set of modules, and the module-specific errors
    pub module_error_maps: BTreeMap<ModuleId, BTreeMap<u64, ErrorDescription>>,
}

impl Default for ErrorMapping {
    fn default() -> Self {
        Self {
            error_categories: BTreeMap::new(),
            module_error_maps: BTreeMap::new(),
        }
    }
}

impl ErrorMapping {
    pub fn add_error_category(
        &mut self,
        category_id: u64,
        description: ErrorDescription,
    ) -> Result<()> {
        if let Some(previous_entry) = self.error_categories.insert(category_id, description) {
            bail!(format!(
                "Entry for category {} already taken by: {:#?}",
                category_id, previous_entry
            ))
        }
        Ok(())
    }

    pub fn add_module_error(
        &mut self,
        module_id: ModuleId,
        abort_code: u64,
        description: ErrorDescription,
    ) -> Result<()> {
        let module_error_map = self.module_error_maps.entry(module_id.clone()).or_default();
        if let Some(previous_entry) = module_error_map.insert(abort_code, description) {
            bail!(format!(
                "Duplicate entry for abort code {} found in {}, previous entry: {:#?}",
                abort_code, module_id, previous_entry
            ))
        }
        Ok(())
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Self {
        let mut bytes = Vec::new();
        File::open(path).unwrap().read_to_end(&mut bytes).unwrap();
        bcs::from_bytes(&bytes).unwrap()
    }

    pub fn to_file<P: AsRef<Path>>(&self, path: P) {
        let bytes = bcs::to_bytes(self).unwrap();
        let mut file = File::create(path).unwrap();
        file.write_all(&bytes).unwrap();
    }

    pub fn get_explanation(&self, module: &ModuleId, output_code: u64) -> Option<ErrorContext> {
        let category = output_code & 0xFFu64;
        let reason_code = output_code >> 8;
        self.error_categories.get(&category).and_then(|category| {
            self.module_error_maps.get(module).and_then(|module_map| {
                module_map.get(&reason_code).map(|reason| ErrorContext {
                    category: category.clone(),
                    reason: reason.clone(),
                })
            })
        })
    }
}
