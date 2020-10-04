// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::coverage_map::ExecCoverageMap;
use move_core_types::{identifier::Identifier, language_storage::ModuleId};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    io::{self, Write},
};
use vm::{access::ModuleAccess, CompiledModule};

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

pub fn summarize_inst_cov(
    module: &CompiledModule,
    coverage_map: &ExecCoverageMap,
) -> ModuleSummary {
    let module_name = module.self_id();
    let module_map = coverage_map
        .module_maps
        .get(&(*module_name.address(), module_name.name().to_owned()));

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
