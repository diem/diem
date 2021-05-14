// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{Result, SystemError, WorkspaceSubsets, XCoreContext};
use guppy::{graph::PackageGraph, MetadataCommand};
use ouroboros::self_referencing;

#[self_referencing]
pub(crate) struct PackageGraphPlus {
    g: Box<PackageGraph>,
    #[borrows(g)]
    #[covariant]
    subsets: WorkspaceSubsets<'this>,
}

impl PackageGraphPlus {
    pub(crate) fn create(ctx: &XCoreContext) -> Result<Self> {
        let mut cmd = MetadataCommand::new();
        // Run cargo metadata from the root of the workspace.
        let project_root = ctx.project_root();
        cmd.current_dir(project_root);

        Self::try_new(
            Box::new(
                cmd.build_graph()
                    .map_err(|err| SystemError::guppy("building package graph", err))?,
            ),
            move |graph| WorkspaceSubsets::new(graph, project_root, &ctx.config().subsets),
        )
    }

    pub(crate) fn package_graph(&self) -> &PackageGraph {
        self.borrow_g()
    }

    pub(crate) fn subsets(&self) -> &WorkspaceSubsets {
        self.borrow_subsets()
    }
}
