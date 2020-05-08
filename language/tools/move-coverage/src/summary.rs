// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::coverage_map::CoverageMap;
use move_core_types::{identifier::Identifier, language_storage::ModuleId};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    io::{self, Write},
};
use vm::{access::ModuleAccess, CompiledModule};

#[derive(Debug, Serialize, Deserialize)]
pub struct ModuleSummaryOptions {
    pub summarize_function_coverage: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModuleSummary {
    pub module_name: ModuleId,
    pub function_summaries: BTreeMap<Identifier, FunctionSummary>,
    summary_options: ModuleSummaryOptions,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FunctionSummary {
    pub fn_is_native: bool,
    pub total_number_of_instructions: u64,
    pub covered_instructions: u64,
}

impl Default for ModuleSummaryOptions {
    fn default() -> Self {
        ModuleSummaryOptions {
            summarize_function_coverage: false,
        }
    }
}

impl ModuleSummary {
    pub fn new(
        summary_options: ModuleSummaryOptions,
        module: &CompiledModule,
        coverage_map: &CoverageMap,
    ) -> Self {
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
                        total_number_of_instructions: 0,
                        covered_instructions: 0,
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
                            total_number_of_instructions,
                            covered_instructions,
                        }
                    }
                };

                (fn_name, fn_summmary)
            })
            .collect();

        Self {
            summary_options,
            module_name,
            function_summaries,
        }
    }

    /// Summarizes the modules coverage, and returns the total module coverage
    pub fn summarize<W: Write>(&self, summary_writer: &mut W) -> io::Result<(u64, u64)> {
        let mut total_covered = 0;
        let mut total_instructions = 0;

        writeln!(
            summary_writer,
            "Module {}::{}",
            self.module_name.address(),
            self.module_name.name()
        )?;

        for (fn_name, fn_summary) in self.function_summaries.iter() {
            total_instructions += fn_summary.total_number_of_instructions;
            total_covered += fn_summary.covered_instructions;

            if self.summary_options.summarize_function_coverage {
                let native = if fn_summary.fn_is_native {
                    "native "
                } else {
                    ""
                };
                writeln!(summary_writer, "\t{}fun {}", native, fn_name)?;
                writeln!(
                    summary_writer,
                    "\t\ttotal instructions: {}",
                    fn_summary.total_number_of_instructions
                )?;
                writeln!(
                    summary_writer,
                    "\t\tcovered instructions: {}",
                    fn_summary.covered_instructions
                )?;
                writeln!(
                    summary_writer,
                    "\t\t% coverage: {:.2}",
                    fn_summary.percent_coverage()
                )?;
            }
        }

        let covered_percentage = percent_coverage_for_counts(total_instructions, total_covered);
        writeln!(
            summary_writer,
            ">>> % Module coverage: {:.2}",
            covered_percentage
        )?;
        Ok((total_instructions, total_covered))
    }
}

impl FunctionSummary {
    pub fn percent_coverage(&self) -> f64 {
        percent_coverage_for_counts(self.total_number_of_instructions, self.covered_instructions)
    }
}

pub fn percent_coverage_for_counts(total: u64, covered: u64) -> f64 {
    let total = total as f64;
    let covered = covered as f64;
    (covered as f64) / (total as f64) * 100f64
}
