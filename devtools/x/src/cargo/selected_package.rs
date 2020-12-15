// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{changed_since::changed_since_impl, context::XContext, Result};
use anyhow::anyhow;
use guppy::graph::DependencyDirection;
use std::collections::BTreeSet;
use structopt::StructOpt;
use x_core::WorkspaceStatus;

/// Arguments for the Cargo package selector.
#[derive(Debug, StructOpt)]
pub struct SelectedPackageArgs {
    #[structopt(long, short, number_of_values = 1)]
    /// Run on the provided packages
    pub(crate) package: Vec<String>,
    #[structopt(long, short)]
    /// Run on packages changed since the merge base of this commit
    changed_since: Option<String>,
    #[structopt(long)]
    /// Run on all packages in the workspace
    pub(crate) workspace: bool,
    #[structopt(long)]
    /// Run on production code (non-test-only code, reachable from default members)
    pub(crate) production: bool,
}

impl SelectedPackageArgs {
    pub fn to_selected_packages<'a>(&'a self, xctx: &'a XContext) -> Result<SelectedPackages<'a>> {
        // Mutually exclusive options -- only one of these can be provided.
        {
            let mut exclusive = vec![];
            if !self.package.is_empty() {
                exclusive.push("--package");
            }
            if self.workspace {
                exclusive.push("--workspace");
            }
            if self.production {
                exclusive.push("--production");
            }

            if exclusive.len() > 1 {
                let err_msg = exclusive.join(", ");
                return Err(anyhow!("can only specify one of {}", err_msg));
            }
        }

        let mut includes = if self.workspace {
            SelectedInclude::Workspace
        } else if !self.package.is_empty() {
            SelectedInclude::includes(self.package.iter().map(|s| s.as_str()))
        } else if self.production {
            SelectedInclude::production(xctx)?
        } else {
            SelectedInclude::default_cwd(xctx)?
        };

        // Intersect with --changed-since if specified.
        if let Some(base) = &self.changed_since {
            let affected_set = changed_since_impl(&xctx, &base)?;

            includes = includes.intersection(
                affected_set
                    .packages(DependencyDirection::Forward)
                    .map(|package| package.name()),
            );
        }

        Ok(SelectedPackages::new(includes))
    }
}

/// Package selector for Cargo commands.
///
/// This may represent any of the following:
/// * the entire workspace
/// * a single package without arguments
/// * a list of packages
///
/// This may also exclude a set of packages. Note that currently, excludes only work in the "entire
/// workspace" and "list of packages" situations. They are ignored if a specific local package is
/// being built. (This is an extension on top of Cargo itself, which only supports --exclude
/// together with --workspace.)
///
/// Excludes are applied after includes. This allows changed-since to support excludes, even if only
/// a subset of the workspace changes.
#[derive(Clone, Debug)]
pub struct SelectedPackages<'a> {
    pub(super) includes: SelectedInclude<'a>,
    pub(super) excludes: BTreeSet<&'a str>,
}

impl<'a> SelectedPackages<'a> {
    pub(super) fn new(includes: SelectedInclude<'a>) -> Self {
        Self {
            includes,
            excludes: BTreeSet::new(),
        }
    }

    /// Adds excludes for this `SelectedPackages`.
    pub fn add_excludes(&mut self, exclude_names: impl IntoIterator<Item = &'a str>) -> &mut Self {
        self.excludes.extend(exclude_names);
        self
    }

    // ---
    // Helper methods
    // ---

    pub(super) fn should_invoke(&self) -> bool {
        match &self.includes {
            SelectedInclude::Workspace => true,
            SelectedInclude::Includes(includes) => {
                // If everything in the include set is excluded, a command invocation isn't needed.
                includes.iter().any(|p| !self.excludes.contains(p))
            }
        }
    }
}

#[derive(Clone, Debug)]
pub(super) enum SelectedInclude<'a> {
    Workspace,
    Includes(BTreeSet<&'a str>),
}

impl<'a> SelectedInclude<'a> {
    /// Returns a `SelectedInclude` that selects production crates (non-test-only code).
    pub fn production(xctx: &'a XContext) -> Result<Self> {
        let default_members = xctx.core().subsets()?.default_members();
        let workspace = xctx.core().package_graph()?.workspace();

        let selected = workspace.iter().filter_map(|package| {
            if default_members.status_of(package.id()) != WorkspaceStatus::Absent {
                Some(package.name())
            } else {
                None
            }
        });
        Ok(SelectedInclude::Includes(selected.collect()))
    }

    /// Returns a `SelectedInclude` that selects the default set of packages for the current
    /// working directory. This may either be the entire workspace or a set of packages inside the
    /// workspace.
    pub fn default_cwd(xctx: &'a XContext) -> Result<Self> {
        if xctx.core().current_dir_is_root() {
            Ok(SelectedInclude::Workspace)
        } else {
            // Select all packages that begin with the current rel dir.
            let rel = xctx.core().current_rel_dir();
            let workspace = xctx.core().package_graph()?.workspace();
            let selected = workspace.iter_by_path().filter_map(|(path, package)| {
                // If we're in devtools, run tests for all packages inside devtools.
                // If we're in devtools/x/src, run tests for devtools/x.
                if path.starts_with(rel) || rel.starts_with(path) {
                    Some(package.name())
                } else {
                    None
                }
            });
            Ok(SelectedInclude::Includes(selected.collect()))
        }
    }

    /// Returns a `SelectedInclude` that selects the specified packages.
    pub fn includes(package_names: impl IntoIterator<Item = &'a str>) -> Self {
        SelectedInclude::Includes(package_names.into_iter().collect())
    }

    /// Intersects this `SelectedInclude` with the given names.
    pub fn intersection(&self, names: impl IntoIterator<Item = &'a str>) -> Self {
        let names = names.into_iter().collect();
        match self {
            SelectedInclude::Workspace => SelectedInclude::Includes(names),
            SelectedInclude::Includes(includes) => {
                SelectedInclude::Includes(includes.intersection(&names).copied().collect())
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_should_invoke() {
        let packages = SelectedPackages::new(SelectedInclude::Workspace);
        assert!(packages.should_invoke(), "workspace => invoke");

        let mut packages = SelectedPackages::new(SelectedInclude::Includes(
            vec!["foo", "bar"].into_iter().collect(),
        ));
        packages.add_excludes(vec!["foo"]);
        assert!(packages.should_invoke(), "non-empty packages => invoke");

        let packages = SelectedPackages::new(SelectedInclude::Includes(BTreeSet::new()));
        assert!(!packages.should_invoke(), "no packages => do not invoke");

        let mut packages = SelectedPackages::new(SelectedInclude::Includes(
            vec!["foo", "bar"].into_iter().collect(),
        ));
        packages.add_excludes(vec!["foo", "bar"]);
        assert!(
            !packages.should_invoke(),
            "all packages excluded => do not invoke"
        );
    }
}
