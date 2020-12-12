// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{changed_since::changed_since_impl, context::XContext, Result};
use anyhow::anyhow;
use guppy::graph::DependencyDirection;
use std::collections::BTreeSet;
use structopt::StructOpt;

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
}

impl SelectedPackageArgs {
    pub fn to_selected_packages<'a>(&'a self, xctx: &'a XContext) -> Result<SelectedPackages<'a>> {
        // Mutually exclusive options -- only one of these can be provided.
        {
            let mut exclusive = vec![];
            if self.changed_since.is_some() {
                exclusive.push("--changed-since");
            }
            if !self.package.is_empty() {
                exclusive.push("--package");
            }
            if self.workspace {
                exclusive.push("--workspace");
            }

            if exclusive.len() > 1 {
                let err_msg = exclusive.join(", ");
                return Err(anyhow!("can only specify one of {}", err_msg));
            }
        }

        if self.workspace {
            Ok(SelectedPackages::workspace())
        } else if !self.package.is_empty() {
            Ok(SelectedPackages::includes(
                self.package.iter().map(|s| s.as_str()),
            ))
        } else if let Some(base) = &self.changed_since {
            let affected_set = changed_since_impl(&xctx, &base)?;

            Ok(SelectedPackages::includes(
                affected_set
                    .packages(DependencyDirection::Forward)
                    .map(|package| package.name()),
            ))
        } else {
            SelectedPackages::default_cwd(xctx)
        }
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
    /// Returns a new `CargoPackages` that selects all packages in this workspace.
    pub fn workspace() -> Self {
        Self {
            includes: SelectedInclude::Workspace,
            excludes: BTreeSet::new(),
        }
    }

    /// Returns a new `CargoPackages` that selects the default set of packages for the current
    /// working directory. This may either be the entire workspace or a set of packages inside the
    /// workspace.
    pub fn default_cwd(xctx: &'a XContext) -> Result<Self> {
        let includes = if xctx.core().current_dir_is_root() {
            SelectedInclude::Workspace
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
            SelectedInclude::Includes(selected.collect())
        };
        Ok(Self {
            includes,
            excludes: BTreeSet::new(),
        })
    }

    /// Returns a new `CargoPackages` that selects the specified packages.
    pub fn includes(package_names: impl IntoIterator<Item = &'a str>) -> Self {
        Self {
            includes: SelectedInclude::Includes(package_names.into_iter().collect()),
            excludes: BTreeSet::new(),
        }
    }

    /// Adds excludes for this `CargoPackages`.
    ///
    /// The excludes are currently ignored if the local package is built.
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
    Includes(Vec<&'a str>),
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_should_invoke() {
        let packages = SelectedPackages::workspace();
        assert!(packages.should_invoke(), "workspace => invoke");

        let mut packages = SelectedPackages::includes(vec!["foo", "bar"]);
        packages.add_excludes(vec!["foo"]);
        assert!(packages.should_invoke(), "non-empty packages => invoke");

        let packages = SelectedPackages::includes(vec![]);
        assert!(!packages.should_invoke(), "no packages => do not invoke");

        let mut packages = SelectedPackages::includes(vec!["foo", "bar"]);
        packages.add_excludes(vec!["foo", "bar"]);
        assert!(
            !packages.should_invoke(),
            "all packages excluded => do not invoke"
        );
    }
}
