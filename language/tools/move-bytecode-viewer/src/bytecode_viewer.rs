// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]
use crate::interfaces::LeftScreen;
use bytecode_source_map::{mapping::SourceMapping, source_map::SourceMap};
use disassembler::disassembler::{Disassembler, DisassemblerOptions};
use move_binary_format::{
    binary_views::BinaryIndexedView,
    file_format::{CodeOffset, CompiledModule, FunctionDefinitionIndex},
};
use move_ir_types::location::Loc;
use regex::Regex;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct BytecodeInfo {
    pub function_name: String,
    pub function_index: FunctionDefinitionIndex,
    pub code_offset: CodeOffset,
}

#[derive(Clone, Debug)]
pub struct BytecodeViewer<'a> {
    pub lines: Vec<String>,
    pub view: BinaryIndexedView<'a>,
    pub line_map: HashMap<usize, BytecodeInfo>,
}

impl<'a> BytecodeViewer<'a> {
    pub fn new(source_map: SourceMap<Loc>, module: &'a CompiledModule) -> Self {
        let view = BinaryIndexedView::Module(module);
        let source_mapping = SourceMapping::new(source_map, module.clone());
        let options = DisassemblerOptions {
            print_code: true,
            print_basic_blocks: true,
            ..Default::default()
        };
        let disassembled_string = Disassembler::new(source_mapping, options)
            .disassemble()
            .unwrap();

        let mut base_viewer = Self {
            lines: disassembled_string.lines().map(|x| x.to_string()).collect(),
            line_map: HashMap::new(),
            view,
        };
        base_viewer.build_mapping();
        base_viewer
    }

    fn build_mapping(&mut self) {
        let regex = Regex::new(r"^(\d+):.*").unwrap();
        let fun_regex = Regex::new(r"^public\s+([a-zA-Z_]+)\(").unwrap();
        let mut current_fun = None;
        let mut current_fdef_idx = None;
        let mut line_map = HashMap::new();

        let function_def_for_name: HashMap<String, u16> = self
            .view
            .function_defs()
            .into_iter()
            .flatten()
            .enumerate()
            .map(|(index, fdef)| {
                (
                    self.view
                        .identifier_at(self.view.function_handle_at(fdef.function).name)
                        .to_string(),
                    index as u16,
                )
            })
            .collect();

        for (i, line) in self.lines.iter().enumerate() {
            let line = line.trim();
            if let Some(cap) = fun_regex.captures(line) {
                let fn_name = cap.get(1).unwrap().as_str();
                let function_definition_index = function_def_for_name[fn_name];
                current_fun = Some(fn_name);
                current_fdef_idx = Some(FunctionDefinitionIndex(function_definition_index));
            }

            if let Some(cap) = regex.captures(line) {
                current_fun.map(|fname| {
                    let d = cap.get(1).unwrap().as_str().parse::<u16>().unwrap();
                    line_map.insert(
                        i,
                        BytecodeInfo {
                            function_name: fname.to_string(),
                            function_index: current_fdef_idx.unwrap(),
                            code_offset: d,
                        },
                    )
                });
            }
        }
        self.line_map = line_map;
    }
}

impl LeftScreen for BytecodeViewer<'_> {
    type SourceIndex = BytecodeInfo;

    fn get_source_index_for_line(&self, line: usize, _column: usize) -> Option<&Self::SourceIndex> {
        self.line_map.get(&line)
    }

    fn backing_string(&self) -> String {
        self.lines.join("\n").replace("\t", "    ")
    }
}
