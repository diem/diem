// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Context;
use determinator::{
    rules::{DeterminatorRules, PathMatch},
    Determinator,
};
use guppy::graph::PackageGraph;
use indoc::indoc;
use x_lint::prelude::*;

#[derive(Debug)]
pub(super) struct DeterminatorMatch<'cfg> {
    determinator: Determinator<'cfg, 'cfg>,
}

impl<'cfg> DeterminatorMatch<'cfg> {
    pub fn new(graph: &'cfg PackageGraph, rules: &'cfg DeterminatorRules) -> crate::Result<Self> {
        // Use the same graph for old and new since we only care about file changes here.
        let mut determinator = Determinator::new(graph, graph);
        determinator
            .set_rules(rules)
            .with_context(|| "failed to set determinator rules")?;
        Ok(Self { determinator })
    }
}

impl<'cfg> Linter for DeterminatorMatch<'cfg> {
    fn name(&self) -> &'static str {
        "determinator-match"
    }
}

impl<'cfg> FilePathLinter for DeterminatorMatch<'cfg> {
    fn run<'l>(
        &self,
        ctx: &FilePathContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>> {
        // All other options for PathMatch are valid.
        if self.determinator.match_path(ctx.file_path(), |_| ()) == PathMatch::NoMatches {
            let msg = indoc!(
                "path didn't match any determinator rules or packages:
            * add path to x.toml's [[determinator.path-rule]] section
            * rules can include \"build everything\" or \"ignore file\""
            );
            out.write(LintLevel::Error, msg);
        }

        Ok(RunStatus::Executed)
    }
}
