// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod compilation;
pub mod resolution;
pub mod source_package;

use anyhow::Result;
use compilation::compiled_package::CompiledPackage;
use std::{io::Write, path::Path};
use structopt::*;

use crate::{
    compilation::build_plan::BuildPlan,
    resolution::resolution_graph::ResolutionGraph,
    source_package::{layout, manifest_parser},
};

#[derive(Debug, StructOpt, Clone)]
#[structopt(
    name = "Move Package",
    about = "Package and build system for Move code."
)]
pub struct BuildConfig {
    /// Compile in 'dev' mode. The 'dev-addresses' and 'dev-dependencies' fields will be used if
    /// this flag is set. This flag is useful for development of packages that expose named
    /// addresses that are not set to a specific value.
    #[structopt(name = "dev-mode", short = "d", long = "dev")]
    pub dev_mode: bool,

    /// Compile in 'test' mode. The 'dev-addresses' and 'dev-dependencies' fields will be used
    /// along with any code in the 'test' directory will also be included.
    #[structopt(name = "test-mode", short = "t", long = "test")]
    pub test_mode: bool,
}

impl BuildConfig {
    pub fn compile_package<W: Write>(
        mut self,
        path: &Path,
        writer: &mut W,
    ) -> Result<CompiledPackage> {
        if self.test_mode {
            self.dev_mode = true;
        }
        let manifest_string =
            std::fs::read_to_string(path.join(layout::SourcePackageLayout::Manifest.path()))?;
        let toml_manifest = manifest_parser::parse_move_manifest_string(manifest_string)?;
        let manifest = manifest_parser::parse_source_manifest(toml_manifest)?;
        let resolution_graph = ResolutionGraph::new(manifest, path.to_path_buf(), self)?;
        let resolved_graph = resolution_graph.resolve()?;
        let build_plan = BuildPlan::create(resolved_graph)?;
        build_plan.compile(writer)
    }
}
