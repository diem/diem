// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::coverage_map::CoverageMap;
use bytecode_source_map::source_map::SourceMap;
use codespan::{Files, Span};
use colored::*;
use move_core_types::identifier::Identifier;
use move_ir_types::location::Loc;
use serde::Serialize;
use std::{
    collections::BTreeMap,
    fs,
    io::{self, Write},
    path::Path,
};
use vm::{
    access::ModuleAccess,
    file_format::{CodeOffset, FunctionDefinitionIndex},
    CompiledModule,
};

#[derive(Clone, Debug, Serialize)]
pub struct FunctionSourceCoverage {
    pub fn_is_native: bool,
    pub uncovered_locations: Vec<Loc>,
}

#[derive(Debug, Serialize)]
pub struct SourceCoverageBuilder {
    uncovered_locations: BTreeMap<Identifier, FunctionSourceCoverage>,
}

#[derive(Debug, Serialize)]
pub enum AbstractSegment {
    Bounded { start: u32, end: u32 },
    BoundedRight { end: u32 },
    BoundedLeft { start: u32 },
}

#[derive(Debug, Serialize)]
pub enum StringSegment {
    Covered(String),
    Uncovered(String),
}

pub type AnnotatedLine = Vec<StringSegment>;

#[derive(Debug, Serialize)]
pub struct SourceCoverage {
    pub annotated_lines: Vec<AnnotatedLine>,
}

impl SourceCoverageBuilder {
    pub fn new(
        module: &CompiledModule,
        coverage_map: &CoverageMap,
        source_map: &SourceMap<Loc>,
    ) -> Self {
        let module_name = module.self_id();
        let module_map = coverage_map
            .module_maps
            .get(&(*module_name.address(), module_name.name().to_owned()));

        let uncovered_locations: BTreeMap<Identifier, FunctionSourceCoverage> = module
            .function_defs()
            .iter()
            .enumerate()
            .flat_map(|(function_def_idx, function_def)| {
                let fn_handle = module.function_handle_at(function_def.function);
                let fn_name = module.identifier_at(fn_handle.name).to_owned();
                let function_def_idx = FunctionDefinitionIndex(function_def_idx as u16);

                // If the function summary doesn't exist then that function hasn't been called yet.
                let coverage = match &function_def.code {
                    None => Some(FunctionSourceCoverage {
                        fn_is_native: true,
                        uncovered_locations: Vec::new(),
                    }),
                    Some(code_unit) => {
                        module_map.and_then(|fn_map| match fn_map.function_maps.get(&fn_name) {
                            None => {
                                let function_map = source_map
                                    .get_function_source_map(function_def_idx)
                                    .unwrap();
                                let mut uncovered_locations = vec![function_map.decl_location];
                                uncovered_locations.extend(function_map.code_map.values());

                                Some(FunctionSourceCoverage {
                                    fn_is_native: false,
                                    uncovered_locations,
                                })
                            }
                            Some(function_coverage) => {
                                let uncovered_locations: Vec<_> = (0..code_unit.code.len())
                                    .flat_map(|code_offset| {
                                        if !function_coverage.contains_key(&(code_offset as u64)) {
                                            Some(
                                                source_map
                                                    .get_code_location(
                                                        function_def_idx,
                                                        code_offset as CodeOffset,
                                                    )
                                                    .unwrap(),
                                            )
                                        } else {
                                            None
                                        }
                                    })
                                    .collect();
                                Some(FunctionSourceCoverage {
                                    fn_is_native: false,
                                    uncovered_locations,
                                })
                            }
                        })
                    }
                };
                coverage.map(|x| (fn_name, x))
            })
            .collect();

        Self {
            uncovered_locations,
        }
    }

    pub fn compute_source_coverage(&self, file_path: &Path) -> SourceCoverage {
        let file_contents = fs::read_to_string(file_path).unwrap();
        let mut files = Files::new();
        let file_id = files.add(file_path.as_os_str().to_os_string(), file_contents.clone());

        let mut uncovered_segments = BTreeMap::new();

        for (_, fn_cov) in self.uncovered_locations.iter() {
            for span in merge_spans(fn_cov.clone()).into_iter() {
                let start_loc = files.location(file_id, span.start()).unwrap();
                let end_loc = files.location(file_id, span.end()).unwrap();
                let start_line = start_loc.line.0;
                let end_line = end_loc.line.0;
                if start_line == end_line {
                    let segments = uncovered_segments
                        .entry(start_line)
                        .or_insert_with(Vec::new);
                    segments.push(AbstractSegment::Bounded {
                        start: start_loc.column.0,
                        end: end_loc.column.0,
                    });
                } else {
                    let first_segment = uncovered_segments
                        .entry(start_line)
                        .or_insert_with(Vec::new);
                    first_segment.push(AbstractSegment::BoundedLeft {
                        start: start_loc.column.0,
                    });
                    for i in start_line + 1..end_line {
                        let segment = uncovered_segments.entry(i).or_insert_with(Vec::new);
                        segment.push(AbstractSegment::BoundedLeft { start: 0 });
                    }
                    let last_segment = uncovered_segments.entry(end_line).or_insert_with(Vec::new);
                    last_segment.push(AbstractSegment::BoundedRight {
                        end: end_loc.column.0,
                    });
                }
            }
        }

        let mut annotated_lines = Vec::new();
        for (line_number, mut line) in file_contents.lines().map(|x| x.to_owned()).enumerate() {
            match uncovered_segments.get(&(line_number as u32)) {
                None => annotated_lines.push(vec![StringSegment::Covered(line)]),
                Some(segments) => {
                    // Note: segments are already pre-sorted by construction so don't need to be
                    // resorted.
                    let mut line_acc = Vec::new();
                    let mut cursor = 0;
                    for segment in segments {
                        match segment {
                            AbstractSegment::Bounded { start, end } => {
                                let length = end - start;
                                let (before, after) = line.split_at((start - cursor) as usize);
                                let (uncovered, rest) = after.split_at(length as usize);
                                line_acc.push(StringSegment::Covered(before.to_string()));
                                line_acc.push(StringSegment::Uncovered(uncovered.to_string()));
                                line = rest.to_string();
                                cursor = *end;
                            }
                            AbstractSegment::BoundedRight { end } => {
                                let (uncovered, rest) = line.split_at((end - cursor) as usize);
                                line_acc.push(StringSegment::Uncovered(uncovered.to_string()));
                                line = rest.to_string();
                                cursor = *end;
                            }
                            AbstractSegment::BoundedLeft { start } => {
                                let (before, after) = line.split_at((start - cursor) as usize);
                                line_acc.push(StringSegment::Covered(before.to_string()));
                                line_acc.push(StringSegment::Uncovered(after.to_string()));
                                line = "".to_string();
                                cursor = 0;
                            }
                        }
                    }
                    if !line.is_empty() {
                        line_acc.push(StringSegment::Covered(line))
                    }
                    annotated_lines.push(line_acc)
                }
            }
        }

        SourceCoverage { annotated_lines }
    }
}

impl SourceCoverage {
    pub fn output_source_coverage<W: Write>(&self, output_writer: &mut W) -> io::Result<()> {
        for line in self.annotated_lines.iter() {
            for string_segment in line.iter() {
                match string_segment {
                    StringSegment::Covered(s) => write!(output_writer, "{}", s.green())?,
                    StringSegment::Uncovered(s) => write!(output_writer, "{}", s.red())?,
                }
            }
            writeln!(output_writer)?;
        }
        Ok(())
    }
}

fn merge_spans(cov: FunctionSourceCoverage) -> Vec<Span> {
    if cov.uncovered_locations.is_empty() {
        return vec![];
    }

    let mut covs: Vec<_> = cov
        .uncovered_locations
        .iter()
        .map(|loc| loc.span())
        .collect();
    covs.sort();

    let mut unioned = Vec::new();
    let mut curr = covs.remove(0);

    for interval in covs {
        if curr.disjoint(interval) {
            unioned.push(curr);
            curr = interval;
        } else {
            curr = curr.merge(interval);
        }
    }

    unioned.push(curr);
    unioned
}
