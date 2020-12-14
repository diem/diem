// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::on_chain_config::OnChainConfig;
use anyhow::{format_err, Result};
use move_core_types::gas_schedule::{CostTable, GasConstants};
use serde::{Deserialize, Serialize};

/// Defines all the on chain configuration data needed by VM.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct VMConfig {
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
    pub gas_schedule: CostTableInner,
}

impl CostTableInner {
    pub fn as_cost_table(&self) -> Result<CostTable> {
        let instruction_table = bcs::from_bytes(&self.instruction_table)?;
        let native_table = bcs::from_bytes(&self.native_table)?;
        Ok(CostTable {
            instruction_table,
            native_table,
            gas_constants: self.gas_constants.clone(),
        })
    }
}

impl OnChainConfig for VMConfig {
    const IDENTIFIER: &'static str = "DiemVMConfig";

    fn deserialize_into_config(bytes: &[u8]) -> Result<Self> {
        let raw_vm_config = bcs::from_bytes::<VMConfigInner>(&bytes).map_err(|e| {
            format_err!(
                "Failed first round of deserialization for VMConfigInner: {}",
                e
            )
        })?;
        let gas_schedule = raw_vm_config.gas_schedule.as_cost_table()?;
        Ok(VMConfig { gas_schedule })
    }
}
