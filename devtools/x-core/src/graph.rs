// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{Result, WorkspaceSubset, XCoreContext};
use guppy::MetadataCommand;

rental! {
    mod rent_package_graph {
        use crate::WorkspaceSubset;
        use guppy::graph::PackageGraph;

        #[rental(covariant)]
        pub(crate) struct PackageGraphPlus {
            g: Box<PackageGraph>,
            default_members: WorkspaceSubset<'g>,
        }
    }
}

pub(crate) use rent_package_graph::PackageGraphPlus;

impl PackageGraphPlus {
    pub(crate) fn create(ctx: &XCoreContext) -> Result<Self> {
        let mut cmd = MetadataCommand::new();
        // Run cargo metadata from the root of the workspace.
        let project_root = ctx.project_root();
        cmd.current_dir(project_root);

        Self::try_new_or_drop(Box::new(cmd.build_graph()?), move |graph| {
            WorkspaceSubset::default_members(graph, project_root)
        })
    }
}
