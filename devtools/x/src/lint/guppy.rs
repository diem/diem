// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Project and package linters that run queries on guppy.

use crate::{
    config::{EnforcedAttributesConfig, OverlayConfig, TestOnlyConfig},
    lint::toml::toml_mismatch_message,
};
use guppy::{graph::feature::FeatureFilterFn, Version};
use std::{
    collections::{BTreeMap, HashMap},
    ffi::OsStr,
    iter,
    path::Path,
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

/// Ensure that the list of test-only crates is up to date.
#[derive(Debug)]
pub struct TestOnlyMembers<'cfg> {
    config: &'cfg TestOnlyConfig,
}

impl<'cfg> TestOnlyMembers<'cfg> {
    pub fn new(config: &'cfg TestOnlyConfig) -> Self {
        Self { config }
    }
}

impl<'cfg> Linter for TestOnlyMembers<'cfg> {
    fn name(&self) -> &'static str {
        "test-only-members"
    }
}

impl<'cfg> ProjectLinter for TestOnlyMembers<'cfg> {
    fn run<'l>(
        &self,
        ctx: &ProjectContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>> {
        // Set of test-only members is all workspace members minus default ones.
        let pkg_graph = ctx.package_graph()?;
        let workspace = pkg_graph.workspace();
        let default_members = ctx.default_workspace_members()?;
        let mut expected: Vec<_> = workspace
            .members()
            .filter_map(|(path, package)| {
                if default_members.contains(package.id()) {
                    None
                } else {
                    Some(path)
                }
            })
            .collect();
        expected.sort();

        if expected != self.config.members {
            // Create the expected TestOnlyConfig struct.
            let expected = TestOnlyConfig {
                members: expected
                    .into_iter()
                    .map(|path| path.to_path_buf())
                    .collect(),
            };
            out.write_kind(
                LintKind::File(Path::new("x.toml")),
                LintLevel::Error,
                toml_mismatch_message(
                    &expected,
                    self.config,
                    "test-only member list not canonical",
                )?,
            );
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

/// Assertions for "overlay" features.
///
/// An "overlay" feature is a feature name used throughout the codebase, whose purpose it is to
/// augment each package with extra code (e.g. proptest generators). Overlay features shouldn't be
/// enabled by anything except overlay features on workspace members, but may be enabled by default
/// by test-only or other non-default workspace members.
#[derive(Debug)]
pub struct OverlayFeatures<'cfg> {
    config: &'cfg OverlayConfig,
}

impl<'cfg> OverlayFeatures<'cfg> {
    pub fn new(config: &'cfg OverlayConfig) -> Self {
        Self { config }
    }

    fn is_overlay(&self, feature: Option<&str>) -> bool {
        match feature {
            Some(feature) => self.config.features.iter().any(|f| *f == feature),
            // The base feature isn't banned.
            None => false,
        }
    }
}

impl<'cfg> Linter for OverlayFeatures<'cfg> {
    fn name(&self) -> &'static str {
        "overlay-features"
    }
}

impl<'cfg> PackageLinter for OverlayFeatures<'cfg> {
    fn run<'l>(
        &self,
        ctx: &PackageContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>> {
        let package = ctx.metadata();
        if !ctx.is_default_member() {
            return Ok(RunStatus::Skipped(SkipReason::UnsupportedPackage(
                package.id(),
            )));
        }

        let filter = FeatureFilterFn::new(|_, feature_id| {
            // Accept all features except for overlay ones.
            !self.is_overlay(feature_id.feature())
        });

        let package_graph = ctx.package_graph();

        let package_query = package_graph
            .query_forward(iter::once(package.id()))
            .expect("valid package ID");
        let feature_query = package_graph
            .feature_graph()
            .query_packages(&package_query, filter);

        let mut overlays: Vec<(Option<&str>, &str, Option<&str>)> = vec![];

        feature_query.resolve_with_fn(|_, link| {
            // Consider the dependency even if it's dev-only since the v1 resolver unifies these.
            // TODO: might be able to relax this for the v2 resolver.
            let (from, to) = link.endpoints();
            let to_package = to.package();
            if to_package.in_workspace() && self.is_overlay(to.feature_id().feature()) {
                overlays.push((
                    from.feature_id().feature(),
                    to_package.name(),
                    to.feature_id().feature(),
                ));
            }

            // Don't need to traverse past direct dependencies.
            false
        });

        if !overlays.is_empty() {
            let mut msg = "overlay features enabled by default:\n".to_string();
            for (from_feature, to_package, to_feature) in overlays {
                msg.push_str(&format!(
                    "  * {} -> {}/{}\n",
                    feature_str(from_feature),
                    to_package,
                    feature_str(to_feature)
                ));
            }
            msg.push_str("Use a line in the [features] section instead.\n");
            out.write(LintLevel::Error, msg);
        }

        Ok(RunStatus::Executed)
    }
}

fn feature_str(feature: Option<&str>) -> &str {
    match feature {
        Some(feature) => feature,
        None => "[base]",
    }
}
