// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use anyhow::{format_err, Result};
use libra_types::account_address::AccountAddress;
use move_core_types::identifier::{IdentStr, Identifier};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufRead, BufReader, Read, Write},
    path::Path,
};

pub type FunctionCoverage = BTreeMap<u64, u64>;

#[derive(Debug, Serialize, Deserialize)]
pub struct CoverageMap {
    pub module_maps: BTreeMap<(AccountAddress, Identifier), ModuleCoverageMap>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModuleCoverageMap {
    pub module_addr: AccountAddress,
    pub module_name: Identifier,
    pub function_maps: BTreeMap<Identifier, FunctionCoverage>,
}

impl CoverageMap {
    /// Takes in a file containing a raw VM trace, and returns a coverage map.
    pub fn from_trace_file<P: AsRef<Path>>(filename: P) -> Self {
        let file = File::open(filename).unwrap();
        let mut module_maps = BTreeMap::new();
        for line in BufReader::new(file).lines() {
            let line = line.unwrap();
            let mut splits = line.split(',');
            let context = splits.next().unwrap();
            let pc = splits.next().unwrap().parse::<u64>().unwrap();

            let mut context_segs: Vec<_> = context.split("::").collect();
            let is_script = context_segs.len() == 2;
            // Don't count scripts (for now)
            if !is_script {
                let func_name = Identifier::new(context_segs.pop().unwrap()).unwrap();
                let module_name = Identifier::new(context_segs.pop().unwrap()).unwrap();
                let addr = AccountAddress::from_hex_literal(context_segs.pop().unwrap()).unwrap();
                let entry = module_maps
                    .entry((addr, module_name.clone()))
                    .or_insert_with(|| ModuleCoverageMap::new(addr, module_name));
                entry.insert(func_name, pc);
            }
        }
        CoverageMap { module_maps }
    }

    /// Takes in a file containing a serialized coverage map and returns a coverage map.
    pub fn from_binary_file<P: AsRef<Path>>(filename: P) -> Self {
        let mut bytes = Vec::new();
        File::open(filename)
            .ok()
            .and_then(|mut file| file.read_to_end(&mut bytes).ok())
            .ok_or_else(|| format_err!("Error while reading in coverage map binary"))
            .unwrap();
        lcs::from_bytes(&bytes)
            .map_err(|_| format_err!("Error deserializing into coverage map"))
            .unwrap()
    }
}

impl ModuleCoverageMap {
    pub fn new(module_addr: AccountAddress, module_name: Identifier) -> Self {
        ModuleCoverageMap {
            module_addr,
            module_name,
            function_maps: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, func_name: Identifier, pc: u64) {
        let func_entry = self
            .function_maps
            .entry(func_name)
            .or_insert_with(FunctionCoverage::new);
        let pc_entry = func_entry.entry(pc).or_insert(0);
        *pc_entry += 1;
    }

    pub fn get_function_coverage(&self, func_name: &IdentStr) -> Option<&FunctionCoverage> {
        self.function_maps.get(func_name)
    }
}

pub fn output_map_to_file<M: Serialize, P: AsRef<Path>>(file_name: P, data: &M) -> Result<()> {
    let bytes = lcs::to_bytes(data)?;
    let mut file = File::create(file_name)?;
    file.write_all(&bytes)?;
    Ok(())
}
