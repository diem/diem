// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod resolution;
pub mod source_package;

use anyhow::Result;
use std::path::Path;
use structopt::*;

use crate::{
    resolution::resolution_graph::ResolutionGraph,
    source_package::{layout, manifest_parser},
};

#[derive(Debug, StructOpt, Clone)]
#[structopt(
    name = "Move Package",
    about = "Package and build system for Move code."
)]
pub struct BuildConfig {
    /// Compile in 'dev' mode. The 'dev-addresses' and 'dev-dependencies' fields will be used
    #[structopt(name = "dev-mode", short = "d", long = "dev")]
    pub dev_mode: bool,

    /// Generate transaction builders for use in other languages
    #[structopt(
        name = "generate-transaction-builders",
        short = "b",
        long = "gen-builders"
    )]
    pub generate_transaction_builders: bool,

    /// Generate ABIs for scripts in the package
    #[structopt(name = "generate-abis", short = "a", long = "gen-abis")]
    pub generate_abis: bool,
}

impl BuildConfig {
    pub fn build(self, path: &Path) -> Result<()> {
        let manifest_string =
            std::fs::read_to_string(path.join(layout::SourcePackageLayout::Manifest.path()))?;
        let toml_manifest = manifest_parser::parse_move_manifest_string(manifest_string)?;
        let manifest = manifest_parser::parse_source_manifest(toml_manifest)?;
        let resolution_graph = ResolutionGraph::new(manifest, path.to_path_buf(), self)?;
        let _resolved_graph = resolution_graph.resolve()?;
        Ok(())
    }
}
