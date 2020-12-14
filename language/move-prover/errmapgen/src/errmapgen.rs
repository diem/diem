// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use anyhow::{bail, Result};
use move_core_types::{
    account_address::AccountAddress, identifier::Identifier, language_storage::ModuleId,
};
use serde::{Deserialize, Serialize};
use spec_lang::{
    ast::Value,
    env::{GlobalEnv, ModuleEnv, NamedConstantEnv},
    symbol::Symbol,
};
use std::{
    collections::BTreeMap,
    convert::TryFrom,
    fs::File,
    io::{Read, Write},
    path::Path,
    rc::Rc,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrmapOptions {
    /// The constant prefix that determines if a constant is an error or not
    pub error_prefix: String,
    /// The module ID of the error category module
    pub error_category_module: ModuleId,
    /// In which file to store the output
    pub output_file: String,
}

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

impl Default for ErrmapOptions {
    fn default() -> Self {
        Self {
            error_prefix: "E".to_string(),
            error_category_module: ModuleId::new(
                AccountAddress::from_hex_literal("0x1").unwrap(),
                Identifier::new("Errors").unwrap(),
            ),
            output_file: "errmap".to_string(),
        }
    }
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

pub struct ErrmapGen<'env> {
    /// Options for error map generation
    options: &'env ErrmapOptions,
    /// Input definitions
    env: &'env GlobalEnv,
    /// Output error mapping
    output: ErrorMapping,
}

impl<'env> ErrmapGen<'env> {
    pub fn new(env: &'env GlobalEnv, options: &'env ErrmapOptions) -> Self {
        Self {
            options,
            env,
            output: ErrorMapping::default(),
        }
    }

    pub fn save_result(&self) {
        self.output.to_file(&self.options.output_file);
    }

    pub fn gen(&mut self) {
        for module in self.env.get_modules() {
            if !module.is_script_module() && !module.is_dependency() {
                self.build_error_map(&module).unwrap()
            }
        }
    }

    fn build_error_map(&mut self, module: &ModuleEnv<'_>) -> Result<()> {
        let module_id = self.get_module_id_for_name(module);
        if module_id == self.options.error_category_module {
            self.build_error_categories(module)?
        } else {
            self.build_error_map_for_module(&module_id, module)?
        }
        Ok(())
    }

    fn build_error_categories(&mut self, module: &ModuleEnv<'_>) -> Result<()> {
        for named_constant in module.get_named_constants() {
            let name = self.name_string(named_constant.get_name());
            let error_category = self.get_abort_code(&named_constant)?;
            self.output.add_error_category(
                error_category,
                ErrorDescription {
                    code_name: name.to_string(),
                    code_description: named_constant.get_doc().to_string(),
                },
            )?
        }
        Ok(())
    }

    fn build_error_map_for_module(
        &mut self,
        module_id: &ModuleId,
        module: &ModuleEnv<'_>,
    ) -> Result<()> {
        for named_constant in module.get_named_constants() {
            let name = self.name_string(named_constant.get_name());
            if name.starts_with(&self.options.error_prefix) {
                let abort_code = self.get_abort_code(&named_constant)?;
                self.output.add_module_error(
                    module_id.clone(),
                    abort_code,
                    ErrorDescription {
                        code_name: name.to_string(),
                        code_description: named_constant.get_doc().to_string(),
                    },
                )?
            }
        }
        Ok(())
    }

    fn get_abort_code(&self, constant: &NamedConstantEnv<'_>) -> Result<u64> {
        match constant.get_value() {
            Value::Number(big_int) => u64::try_from(big_int).map_err(|err| err.into()),
            x => bail!(
                "Invalid abort code constant {} found for code {}",
                x,
                self.name_string(constant.get_name())
            ),
        }
    }

    fn get_module_id_for_name(&self, module: &ModuleEnv<'_>) -> ModuleId {
        let name = module.get_name();
        let addr = AccountAddress::from_hex_literal(&format!("0x{:x}", name.addr())).unwrap();
        let name = Identifier::new(self.name_string(name.name()).to_string()).unwrap();
        ModuleId::new(addr, name)
    }

    fn name_string(&self, symbol: Symbol) -> Rc<String> {
        self.env.symbol_pool().string(symbol)
    }
}
