// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use anyhow::{format_err, Result};
use move_binary_format::file_format::{CodeOffset, CompiledModule};
use move_core_types::{
    account_address::AccountAddress,
    identifier::{IdentStr, Identifier},
};
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
    pub exec_maps: BTreeMap<String, ExecCoverageMap>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModuleCoverageMap {
    pub module_addr: AccountAddress,
    pub module_name: Identifier,
    pub function_maps: BTreeMap<Identifier, FunctionCoverage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecCoverageMap {
    pub exec_id: String,
    pub module_maps: BTreeMap<(AccountAddress, Identifier), ModuleCoverageMap>,
}

#[derive(Debug)]
pub struct ExecCoverageMapWithModules {
    pub module_maps: BTreeMap<(String, AccountAddress, Identifier), ModuleCoverageMap>,
    pub compiled_modules: BTreeMap<String, CompiledModule>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TraceEntry {
    pub module_addr: AccountAddress,
    pub module_name: Identifier,
    pub func_name: Identifier,
    pub func_pc: CodeOffset,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TraceMap {
    pub exec_maps: BTreeMap<String, Vec<TraceEntry>>,
}

impl CoverageMap {
    /// Takes in a file containing a raw VM trace, and returns an updated coverage map.
    pub fn update_coverage_from_trace_file<P: AsRef<Path>>(mut self, filename: P) -> Self {
        let file = File::open(filename).unwrap();
        for line in BufReader::new(file).lines() {
            let line = line.unwrap();
            let mut splits = line.split(',');
            let exec_id = splits.next().unwrap();
            let context = splits.next().unwrap();
            let pc = splits.next().unwrap().parse::<u64>().unwrap();

            let mut context_segs: Vec<_> = context.split("::").collect();
            let is_script = context_segs.len() == 2;
            if !is_script {
                let func_name = Identifier::new(context_segs.pop().unwrap()).unwrap();
                let module_name = Identifier::new(context_segs.pop().unwrap()).unwrap();
                let module_addr =
                    AccountAddress::from_hex_literal(context_segs.pop().unwrap()).unwrap();
                self.insert(exec_id, module_addr, module_name, func_name, pc);
            } else {
                // Don't count scripts (for now)
                assert_eq!(context_segs.pop().unwrap(), "main",);
                assert_eq!(context_segs.pop().unwrap(), "Script",);
            }
        }
        self
    }

    /// Takes in a file containing a raw VM trace, and returns a coverage map.
    pub fn from_trace_file<P: AsRef<Path>>(filename: P) -> Self {
        let empty_module_map = CoverageMap {
            exec_maps: BTreeMap::new(),
        };
        empty_module_map.update_coverage_from_trace_file(filename)
    }

    /// Takes in a file containing a serialized coverage map and returns a coverage map.
    pub fn from_binary_file<P: AsRef<Path>>(filename: P) -> Self {
        let mut bytes = Vec::new();
        File::open(filename)
            .ok()
            .and_then(|mut file| file.read_to_end(&mut bytes).ok())
            .ok_or_else(|| format_err!("Error while reading in coverage map binary"))
            .unwrap();
        bcs::from_bytes(&bytes)
            .map_err(|_| format_err!("Error deserializing into coverage map"))
            .unwrap()
    }

    // add entries in a cascading manner
    pub fn insert(
        &mut self,
        exec_id: &str,
        module_addr: AccountAddress,
        module_name: Identifier,
        func_name: Identifier,
        pc: u64,
    ) {
        let exec_entry = self
            .exec_maps
            .entry(exec_id.to_owned())
            .or_insert_with(|| ExecCoverageMap::new(exec_id.to_owned()));
        exec_entry.insert(module_addr, module_name, func_name, pc);
    }

    pub fn to_unified_exec_map(&self) -> ExecCoverageMap {
        let mut unified_map = ExecCoverageMap::new(String::new());
        for (_, exec_map) in self.exec_maps.iter() {
            for ((module_addr, module_name), module_map) in exec_map.module_maps.iter() {
                for (func_name, func_map) in module_map.function_maps.iter() {
                    for (pc, count) in func_map.iter() {
                        unified_map.insert_multi(
                            *module_addr,
                            module_name.clone(),
                            func_name.clone(),
                            *pc,
                            *count,
                        );
                    }
                }
            }
        }
        unified_map
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

    pub fn insert_multi(&mut self, func_name: Identifier, pc: u64, count: u64) {
        let func_entry = self
            .function_maps
            .entry(func_name)
            .or_insert_with(FunctionCoverage::new);
        let pc_entry = func_entry.entry(pc).or_insert(0);
        *pc_entry += count;
    }

    pub fn insert(&mut self, func_name: Identifier, pc: u64) {
        self.insert_multi(func_name, pc, 1);
    }

    pub fn merge(&mut self, another: ModuleCoverageMap) {
        for (key, val) in another.function_maps {
            self.function_maps
                .entry(key)
                .or_insert_with(FunctionCoverage::new)
                .extend(val);
        }
    }

    pub fn get_function_coverage(&self, func_name: &IdentStr) -> Option<&FunctionCoverage> {
        self.function_maps.get(func_name)
    }
}

impl ExecCoverageMap {
    pub fn new(exec_id: String) -> Self {
        ExecCoverageMap {
            exec_id,
            module_maps: BTreeMap::new(),
        }
    }

    pub fn insert_multi(
        &mut self,
        module_addr: AccountAddress,
        module_name: Identifier,
        func_name: Identifier,
        pc: u64,
        count: u64,
    ) {
        let module_entry = self
            .module_maps
            .entry((module_addr, module_name.clone()))
            .or_insert_with(|| ModuleCoverageMap::new(module_addr, module_name));
        module_entry.insert_multi(func_name, pc, count);
    }

    pub fn insert(
        &mut self,
        module_addr: AccountAddress,
        module_name: Identifier,
        func_name: Identifier,
        pc: u64,
    ) {
        self.insert_multi(module_addr, module_name, func_name, pc, 1);
    }

    pub fn into_coverage_map_with_modules(
        self,
        modules: BTreeMap<AccountAddress, BTreeMap<Identifier, (String, CompiledModule)>>,
    ) -> ExecCoverageMapWithModules {
        let retained: BTreeMap<(String, AccountAddress, Identifier), ModuleCoverageMap> = self
            .module_maps
            .into_iter()
            .filter_map(|((module_addr, module_name), module_cov)| {
                modules.get(&module_addr).and_then(|func_map| {
                    func_map.get(&module_name).map(|(module_path, _)| {
                        ((module_path.clone(), module_addr, module_name), module_cov)
                    })
                })
            })
            .collect();

        let compiled_modules = modules
            .into_iter()
            .map(|(_, module_map)| {
                module_map
                    .into_iter()
                    .map(|(_, (module_path, compiled_module))| (module_path, compiled_module))
            })
            .flatten()
            .collect();

        ExecCoverageMapWithModules {
            module_maps: retained,
            compiled_modules,
        }
    }
}

impl ExecCoverageMapWithModules {
    pub fn empty() -> Self {
        Self {
            module_maps: BTreeMap::new(),
            compiled_modules: BTreeMap::new(),
        }
    }

    pub fn merge(&mut self, another: ExecCoverageMapWithModules) {
        for ((module_path, module_addr, module_name), val) in another.module_maps {
            self.module_maps
                .entry((module_path.clone(), module_addr, module_name.clone()))
                .or_insert_with(|| ModuleCoverageMap::new(module_addr, module_name))
                .merge(val);
        }

        for (module_path, compiled_module) in another.compiled_modules {
            self.compiled_modules
                .entry(module_path)
                .or_insert(compiled_module);
        }
    }
}

impl TraceMap {
    /// Takes in a file containing a raw VM trace, and returns an updated coverage map.
    pub fn update_from_trace_file<P: AsRef<Path>>(mut self, filename: P) -> Self {
        let file = File::open(filename).unwrap();
        for line in BufReader::new(file).lines() {
            let line = line.unwrap();
            let mut splits = line.split(',');
            let exec_id = splits.next().unwrap();
            let context = splits.next().unwrap();
            let pc = splits.next().unwrap().parse::<u64>().unwrap();

            let mut context_segs: Vec<_> = context.split("::").collect();
            let is_script = context_segs.len() == 2;
            if !is_script {
                let func_name = Identifier::new(context_segs.pop().unwrap()).unwrap();
                let module_name = Identifier::new(context_segs.pop().unwrap()).unwrap();
                let module_addr =
                    AccountAddress::from_hex_literal(context_segs.pop().unwrap()).unwrap();
                self.insert(exec_id, module_addr, module_name, func_name, pc);
            } else {
                // Don't count scripts (for now)
                assert_eq!(context_segs.pop().unwrap(), "main",);
                assert_eq!(context_segs.pop().unwrap(), "Script",);
            }
        }
        self
    }

    // Takes in a file containing a raw VM trace, and returns a parsed trace.
    pub fn from_trace_file<P: AsRef<Path>>(filename: P) -> Self {
        let trace_map = TraceMap {
            exec_maps: BTreeMap::new(),
        };
        trace_map.update_from_trace_file(filename)
    }

    // Takes in a file containing a serialized trace and deserialize it.
    pub fn from_binary_file<P: AsRef<Path>>(filename: P) -> Self {
        let mut bytes = Vec::new();
        File::open(filename)
            .ok()
            .and_then(|mut file| file.read_to_end(&mut bytes).ok())
            .ok_or_else(|| format_err!("Error while reading in coverage map binary"))
            .unwrap();
        bcs::from_bytes(&bytes)
            .map_err(|_| format_err!("Error deserializing into coverage map"))
            .unwrap()
    }

    // add entries in a cascading manner
    pub fn insert(
        &mut self,
        exec_id: &str,
        module_addr: AccountAddress,
        module_name: Identifier,
        func_name: Identifier,
        pc: u64,
    ) {
        let exec_entry = self
            .exec_maps
            .entry(exec_id.to_owned())
            .or_insert_with(Vec::new);
        exec_entry.push(TraceEntry {
            module_addr,
            module_name,
            func_name,
            func_pc: pc as CodeOffset,
        });
    }
}

pub fn output_map_to_file<M: Serialize, P: AsRef<Path>>(file_name: P, data: &M) -> Result<()> {
    let bytes = bcs::to_bytes(data)?;
    let mut file = File::create(file_name)?;
    file.write_all(&bytes)?;
    Ok(())
}
