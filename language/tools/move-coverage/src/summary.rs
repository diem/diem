// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::coverage_map::{
    ExecCoverageMap, ExecCoverageMapWithModules, ModuleCoverageMap, TraceMap,
};
use move_binary_format::{
    access::ModuleAccess,
    control_flow_graph::{BlockId, ControlFlowGraph, VMControlFlowGraph},
    file_format::{Bytecode, CodeOffset},
    CompiledModule,
};
use move_core_types::{identifier::Identifier, language_storage::ModuleId};
use petgraph::{algo::tarjan_scc, Graph};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{self, Write},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct ModuleSummary {
    pub module_name: ModuleId,
    pub function_summaries: BTreeMap<Identifier, FunctionSummary>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FunctionSummary {
    pub fn_is_native: bool,
    pub total: u64,
    pub covered: u64,
}

pub struct FunctionInfo {
    pub fn_name: Identifier,
    pub fn_entry: CodeOffset,
    pub fn_returns: BTreeSet<CodeOffset>,
    pub fn_branches: BTreeMap<CodeOffset, BTreeSet<CodeOffset>>,
    pub fn_num_paths: u64,
}

impl ModuleSummary {
    /// Summarizes the modules coverage in CSV format
    pub fn summarize_csv<W: Write>(&self, summary_writer: &mut W) -> io::Result<()> {
        let module = format!(
            "{}::{}",
            self.module_name.address(),
            self.module_name.name()
        );

        let mut format_line = |fn_name, covered, uncovered| {
            writeln!(
                summary_writer,
                "{},{},{},{}",
                module, fn_name, covered, uncovered
            )
        };

        for (fn_name, fn_summary) in self
            .function_summaries
            .iter()
            .filter(|(_, summary)| !summary.fn_is_native)
        {
            format_line(fn_name, fn_summary.covered, fn_summary.total)?;
        }

        Ok(())
    }

    /// Summarizes the modules coverage, and returns the total module coverage in a human-readable
    /// format.
    pub fn summarize_human<W: Write>(
        &self,
        summary_writer: &mut W,
        summarize_function_coverage: bool,
    ) -> io::Result<(u64, u64)> {
        let mut all_total = 0;
        let mut all_covered = 0;

        writeln!(
            summary_writer,
            "Module {}::{}",
            self.module_name.address(),
            self.module_name.name()
        )?;

        for (fn_name, fn_summary) in self.function_summaries.iter() {
            all_total += fn_summary.total;
            all_covered += fn_summary.covered;

            if summarize_function_coverage {
                let native = if fn_summary.fn_is_native {
                    "native "
                } else {
                    ""
                };
                writeln!(summary_writer, "\t{}fun {}", native, fn_name)?;
                writeln!(summary_writer, "\t\ttotal: {}", fn_summary.total)?;
                writeln!(summary_writer, "\t\tcovered: {}", fn_summary.covered)?;
                writeln!(
                    summary_writer,
                    "\t\t% coverage: {:.2}",
                    fn_summary.percent_coverage()
                )?;
            }
        }

        let covered_percentage = (all_covered as f64) / (all_total as f64) * 100f64;
        writeln!(
            summary_writer,
            ">>> % Module coverage: {:.2}",
            covered_percentage
        )?;
        Ok((all_total, all_covered))
    }
}

impl FunctionSummary {
    pub fn percent_coverage(&self) -> f64 {
        (self.covered as f64) / (self.total as f64) * 100f64
    }
}

pub fn summarize_inst_cov_by_module(
    module: &CompiledModule,
    module_map: Option<&ModuleCoverageMap>,
) -> ModuleSummary {
    let module_name = module.self_id();
    let function_summaries: BTreeMap<_, _> = module
        .function_defs()
        .iter()
        .map(|function_def| {
            let fn_handle = module.function_handle_at(function_def.function);
            let fn_name = module.identifier_at(fn_handle.name).to_owned();

            let fn_summmary = match &function_def.code {
                None => FunctionSummary {
                    fn_is_native: true,
                    total: 0,
                    covered: 0,
                },
                Some(code_unit) => {
                    let total_number_of_instructions = code_unit.code.len() as u64;
                    let covered_instructions = module_map
                        .and_then(|fn_map| {
                            fn_map
                                .function_maps
                                .get(&fn_name)
                                .map(|function_map| function_map.len())
                        })
                        .unwrap_or(0) as u64;
                    FunctionSummary {
                        fn_is_native: false,
                        total: total_number_of_instructions,
                        covered: covered_instructions,
                    }
                }
            };

            (fn_name, fn_summmary)
        })
        .collect();

    ModuleSummary {
        module_name,
        function_summaries,
    }
}

pub fn summarize_inst_cov(
    module: &CompiledModule,
    coverage_map: &ExecCoverageMap,
) -> ModuleSummary {
    let module_name = module.self_id();
    let module_map = coverage_map
        .module_maps
        .get(&(*module_name.address(), module_name.name().to_owned()));
    summarize_inst_cov_by_module(module, module_map)
}

pub fn summarize_path_cov(module: &CompiledModule, trace_map: &TraceMap) -> ModuleSummary {
    let module_name = module.self_id();

    // collect branching information per function
    let func_info: BTreeMap<_, _> = module
        .function_defs()
        .iter()
        .filter_map(|function_def| {
            match &function_def.code {
                None => None,
                Some(code_unit) => {
                    // build control-flow graph
                    let fn_cfg = VMControlFlowGraph::new(code_unit.code.as_slice());

                    // get function entry and return points
                    let fn_entry = fn_cfg.block_start(fn_cfg.entry_block_id());
                    let mut fn_returns: BTreeSet<CodeOffset> = BTreeSet::new();
                    for block_id in fn_cfg.blocks().into_iter() {
                        for i in fn_cfg.block_start(block_id)..=fn_cfg.block_end(block_id) {
                            if let Bytecode::Ret = &code_unit.code[i as usize] {
                                fn_returns.insert(i);
                            }
                        }
                    }

                    // convert into strongly connected components (SCC) graph
                    let mut fn_dgraph: Graph<BlockId, ()> = Graph::new();

                    let block_to_node: BTreeMap<_, _> = fn_cfg
                        .blocks()
                        .into_iter()
                        .map(|block_id| (block_id, fn_dgraph.add_node(block_id)))
                        .collect();

                    for block_id in fn_cfg.blocks().into_iter() {
                        for succ_block_id in fn_cfg.successors(block_id).iter() {
                            fn_dgraph.add_edge(
                                *block_to_node.get(&block_id).unwrap(),
                                *block_to_node.get(succ_block_id).unwrap(),
                                (),
                            );
                        }
                    }

                    let scc_iter = tarjan_scc(&fn_dgraph).into_iter();

                    // collect branching points
                    let mut fn_branches: BTreeMap<CodeOffset, BTreeSet<CodeOffset>> =
                        BTreeMap::new();

                    let mut path_nums: BTreeMap<usize, BTreeMap<usize, usize>> = BTreeMap::new();
                    let mut inst_locs: BTreeMap<CodeOffset, usize> = BTreeMap::new();
                    for (scc_idx, scc) in scc_iter.enumerate() {
                        // collect locations (i.e., offsets) in this SCC
                        for node_idx in scc.iter() {
                            let block_id = *fn_dgraph.node_weight(*node_idx).unwrap();
                            for i in fn_cfg.block_start(block_id)..=fn_cfg.block_end(block_id) {
                                // there is no way we could assign the same instruction twice
                                assert!(inst_locs.insert(i, scc_idx).is_none());
                            }
                        }

                        // collect branches out of this SCC
                        let mut exits: BTreeSet<(CodeOffset, CodeOffset)> = BTreeSet::new();
                        for node_idx in scc.iter() {
                            let block_id = *fn_dgraph.node_weight(*node_idx).unwrap();
                            let term_inst_id = fn_cfg.block_end(block_id);
                            for dest in
                                Bytecode::get_successors(term_inst_id, code_unit.code.as_slice())
                                    .into_iter()
                            {
                                if *inst_locs.get(&dest).unwrap() != scc_idx {
                                    assert!(exits.insert((term_inst_id, dest)));
                                }
                            }
                        }

                        // calculate number of possible paths
                        if exits.is_empty() {
                            // this is the termination scc
                            assert!(path_nums.insert(scc_idx, BTreeMap::new()).is_none());
                            path_nums.get_mut(&scc_idx).unwrap().insert(scc_idx, 1);
                        } else {
                            // update reachability map
                            let mut reachability: BTreeMap<usize, usize> = BTreeMap::new();
                            for (_, dst) in exits.iter() {
                                let dst_scc_idx = inst_locs.get(dst).unwrap();
                                for (path_end_scc, path_end_reach_set) in path_nums.iter() {
                                    let reach_from_dst =
                                        if let Some(v) = path_end_reach_set.get(dst_scc_idx) {
                                            *v
                                        } else {
                                            0
                                        };
                                    let reach_from_scc =
                                        reachability.entry(*path_end_scc).or_insert(0);
                                    *reach_from_scc += reach_from_dst;
                                }
                            }

                            for (path_end_scc, path_end_reachability) in reachability.into_iter() {
                                assert!(path_nums
                                    .get_mut(&path_end_scc)
                                    .unwrap()
                                    .insert(scc_idx, path_end_reachability)
                                    .is_none());
                            }

                            // move to branch info if there are more than one branches
                            if exits.len() > 1 {
                                for (src, dst) in exits.into_iter() {
                                    fn_branches
                                        .entry(src)
                                        .or_insert_with(BTreeSet::new)
                                        .insert(dst);
                                }
                            }
                        }
                    }

                    // calculate path num
                    let entry_scc = inst_locs
                        .get(&fn_cfg.block_start(fn_cfg.entry_block_id()))
                        .unwrap();
                    let mut fn_num_paths: u64 = 0;
                    for (_, path_end_reachability) in path_nums {
                        fn_num_paths += if let Some(v) = path_end_reachability.get(entry_scc) {
                            *v as u64
                        } else {
                            0
                        };
                    }

                    // use function name as key
                    let fn_name = module
                        .identifier_at(module.function_handle_at(function_def.function).name)
                        .to_owned();
                    Some((
                        fn_name.clone(),
                        FunctionInfo {
                            fn_name,
                            fn_entry,
                            fn_returns,
                            fn_branches,
                            fn_num_paths,
                        },
                    ))
                }
            }
        })
        .collect();

    // examine the trace and check the path covered
    let mut func_path_cov_stats: BTreeMap<
        Identifier,
        BTreeMap<BTreeSet<(CodeOffset, CodeOffset)>, u64>,
    > = BTreeMap::new();

    for (_, trace) in trace_map.exec_maps.iter() {
        let mut call_stack: Vec<&FunctionInfo> = Vec::new();
        let mut path_stack: Vec<BTreeSet<(CodeOffset, CodeOffset)>> = Vec::new();
        let mut path_store: Vec<(Identifier, BTreeSet<(CodeOffset, CodeOffset)>)> = Vec::new();
        for (index, record) in trace.iter().enumerate().filter(|(_, e)| {
            e.module_addr == *module_name.address()
                && e.module_name.as_ident_str() == module_name.name()
        }) {
            let (info, is_call) = if let Some(last) = call_stack.last() {
                if last.fn_name.as_ident_str() != record.func_name.as_ident_str() {
                    // calls into a new function
                    (func_info.get(&record.func_name).unwrap(), true)
                } else if last.fn_entry == record.func_pc {
                    // recursive calls into itself
                    (*last, true)
                } else {
                    // execution stayed within the function
                    (*last, false)
                }
            } else {
                // fresh into the module
                (func_info.get(&record.func_name).unwrap(), true)
            };

            // push stacks if we call into a new function
            if is_call {
                assert_eq!(info.fn_entry, record.func_pc);
                call_stack.push(info);
                path_stack.push(BTreeSet::new());
            }
            let path = path_stack.last_mut().unwrap();

            // check if branching
            if let Some(dests) = info.fn_branches.get(&record.func_pc) {
                // the nest instruction must be within the same function
                let next_record = trace.get(index + 1).unwrap();
                assert_eq!(record.func_name, next_record.func_name);

                // add the transition to path
                if dests.contains(&next_record.func_pc) {
                    assert!(path.insert((record.func_pc, next_record.func_pc)));
                }
            }

            // pop stacks if we returned
            if info.fn_returns.contains(&record.func_pc) {
                call_stack.pop().unwrap();
                // save the full path temporarily in path_store
                path_store.push((record.func_name.clone(), path_stack.pop().unwrap()));
            }
        }

        // assert matches between calls and returns
        if !call_stack.is_empty() {
            // execution aborted...
            // TODO: it is better to confirm this by adding a trace record
            call_stack.clear();
            path_stack.clear();
            path_store.clear();
        } else {
            // record path only when execution finishes properly
            for (func_name, path) in path_store.into_iter() {
                let path_count = func_path_cov_stats
                    .entry(func_name)
                    .or_insert_with(BTreeMap::new)
                    .entry(path)
                    .or_insert(0);
                *path_count += 1;
            }
        }
    }

    // calculate function summaries
    let function_summaries: BTreeMap<_, _> = module
        .function_defs()
        .iter()
        .map(|function_def| {
            let fn_handle = module.function_handle_at(function_def.function);
            let fn_name = module.identifier_at(fn_handle.name).to_owned();

            let fn_summmary = match &function_def.code {
                None => FunctionSummary {
                    fn_is_native: true,
                    total: 0,
                    covered: 0,
                },
                Some(_) => FunctionSummary {
                    fn_is_native: false,
                    total: func_info.get(&fn_name).unwrap().fn_num_paths,
                    covered: match func_path_cov_stats.get(&fn_name) {
                        None => 0,
                        Some(pathset) => pathset.len() as u64,
                    },
                },
            };

            (fn_name, fn_summmary)
        })
        .collect();

    ModuleSummary {
        module_name,
        function_summaries,
    }
}

impl ExecCoverageMapWithModules {
    pub fn into_module_summaries(self) -> BTreeMap<String, ModuleSummary> {
        let compiled_modules = self.compiled_modules;
        self.module_maps
            .into_iter()
            .map(|((module_path, _, _), module_cov)| {
                let module_summary = summarize_inst_cov_by_module(
                    compiled_modules.get(&module_path).unwrap(),
                    Some(&module_cov),
                );
                (module_path, module_summary)
            })
            .collect()
    }
}
