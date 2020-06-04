// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::config::{OverlayConfig, TestOnlyConfig};
use guppy::graph::feature::{default_filter, FeatureFilterFn, FeatureQuery};
use guppy::graph::DependencyDirection;
use guppy::DependencyKind;
use std::collections::BTreeMap;
use std::iter;
use x_lint::prelude::*;

/// Assertions for "overlay" features.
///
/// An "overlay" feature is a feature name used throughout the codebase, whose purpose it is to
/// augment each package with extra code (e.g. proptest generators). Overlay features shouldn't be
/// enabled by anything except overlay features on workspace members, but must be enabled by default
/// by test-only workspace members.
#[derive(Debug)]
pub struct OverlayFeatures<'cfg> {
    overlay_config: &'cfg OverlayConfig,
    test_only_config: &'cfg TestOnlyConfig,
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
        if ctx.is_default_member() {
            self.default_lint(ctx, out)
        } else if self.test_only_config.members.contains(ctx.workspace_path()) {
            self.test_only_lint(ctx, out)
        } else {
            // Neither a default member nor test-only. The test-only lint will catch this.
            Ok(RunStatus::Skipped(SkipReason::UnsupportedPackage(
                ctx.metadata().id(),
            )))
        }
    }
}

impl<'cfg> OverlayFeatures<'cfg> {
    pub fn new(
        overlay_config: &'cfg OverlayConfig,
        test_only_config: &'cfg TestOnlyConfig,
    ) -> Self {
        Self {
            overlay_config,
            test_only_config,
        }
    }

    fn is_overlay(&self, feature: Option<&str>) -> bool {
        match feature {
            Some(feature) => self.overlay_config.features.iter().any(|f| *f == feature),
            // The base feature isn't banned.
            None => false,
        }
    }

    fn default_lint<'l>(
        &self,
        ctx: &PackageContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>> {
        let package = ctx.metadata();
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

    /// Ensure that links from test-only to default members always enable the fuzzing feature if
    /// available.
    fn test_only_lint<'l>(
        &self,
        ctx: &PackageContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>> {
        let package = ctx.metadata();
        let package_graph = ctx.package_graph();
        let feature_graph = package_graph.feature_graph();

        let package_query = package_graph
            .query_forward(iter::once(package.id()))
            .expect("valid package ID");
        // Do the query with default features.
        // TODO: should this be stricter -- no features?
        let feature_query = feature_graph.query_packages(&package_query, default_filter());

        let missing_overlays = self.missing_overlays(ctx, feature_query)?;

        if !missing_overlays.is_empty() {
            let mut msg = "missing overlay features from test-only package:\n".to_string();
            for (name, (kind, overlay_feature)) in &missing_overlays {
                msg.push_str(&format!(
                    "  * {} dependency '{}' missing feature '{}'\n",
                    kind, name, overlay_feature
                ));
            }
            msg.push_str(
                "Enable overlay features in test-only packages through [dependencies] etc.",
            );
            out.write(LintLevel::Error, msg);
        }

        Ok(RunStatus::Executed)
    }

    fn missing_overlays<'l>(
        &self,
        ctx: &PackageContext<'l>,
        feature_query: FeatureQuery<'l>,
    ) -> Result<BTreeMap<&'l str, (DependencyKind, &'cfg str)>> {
        let package = ctx.metadata();
        let feature_graph = ctx.package_graph().feature_graph();
        let default_members = ctx.project_ctx().default_members()?;

        // TODO: guppy should provide an easier way to get all features for a package.
        let all_feature_set = feature_graph.resolve_all();

        let mut missing_overlays = BTreeMap::new();

        for kind in &[DependencyKind::Normal, DependencyKind::Build] {
            let feature_set = feature_query.clone().resolve_with_fn(|_, link| {
                let (from, to) = link.endpoints();

                // Look for edges to default members.
                let to_package = to.package();
                let consider_link = from.package().id() == package.id()
                    && to_package.in_workspace()
                    && default_members.status_of(to_package.id()).is_present();
                consider_link && link.status_for_kind(*kind).is_present()
            });

            // Check that the feature set for all members includes overlay features if available.
            for feature_list in feature_set.packages_with_features(DependencyDirection::Forward) {
                let package = feature_list.package();
                if !default_members.status_of(package.id()).is_present() {
                    // Skip non-default members.
                    continue;
                }

                let all_features = all_feature_set
                    .features_for(package.id())
                    .expect("all_feature_set contains all packages");
                for overlay_feature in &self.overlay_config.features {
                    if all_features.contains(overlay_feature)
                        && !feature_list.contains(overlay_feature)
                    {
                        missing_overlays.insert(package.name(), (*kind, overlay_feature.as_str()));
                    }
                }
            }
        }

        Ok(missing_overlays)
    }
}

fn feature_str(feature: Option<&str>) -> &str {
    match feature {
        Some(feature) => feature,
        None => "[base]",
    }
}
