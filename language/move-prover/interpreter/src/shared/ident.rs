// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::fmt;

use bytecode::{function_target::FunctionTarget, function_target_pipeline::FunctionVariant};
use move_core_types::account_address::AccountAddress;
use move_model::model::{ModuleEnv, StructEnv};

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ModuleIdent {
    pub address: AccountAddress,
    pub name: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct FunctionIdent {
    pub module: ModuleIdent,
    pub name: String,
    pub variant: FunctionVariant,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct StructIdent {
    pub module: ModuleIdent,
    pub name: String,
}

//**************************************************************************************************
// Display
//**************************************************************************************************

impl fmt::Display for ModuleIdent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}::{}", self.address.short_str_lossless(), self.name)
    }
}

impl fmt::Display for FunctionIdent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "$[{}]-{}::{}", self.variant, self.module, self.name)
    }
}

impl fmt::Display for StructIdent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}::{}", self.module, self.name)
    }
}

//**************************************************************************************************
// Implementation
//**************************************************************************************************

impl ModuleIdent {
    pub fn new(module_env: &ModuleEnv) -> Self {
        let env = module_env.env;
        Self {
            address: *module_env.self_address(),
            name: env
                .symbol_pool()
                .string(module_env.get_name().name())
                .to_string(),
        }
    }
}

impl FunctionIdent {
    #[allow(dead_code)]
    pub fn new(target: &FunctionTarget) -> Self {
        let func_env = target.func_env;
        let module_env = &func_env.module_env;
        let env = module_env.env;
        FunctionIdent {
            module: ModuleIdent::new(module_env),
            name: env.symbol_pool().string(target.get_name()).to_string(),
            variant: target.data.variant.clone(),
        }
    }
}

impl StructIdent {
    pub fn new(struct_env: &StructEnv) -> Self {
        let module_env = &struct_env.module_env;
        let env = module_env.env;
        StructIdent {
            module: ModuleIdent::new(module_env),
            name: env.symbol_pool().string(struct_env.get_name()).to_string(),
        }
    }
}
