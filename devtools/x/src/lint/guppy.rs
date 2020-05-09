// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Project and package linters that run queries on guppy.

use crate::config::EnforcedAttributesConfig;
use guppy::Version;
use std::{
    collections::{BTreeMap, HashMap},
    ffi::OsStr,
};
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
            for link in package.reverse_direct_links() {
                let from = link.from();
                if let Some(workspace_path) = from.workspace_path() {
                    out.write_kind(
                        LintKind::Package {
                            name: from.name(),
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

/// Check conventions in crate names and paths.
#[derive(Debug)]
pub struct CrateNamesPaths;

impl Linter for CrateNamesPaths {
    fn name(&self) -> &'static str {
        "crate-names-paths"
    }
}

impl PackageLinter for CrateNamesPaths {
    fn run<'l>(
        &self,
        ctx: &PackageContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>> {
        let name = ctx.metadata().name();
        if name.contains('_') {
            out.write(
                LintLevel::Error,
                "crate name contains '_' (use '-' instead)",
            );
        }

        let workspace_path = ctx.workspace_path();
        if let Some(path) = workspace_path.to_str() {
            if path.contains('_') {
                out.write(
                    LintLevel::Error,
                    "workspace path contains '_' (use '-' instead)",
                );
            }
        } else {
            // Workspace path is invalid UTF-8. A different lint should catch this.
        }

        for build_target in ctx.metadata().build_targets() {
            let target_name = build_target.name();
            if target_name.contains('_') {
                // If the path is implicitly specified by the name, don't warn about it.
                let file_stem = build_target.path().file_stem();
                if file_stem != Some(OsStr::new(target_name)) {
                    out.write(
                        LintLevel::Error,
                        format!(
                            "build target '{}' contains '_' (use '-' instead)",
                            target_name
                        ),
                    );
                }
            }
        }

        Ok(RunStatus::Executed)
    }
}

/// Ensure libra-workspace-hack is a dependency
#[derive(Debug)]
pub struct WorkspaceHack;

impl Linter for WorkspaceHack {
    fn name(&self) -> &'static str {
        "workspace-hack"
    }
}

impl PackageLinter for WorkspaceHack {
    fn run<'l>(
        &self,
        ctx: &PackageContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>> {
        let package = ctx.metadata();
        let pkg_graph = ctx.package_graph();
        let workspace_hack_id = pkg_graph
            .workspace()
            .member_by_name("libra-workspace-hack")
            .expect("can't find libra-workspace-hack package")
            .id();

        // libra-workspace-hack does not need to depend on itself
        if package.id() == workspace_hack_id {
            return Ok(RunStatus::Executed);
        }

        let has_links = package.direct_links().next().is_some();
        let has_hack_dep = pkg_graph
            .directly_depends_on(package.id(), workspace_hack_id)
            .expect("valid package ID");
        if has_links && !has_hack_dep {
            out.write(LintLevel::Error, "missing libra-workspace-hack dependency");
        }

        Ok(RunStatus::Executed)
    }
}

/// Ensure that any workspace packages with build dependencies also have a build script.
#[derive(Debug)]
pub struct IrrelevantBuildDeps;

impl Linter for IrrelevantBuildDeps {
    fn name(&self) -> &'static str {
        "irrelevant-build-deps"
    }
}

impl PackageLinter for IrrelevantBuildDeps {
    fn run<'l>(
        &self,
        ctx: &PackageContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>> {
        let metadata = ctx.metadata();

        let has_build_dep = metadata
            .direct_links()
            .any(|link| link.build().is_present());

        if !metadata.has_build_script() && has_build_dep {
            out.write(LintLevel::Error, "build dependencies but no build script");
        }

        Ok(RunStatus::Executed)
    }
}

/// Ensure that packages within the workspace only depend on one version of a third-party crate.
#[derive(Debug)]
pub struct DirectDepDups;

impl Linter for DirectDepDups {
    fn name(&self) -> &'static str {
        "direct-dep-dups"
    }
}

impl ProjectLinter for DirectDepDups {
    fn run<'l>(
        &self,
        ctx: &ProjectContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>> {
        let package_graph = ctx.package_graph()?;

        // This is a map of direct deps by name -> version -> packages that depend on it.
        let mut direct_deps: BTreeMap<&str, BTreeMap<&Version, Vec<&str>>> = BTreeMap::new();
        package_graph.query_workspace().resolve_with_fn(|_, link| {
            // Collect direct dependencies of workspace packages.
            let (from, to) = link.endpoints();
            if from.in_workspace() && !to.in_workspace() {
                direct_deps
                    .entry(to.name())
                    .or_default()
                    .entry(to.version())
                    .or_default()
                    .push(from.name());
            }
            // query_workspace + preventing further traversals will mean that only direct
            // dependencies are considered.
            false
        });
        for (direct_dep, versions) in direct_deps {
            if versions.len() > 1 {
                let mut msg = format!("duplicate direct dependency '{}':\n", direct_dep);
                for (version, packages) in versions {
                    msg.push_str(&format!("  * {} (", version));
                    msg.push_str(&packages.join(", "));
                    msg.push_str(")\n");
                }
                out.write(LintLevel::Error, msg);
            }
        }

        Ok(RunStatus::Executed)
    }
}
