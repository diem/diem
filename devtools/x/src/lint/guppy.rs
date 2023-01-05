// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Project and package linters that run queries on guppy.

use crate::config::{
    BannedDepsConfig, DirectDepDupsConfig, EnforcedAttributesConfig, MoveToDiemDepsConfig,
    OverlayConfig,
};
use anyhow::anyhow;
use guppy::{
    graph::{feature::FeatureFilterFn, PackagePublish},
    Version, VersionReq,
};
use hakari::summaries::HakariBuilderSummary;
use std::{
    collections::{BTreeMap, HashMap},
    iter,
};
use x_core::WorkspaceStatus;
use x_lint::prelude::*;

/// Ban certain crates from being used as direct dependencies or in the default build.
#[derive(Debug)]
pub struct BannedDeps<'cfg> {
    config: &'cfg BannedDepsConfig,
}

impl<'cfg> BannedDeps<'cfg> {
    pub fn new(config: &'cfg BannedDepsConfig) -> Self {
        Self { config }
    }
}

impl<'cfg> Linter for BannedDeps<'cfg> {
    fn name(&self) -> &'static str {
        "banned-deps"
    }
}

impl<'cfg> ProjectLinter for BannedDeps<'cfg> {
    fn run<'l>(
        &self,
        ctx: &ProjectContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>> {
        let package_graph = ctx.package_graph()?;

        let filter_ban = |banned: &'cfg HashMap<String, String>| {
            package_graph.packages().filter_map(move |package| {
                banned
                    .get(package.name())
                    .map(move |message| (package, message))
            })
        };

        let banned_direct = &self.config.direct;
        for (package, message) in filter_ban(banned_direct) {
            // Look at the reverse direct dependencies of this package.
            for link in package.reverse_direct_links() {
                let from = link.from();
                if let Some(workspace_path) = from.source().workspace_path() {
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

        let default_members = ctx.default_members()?;

        let banned_default_build = &self.config.default_build;
        for (package, message) in filter_ban(banned_default_build) {
            if default_members.status_of(package.id()) != WorkspaceStatus::Absent {
                out.write(
                    LintLevel::Error,
                    format!(
                        "banned dependency in default build '{}': {}",
                        package.name(),
                        message
                    ),
                );
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
        if workspace_path.as_str().contains('_') {
            out.write(
                LintLevel::Error,
                "workspace path contains '_' (use '-' instead)",
            );
        }

        for build_target in ctx.metadata().build_targets() {
            let target_name = build_target.name();
            if target_name.contains('_') {
                // If the path is implicitly specified by the name, don't warn about it.
                let file_stem = build_target.path().file_stem();
                if file_stem != Some(target_name) {
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
pub struct DirectDepDups<'cfg> {
    config: &'cfg DirectDepDupsConfig,
    hakari_package: &'cfg str,
}

impl<'cfg> DirectDepDups<'cfg> {
    pub fn new(
        config: &'cfg DirectDepDupsConfig,
        hakari_config: &'cfg HakariBuilderSummary,
    ) -> crate::Result<Self> {
        let hakari_package = hakari_config
            .hakari_package
            .as_deref()
            .ok_or_else(|| anyhow!("hakari.hakari-package not defined in x.toml"))?;
        Ok(Self {
            config,
            hakari_package,
        })
    }
}

impl<'cfg> Linter for DirectDepDups<'cfg> {
    fn name(&self) -> &'static str {
        "direct-dep-dups"
    }
}

impl<'cfg> ProjectLinter for DirectDepDups<'cfg> {
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
            if from.name() == self.hakari_package {
                // Skip the workspace hack package.
                return false;
            }
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
        for (direct_dep, versions) in direct_deps
            .iter()
            .filter(|(d, _)| !self.config.allow.contains(&d.to_string()))
        {
            if versions.len() > 1 {
                let mut msg = format!("duplicate direct dependency '{}':\n", direct_dep);
                for (version, packages) in versions {
                    use std::fmt::Write as _;
                    let _ = write!(msg, "  * {} (", version);
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

        let feature_query = package_graph
            .query_forward(iter::once(package.id()))
            .expect("valid package ID")
            .to_feature_query(filter);

        let mut overlays: Vec<(Option<&str>, &str, Option<&str>)> = vec![];

        feature_query.resolve_with_fn(|_, link| {
            // We now use the v2 resolver, so dev-only links can be skipped.
            if !link.dev_only() {
                let (from, to) = link.endpoints();
                let to_package = to.package();
                if to_package.in_workspace() && self.is_overlay(to.feature_id().feature()) {
                    overlays.push((
                        from.feature_id().feature(),
                        to_package.name(),
                        to.feature_id().feature(),
                    ));
                }
            }

            // Don't need to traverse past direct dependencies.
            false
        });

        if !overlays.is_empty() {
            let mut msg = "overlay features enabled by default:\n".to_string();
            for (from_feature, to_package, to_feature) in overlays {
                use std::fmt::Write as _;
                let _ = writeln!(
                    msg,
                    "  * {} -> {}/{}",
                    feature_str(from_feature),
                    to_package,
                    feature_str(to_feature)
                );
            }
            msg.push_str("Use a line in the [features] section instead.\n");
            out.write(LintLevel::Error, msg);
        }

        Ok(RunStatus::Executed)
    }
}

fn feature_str(feature: Option<&str>) -> &str {
    feature.unwrap_or("[base]")
}

/// Ensure that all unpublished packages only use path dependencies for workspace dependencies
#[derive(Debug)]
pub struct UnpublishedPackagesOnlyUsePathDependencies {
    no_version_req: VersionReq,
}

impl UnpublishedPackagesOnlyUsePathDependencies {
    pub fn new() -> Self {
        Self {
            no_version_req: VersionReq::parse("*").expect("* should be a valid req"),
        }
    }
}

impl Linter for UnpublishedPackagesOnlyUsePathDependencies {
    fn name(&self) -> &'static str {
        "unpublished-packages-only-use-path-dependencies"
    }
}

impl PackageLinter for UnpublishedPackagesOnlyUsePathDependencies {
    fn run<'l>(
        &self,
        ctx: &PackageContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>> {
        let metadata = ctx.metadata();

        // Skip all packages which aren't 'publish = false'
        if !metadata.publish().is_never() {
            return Ok(RunStatus::Executed);
        }

        for direct_dep in metadata.direct_links().filter(|p| p.to().in_workspace()) {
            if direct_dep.version_req() != &self.no_version_req {
                let msg = format!(
                    "unpublished package specifies a version of first-party dependency '{}' ({}); \
                    unpublished packages should only use path dependencies for first-party packages.",
                    direct_dep.dep_name(),
                    direct_dep.version_req(),
                );
                out.write(LintLevel::Error, msg);
            }
        }

        Ok(RunStatus::Executed)
    }
}

/// Ensure that all published packages only depend on other, published packages
#[derive(Debug)]
pub struct PublishedPackagesDontDependOnUnpublishedPackages;

impl Linter for PublishedPackagesDontDependOnUnpublishedPackages {
    fn name(&self) -> &'static str {
        "published-packages-dont-depend-on-unpublished-packages"
    }
}

impl PackageLinter for PublishedPackagesDontDependOnUnpublishedPackages {
    fn run<'l>(
        &self,
        ctx: &PackageContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>> {
        let metadata = ctx.metadata();

        // Skip all packages which aren't publishable
        if metadata.publish().is_never() {
            return Ok(RunStatus::Executed);
        }

        for direct_dep in metadata
            .direct_links()
            .filter(|p| !p.dev_only() && p.to().in_workspace())
        {
            // If the direct dependency isn't publishable
            if direct_dep.to().publish().is_never() {
                out.write(
                    LintLevel::Error,
                    format!(
                        "published package can't depend on unpublished package '{}'",
                        direct_dep.dep_name()
                    ),
                );
            }
        }

        Ok(RunStatus::Executed)
    }
}

/// Only allow crates to be published to crates.io
#[derive(Debug)]
pub struct OnlyPublishToCratesIo;

impl Linter for OnlyPublishToCratesIo {
    fn name(&self) -> &'static str {
        "only-publish-to-crates-io"
    }
}

impl PackageLinter for OnlyPublishToCratesIo {
    fn run<'l>(
        &self,
        ctx: &PackageContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>> {
        let metadata = ctx.metadata();

        let is_ok = match metadata.publish() {
            PackagePublish::Unrestricted => false,
            PackagePublish::Registries(&[ref registry]) => registry == PackagePublish::CRATES_IO,
            // Unpublished package.
            PackagePublish::Registries(&[]) => true,
            // Multiple registries or something else.
            _ => false,
        };

        if !is_ok {
            out.write(
                LintLevel::Error,
                "published package should only be publishable to crates.io. \
                    If you intend to publish this package, ensure the 'publish' \
                    field in the package's Cargo.toml is 'publish = [\"crates-io\"]. \
                    Otherwise set the 'publish' field to 'publish = false'.",
            );
        }

        Ok(RunStatus::Executed)
    }
}

/// Crates in the `/crates` directory have a flatten structure and their directory name is the same
/// as the crate name
#[derive(Debug)]
pub struct CratesInCratesDirectory;

impl Linter for CratesInCratesDirectory {
    fn name(&self) -> &'static str {
        "only-publish-to-crates-io"
    }
}

impl PackageLinter for CratesInCratesDirectory {
    fn run<'l>(
        &self,
        ctx: &PackageContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>> {
        let mut path_components = ctx.workspace_path().components();
        match path_components.next().map(|p| p.as_str()) {
            Some("crates") => {}
            _ => return Ok(RunStatus::Executed),
        }

        match path_components.next().map(|p| p.as_str()) {
            Some(directory) if directory == ctx.metadata().name() => {}
            _ => {
                out.write(
                    LintLevel::Error,
                    "crates in the `crates/` directory must be in a directory with the same name as the crate",
                );
            }
        }

        if path_components.next().is_some() {
            out.write(
                    LintLevel::Error,
                    "crates in the `crates/` directory must be in a flat directory structure, no nesting",
                );
        }

        Ok(RunStatus::Executed)
    }
}

// Ensure that Move crates do not depend on Diem crates.
#[derive(Debug)]
pub struct MoveCratesDontDependOnDiemCrates<'cfg> {
    config: &'cfg MoveToDiemDepsConfig,
}

impl<'cfg> MoveCratesDontDependOnDiemCrates<'cfg> {
    pub fn new(config: &'cfg MoveToDiemDepsConfig) -> Self {
        Self { config }
    }
}

impl<'cfg> Linter for MoveCratesDontDependOnDiemCrates<'cfg> {
    fn name(&self) -> &'static str {
        "move-crates-dont-depend-on-diem-crates"
    }
}

impl<'cfg> PackageLinter for MoveCratesDontDependOnDiemCrates<'cfg> {
    fn run<'l>(
        &self,
        ctx: &PackageContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>> {
        let metadata = ctx.metadata();

        let crate_name = metadata.name();
        let crate_path = metadata.source().to_string();

        // Determine if a crate is considered a Move crate or Diem crate.
        //
        // Current criteria:
        //   1. All crates outside language are considered Diem crates.
        //   2. All crates inside language are considered Move crates, unless marked otherwise.
        let is_move_crate = |crate_path: &str, crate_name: &str| {
            if crate_path.starts_with("language/") {
                if self.config.diem_crates_in_language.contains(crate_name) {
                    return false;
                }
                return true;
            }
            false
        };

        if is_move_crate(&crate_path, crate_name) {
            for direct_dep in metadata.direct_links() {
                let dep = direct_dep.to();
                let dep_name = dep.name();

                if dep.in_workspace()
                    && !self.config.exclude.contains(dep_name)
                    && !is_move_crate(&dep.source().to_string(), dep_name)
                {
                    println!("(\"{}\", \"{}\"),", crate_name, dep_name);
                    out.write(
                        LintLevel::Error,
                        format!(
                            "depending on non-move crate `{}`\n\
                                Note: all crates in language/ are considered Move crates by default. \
                                If you are creating a new Diem crate in language, you need to add its name to the \
                                diem_crates_in_language list in x.toml to make the linter recognize it.",
                            dep_name
                        ),
                    );
                }
            }
        }

        Ok(RunStatus::Executed)
    }
}
