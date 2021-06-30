// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use bytecode_source_map::utils::{remap_owned_loc_to_loc, source_map_from_file, OwnedLoc};
use move_binary_format::file_format::CompiledModule;
use move_bytecode_viewer::{
    bytecode_viewer::BytecodeViewer, source_viewer::ModuleViewer,
    tui::tui_interface::start_tui_with_interface, viewer::Viewer,
};
use move_command_line_common::files::SOURCE_MAP_EXTENSION;
use std::{fs, path::Path};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Move Bytecode Explorer",
    about = "Explore Move bytecode and how the source code compiles to it"
)]
struct Args {
    /// The path to the module binary
    #[structopt(long = "module-path", short = "b")]
    pub module_binary_path: String,

    /// The path to the source file
    #[structopt(long = "source-path", short = "s")]
    pub source_file_path: String,
}

pub fn main() {
    let args = Args::from_args();
    let source_map_extension = SOURCE_MAP_EXTENSION;

    let bytecode_bytes = fs::read(&args.module_binary_path).expect("Unable to read bytecode file");
    let compiled_module =
        CompiledModule::deserialize(&bytecode_bytes).expect("Module blob can't be deserialized");

    let source_map = source_map_from_file::<OwnedLoc>(
        &Path::new(&args.module_binary_path).with_extension(source_map_extension),
    )
    .map(remap_owned_loc_to_loc)
    .unwrap();

    let source_path = Path::new(&args.source_file_path);
    let module_viewer =
        ModuleViewer::new(compiled_module.clone(), source_map.clone(), &source_path);
    let bytecode_viewer = BytecodeViewer::new(source_map, &compiled_module);

    let interface = Viewer::new(module_viewer, bytecode_viewer);
    start_tui_with_interface(interface).unwrap();
}
