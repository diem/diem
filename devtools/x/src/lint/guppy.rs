// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Project and package linters that run queries on guppy.

use crate::config::EnforcedAttributesConfig;
use std::collections::HashMap;
use x_lint::prelude::*;

/// Ban certain crates from being used as direct dependencies.
#[derive(Debug)]
pub struct BannedDirectDeps<'cfg> {
    banned_deps: &'cfg HashMap<String, String>,
}

impl<'cfg> BannedDirectDeps<'cfg> {
    pub fn new(banned_deps: &'cfg HashMap<String, String>) -> Self {
        Self { banned_deps }
    }
}

impl<'cfg> Linter for BannedDirectDeps<'cfg> {
    fn name(&self) -> &'static str {
        "banned-direct-deps"
    }
}

// This could be done either as a project linter or as a package linter -- doing it as a project
// linter is slightly cheaper empirically.
impl<'cfg> ProjectLinter for BannedDirectDeps<'cfg> {
    fn run<'l>(
        &self,
        ctx: &ProjectContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>> {
        let package_graph = ctx.package_graph()?;
        let banned_packages = package_graph.packages().filter_map(|package| {
            self.banned_deps
                .get(package.name())
                .map(|message| (package, message))
        });

        for (package, message) in banned_packages {
            // Look at the reverse direct dependencies of this package.
            let dep_links = package_graph
                .reverse_dep_links(package.id())
                .expect("valid package ID");
            for link in dep_links {
                if let Some(workspace_path) = link.from.workspace_path() {
                    out.write_kind(
                        LintKind::Package {
                            name: link.from.name(),
                            workspace_path,
                        },
                        LintLevel::Error,
                        format!("banned direct dependency '{}': {}", package.name(), message),
                    );
                }
            }
        }

        Ok(RunStatus::Executed)
    }
}

/// Enforce attributes on workspace crates.
#[derive(Debug)]
pub struct EnforcedAttributes<'cfg> {
    config: &'cfg EnforcedAttributesConfig,
}

impl<'cfg> EnforcedAttributes<'cfg> {
    pub fn new(config: &'cfg EnforcedAttributesConfig) -> Self {
        Self { config }
    }
}

impl<'cfg> Linter for EnforcedAttributes<'cfg> {
    fn name(&self) -> &'static str {
        "enforced-attributes"
    }
}

impl<'cfg> PackageLinter for EnforcedAttributes<'cfg> {
    fn run<'l>(
        &self,
        ctx: &PackageContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>> {
        let metadata = ctx.metadata();
        if let Some(authors) = &self.config.authors {
            if metadata.authors() != authors.as_slice() {
                out.write(
                    LintLevel::Error,
                    format!("invalid authors (expected {:?})", authors.join(", "),),
                );
            }
        }
        if let Some(license) = &self.config.license {
            if metadata.license() != Some(license.as_str()) {
                out.write(
                    LintLevel::Error,
                    format!("invalid license (expected {})", license),
                )
            }
        }

        Ok(RunStatus::Executed)
    }
}
