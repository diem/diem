// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::resolution::resolution_graph::ResolvedGraph;
use anyhow::Result;
use move_lang::shared::AddressBytes;
use move_model::{model::GlobalEnv, options::ModelBuilderOptions, run_model_builder_with_options};

#[derive(Debug, Clone)]
pub struct ModelBuilder {
    resolution_graph: ResolvedGraph,
}

impl ModelBuilder {
    pub fn create(resolution_graph: ResolvedGraph) -> Self {
        Self { resolution_graph }
    }

    // NOTE: If there are now renamings, then the root package has the global resolution of all named
    // addresses in the package graph in scope. So we can simply grab all of the source files
    // across all packages and build the Move model from that.
    // TODO: In the future we will need a better way to do this to support renaming in packages
    // where we want to support building a Move model.
    pub fn build_model(&self) -> Result<GlobalEnv> {
        // Make sure no renamings have been performed
        for (pkg_name, pkg) in self.resolution_graph.package_table.iter() {
            if !pkg.renaming.is_empty() {
                anyhow::bail!(
                    "Found address renaming in package '{}' when \
                    building Move model -- this is currently not supported",
                    pkg_name
                )
            }
        }

        // Targets are all files in the root package
        let root_name = &self.resolution_graph.root_package.package.name;
        let root_package = self.resolution_graph.get_package(root_name).clone();
        let targets: Vec<_> = root_package
            .get_sources(&self.resolution_graph.build_options)?
            .into_iter()
            .map(|symbol| symbol.to_string())
            .collect();
        // Dependencies are all files in non-root package
        let deps = self
            .resolution_graph
            .package_table
            .iter()
            .flat_map(|(nm, pkg)| {
                if nm == root_name {
                    vec![]
                } else {
                    pkg.get_sources(&self.resolution_graph.build_options)
                        .unwrap()
                }
            })
            .map(|symbol| symbol.to_string())
            .collect::<Vec<String>>();

        run_model_builder_with_options(
            &targets,
            &deps,
            ModelBuilderOptions::default(),
            root_package
                .resolution_table
                .into_iter()
                .map(|(ident, addr)| (ident.to_string(), AddressBytes::new(addr.to_u8())))
                .collect(),
        )
    }
}
