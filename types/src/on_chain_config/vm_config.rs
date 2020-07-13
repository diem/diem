// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_address::AccountAddress, account_config::libra_root_address,
    on_chain_config::OnChainConfig, transaction::SCRIPT_HASH_LENGTH,
};
use anyhow::{format_err, Result};
use libra_crypto::HashValue;
use move_core_types::gas_schedule::{CostTable, GasConstants};
use serde::{Deserialize, Serialize};

/// Defines and holds the publishing policies for the VM. There are three possible configurations:
/// 1. No module publishing, only whitelisted scripts are allowed.
/// 2. No module publishing, custom scripts are allowed.
/// 3. Both module publishing and custom scripts are allowed.
/// We represent these as an enum instead of a struct since whitelisting and module/script
/// publishing are mutually exclusive options.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct VMPublishingOption {
    pub script_option: ScriptPublishingOption,
    pub module_option: ModulePublishingOption,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum ScriptPublishingOption {
    /// Only allow scripts on a whitelist to be run
    Locked(Vec<[u8; SCRIPT_HASH_LENGTH]>),
    /// Allow both custom scripts
    CustomScripts,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum ModulePublishingOption {
    /// Only allow a limited set of sender to publish a module.
    LimitedSender(Vec<AccountAddress>),
    /// Allow everyone to publish new module
    Open,
}

impl VMPublishingOption {
    pub fn locked(whitelist: Vec<[u8; SCRIPT_HASH_LENGTH]>) -> Self {
        Self {
            script_option: ScriptPublishingOption::Locked(whitelist),
            module_option: ModulePublishingOption::LimitedSender(vec![libra_root_address()]),
        }
    }

    pub fn custom_scripts() -> Self {
        Self {
            script_option: ScriptPublishingOption::CustomScripts,
            module_option: ModulePublishingOption::LimitedSender(vec![libra_root_address()]),
        }
    }

    pub fn open() -> Self {
        Self {
            script_option: ScriptPublishingOption::CustomScripts,
            module_option: ModulePublishingOption::Open,
        }
    }

    pub fn is_open_module(&self) -> bool {
        match &self.module_option {
            ModulePublishingOption::Open => true,
            _ => false,
        }
    }

    pub fn is_allowed_module(&self, module_sender: &AccountAddress) -> bool {
        match &self.module_option {
            ModulePublishingOption::Open => true,
            ModulePublishingOption::LimitedSender(whitelist) => whitelist.contains(module_sender),
        }
    }

    pub fn is_allowed_script(&self, program: &[u8]) -> bool {
        match &self.script_option {
            ScriptPublishingOption::CustomScripts => true,
            ScriptPublishingOption::Locked(whitelist) => {
                let hash_value = HashValue::sha3_256_of(program);
                whitelist.contains(hash_value.as_ref())
            }
        }
    }
}

/// Defines all the on chain configuration data needed by VM.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct VMConfig {
    pub publishing_option: VMPublishingOption,
    pub gas_schedule: CostTable,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
struct CostTableInner {
    pub instruction_table: Vec<u8>,
    pub native_table: Vec<u8>,
    pub gas_constants: GasConstants,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
struct VMConfigInner {
    pub publishing_option: Vec<u8>,
    pub gas_schedule: CostTableInner,
}

impl CostTableInner {
    pub fn as_cost_table(&self) -> Result<CostTable> {
        let instruction_table = lcs::from_bytes(&self.instruction_table)?;
        let native_table = lcs::from_bytes(&self.native_table)?;
        Ok(CostTable {
            instruction_table,
            native_table,
            gas_constants: self.gas_constants.clone(),
        })
    }
}

impl OnChainConfig for VMConfig {
    const IDENTIFIER: &'static str = "LibraVMConfig";

    fn deserialize_into_config(bytes: &[u8]) -> Result<Self> {
        let raw_vm_config = lcs::from_bytes::<VMConfigInner>(&bytes).map_err(|e| {
            format_err!(
                "Failed first round of deserialization for VMConfigInner: {}",
                e
            )
        })?;
        let publishing_option = lcs::from_bytes(&raw_vm_config.publishing_option)?;
        let gas_schedule = raw_vm_config.gas_schedule.as_cost_table()?;
        Ok(VMConfig {
            publishing_option,
            gas_schedule,
        })
    }
}
