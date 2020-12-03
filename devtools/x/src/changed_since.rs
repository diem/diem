// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{context::XContext, Result};
use anyhow::Context;
use determinator::Determinator;
use guppy::graph::{DependencyDirection, PackageSet};
use log::trace;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Args {
    /// List packages changed since this commit
    pub(crate) base: String,
}

pub fn run(args: Args, xctx: XContext) -> Result<()> {
    let affected_set = changed_since_impl(&xctx, &args.base)?;
    for package in affected_set.packages(DependencyDirection::Forward) {
        println!("{}", package.name());
    }
    Ok(())
}

pub(crate) fn changed_since_impl<'g>(xctx: &'g XContext, base: &str) -> Result<PackageSet<'g>> {
    let git_cli = xctx.core().git_cli();
    let merge_base = git_cli
        .merge_base(&base)
        .with_context(|| "failed to get merge base with HEAD")?;
    let (old_graph, (new_graph, files_changed)) = rayon::join(
        || {
            trace!("building old graph");
            git_cli
                .package_graph_at(&merge_base)
                .with_context(|| "failed to build old package graph")
                .map(|old_graph| {
                    // Initialize the feature graph since it will be required later on.
                    old_graph.feature_graph();
                    old_graph
                })
        },
        || {
            rayon::join(
                || {
                    trace!("building new graph");
                    xctx.core().package_graph().map(|new_graph| {
                        // Initialize the feature graph since it will be required later on.
                        new_graph.feature_graph();
                        new_graph
                    })
                },
                || {
                    // Get the list of files changed between the merge base and the current dir.
                    trace!("getting files changed");
                    git_cli
                        .files_changed_between(&merge_base, None, None)
                        .with_context(|| "error while getting files changed from merge base")
                },
            )
        },
    );
    let (old_graph, new_graph, files_changed) = (old_graph?, new_graph?, files_changed?);

    trace!("running determinator");
    let mut determinator = Determinator::new(&old_graph, new_graph);
    determinator
        .add_changed_paths(&files_changed)
        .set_rules(xctx.config().determinator_rules())
        .with_context(|| "failed to set determinator rules")?;

    let determinator_set = determinator.compute();
    Ok(determinator_set.affected_set)
}
