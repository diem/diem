// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::config::TestOnlyConfig;
use guppy::graph::{BuildTargetId, BuildTargetKind};
use indoc::indoc;
use x_core::WorkspaceStatus;
use x_lint::prelude::*;

/// Ensure that every package in the workspace is classified as either a default member or test-only.
#[derive(Debug)]
pub struct DefaultOrTestOnly<'cfg> {
    config: &'cfg TestOnlyConfig,
}

impl<'cfg> DefaultOrTestOnly<'cfg> {
    pub fn new(config: &'cfg TestOnlyConfig) -> Self {
        Self { config }
    }
}

impl<'cfg> Linter for DefaultOrTestOnly<'cfg> {
    fn name(&self) -> &'static str {
        "default-or-test-only"
    }
}

impl<'cfg> PackageLinter for DefaultOrTestOnly<'cfg> {
    fn run<'l>(
        &self,
        ctx: &PackageContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>> {
        let default_members = ctx.project_ctx().default_members()?;
        let package = ctx.metadata();

        let binary_kind = package
            .build_targets()
            .filter_map(|target| {
                if matches!(target.id(), BuildTargetId::Binary(_)) {
                    return Some("binary");
                }
                // If this is the library target, then look for the first binary-equivalent-kind
                if let (BuildTargetId::Library, BuildTargetKind::LibraryOrExample(crate_types)) =
                    (target.id(), target.kind())
                {
                    return crate_types
                        .iter()
                        .filter_map(|crate_type| {
                            // These library types are equivalent to binaries.
                            if crate_type == "cdylib"
                                || crate_type == "dylib"
                                || crate_type == "staticlib"
                            {
                                Some(crate_type.as_str())
                            } else {
                                None
                            }
                        })
                        .next();
                }
                None
            })
            .next();

        let status = default_members.status_of(package.id());
        let test_only = self.config.members.contains(ctx.workspace_path());

        match (binary_kind, status, test_only) {
            (None, WorkspaceStatus::Absent, false) => {
                // Library, not reachable from default members and not marked test-only.
                let msg = indoc!(
                    "library package, not a dependency of default-members:
                     * if test-only, add to test-only in x.toml
                     * otherwise, make it a dependency of a default member (listed in root Cargo.toml)"
                );
                out.write(LintLevel::Error, msg);
            }
            (None, WorkspaceStatus::Absent, true) => {
                // Test-only library package. This is fine.
            }
            (None, WorkspaceStatus::Dependency, false) => {
                // Library, dependency of default members. This is fine.
            }
            (None, WorkspaceStatus::Dependency, true) => {
                // Library, dependency of default members and listed in test-only.
                let msg = indoc!(
                    "library package, dependency of default members and test-only:
                    * remove from test-only if production code
                    * otherwise, ensure it is not a dependency of default members"
                );
                out.write(LintLevel::Error, msg);
            }
            (None, WorkspaceStatus::RootMember, false) => {
                // Library, listed in default members. It shouldn't be.
                let msg = indoc!(
                    "library package, listed in default-members:
                     * if test-only, add to test-only in x.toml instead
                     * otherwise, remove it from default-members and make it a dependency of a binary"
                );
                out.write(LintLevel::Error, msg);
            }
            (None, WorkspaceStatus::RootMember, true) => {
                // Library, listed in default members and in test-only. It shouldn't be.
                let msg = indoc!(
                    "library package, listed in default-members and test-only:
                     * if test-only, add to test-only in x.toml and remove from default-members
                     * otherwise, remove it from both and make it a dependency of a default-member"
                );
                out.write(LintLevel::Error, msg);
            }
            (Some(kind), WorkspaceStatus::Absent, false) => {
                // Binary, not listed in default members, not test-only and not reachable from one.
                let msg = format!(
                    "{} {}",
                    kind,
                    indoc!(
                        "package, not listed in default-members:
                         * if test-only, add to test-only in x.toml
                         * otherwise, list it in root Cargo.toml's default-members"
                    ),
                );
                out.write(LintLevel::Error, msg);
            }
            (Some(_), WorkspaceStatus::Absent, true) => {
                // Test-only binary. This is fine.
            }
            (Some(kind), WorkspaceStatus::Dependency, false) => {
                // Binary, not listed in default members but reachable from one.
                let msg = format!(
                    "{} {}",
                    kind,
                    indoc!(
                        "package, not listed in default-members:
                         * list it in root Cargo.toml's default-members
                         (note: dependency of a default member, so assumed to be a production crate)"
                    ),
                );
                out.write(LintLevel::Error, msg)
            }
            (Some(kind), WorkspaceStatus::Dependency, true) => {
                // Binary, not listed in default members but a dependency of one + test-only
                let msg = format!(
                    "{} {}",
                    kind,
                    indoc!(
                        "package, not listed in default-members but a dependency, and in test-only:
                         * remove it from test-only in x.toml, AND
                         * list it in root Cargo.toml's default-members
                         (note: dependency of a default member, so assumed to be a production crate)"
                    ),
                );
                out.write(LintLevel::Error, msg)
            }
            (Some(_), WorkspaceStatus::RootMember, false) => {
                // Binary, listed in default-members. This is fine.
            }
            (Some(kind), WorkspaceStatus::RootMember, true) => {
                // Binary, listed in default-members and test-only.
                let msg = format!(
                    "{} {}",
                    kind,
                    indoc!(
                        "package, listed in both default-members and test-only:
                         * remove it from test-only in x.toml
                         (note: default member, so assumed to be a production crate)"
                    ),
                );
                out.write(LintLevel::Error, msg)
            }
        }

        Ok(RunStatus::Executed)
    }
}
