// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};
use move_core_types::{
    account_address::AccountAddress,
    errmap::{ErrorDescription, ErrorMapping},
    identifier::Identifier,
    language_storage::ModuleId,
};
use move_model::{
    ast::Value,
    model::{GlobalEnv, ModuleEnv, NamedConstantEnv},
    symbol::Symbol,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, convert::TryFrom, rc::Rc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrmapOptions {
    /// The constant prefix that determines if a constant is an error or not
    pub error_prefix: String,
    /// The module IDs of the error category modules
    pub error_category_modules: BTreeSet<ModuleId>,
    /// In which file to store the output
    pub output_file: String,
}

impl Default for ErrmapOptions {
    fn default() -> Self {
        let mut error_category_modules = BTreeSet::new();
        error_category_modules.insert(ModuleId::new(
            AccountAddress::from_hex_literal("0x1").unwrap(),
            Identifier::new("Errors").unwrap(),
        ));
        Self {
            error_prefix: "E".to_string(),
            error_category_modules,
            output_file: "errmap".to_string(),
        }
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
            if !module.is_script_module() && module.is_target() {
                self.build_error_map(&module).unwrap()
            }
        }
    }

    fn build_error_map(&mut self, module: &ModuleEnv<'_>) -> Result<()> {
        let module_id = self.get_module_id_for_name(module);
        if self.options.error_category_modules.contains(&module_id) {
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
