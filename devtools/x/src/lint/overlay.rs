// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::config::OverlayConfig;
use guppy::graph::feature::FeatureFilterFn;
use std::iter;
use x_lint::prelude::*;

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
