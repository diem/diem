// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::anyhow;
use guppy::PackageId;
use hakari::HakariBuilder;
use x_core::XCoreContext;
use x_lint::prelude::*;

/// Check that the workspace hack is up-to-date.
#[derive(Debug)]
pub struct GenerateWorkspaceHack;

impl Linter for GenerateWorkspaceHack {
    fn name(&self) -> &'static str {
        "generate-workspace-hack"
    }
}

impl ProjectLinter for GenerateWorkspaceHack {
    fn run<'l>(
        &self,
        ctx: &ProjectContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>> {
        let hakari = ctx.hakari()?;
        let existing_toml = hakari
            .read_toml()
            .expect("workspace-hack package was specified")
            .map_err(|err| {
                SystemError::hakari_cargo_toml("while reading workspace-hack Cargo.toml", err)
            })?;
        let new_toml = hakari
            .to_toml_string(&ctx.core().hakari_toml_options())
            .map_err(|err| {
                SystemError::hakari_toml_out("while generating expected Cargo.toml", err)
            })?;
        if existing_toml.is_changed(&new_toml) {
            let patch = existing_toml.diff_toml(&new_toml);
            // TODO: add global coloring options to x
            let formatter = hakari::diffy::PatchFormatter::new().with_color();
            let diff = formatter.fmt_patch(&patch);
            let hakari_package = hakari
                .builder()
                .hakari_package()
                .expect("package was specified at build time");
            out.write(
                LintLevel::Error,
                format!(
                    "existing contents of {}/Cargo.toml do not match generated contents\n\n\
                * diff:\n\
                {}\n\n\
                * run \"cargo x generate-workspace-hack\" to regenerate",
                    hakari_package
                        .source()
                        .workspace_path()
                        .expect("hakari package is in workspace"),
                    diff
                ),
            );
        }

        Ok(RunStatus::Executed)
    }
}

/// Ensure the workspace-hack package is a dependency
#[derive(Debug)]
pub struct WorkspaceHackDep<'cfg> {
    hakari_builder: HakariBuilder<'cfg>,
    hakari_id: &'cfg PackageId,
}

impl<'cfg> WorkspaceHackDep<'cfg> {
    pub fn new(ctx: &'cfg XCoreContext) -> crate::Result<Self> {
        let hakari_builder = ctx.hakari_builder()?;
        let hakari_id = hakari_builder
            .hakari_package()
            .map(|package| package.id())
            .ok_or_else(|| anyhow!("hakari.hakari-package not specified in x.toml"))?;
        Ok(Self {
            hakari_builder,
            hakari_id,
        })
    }
}
impl<'cfg> Linter for WorkspaceHackDep<'cfg> {
    fn name(&self) -> &'static str {
        "workspace-hack-dep"
    }
}

impl<'cfg> PackageLinter for WorkspaceHackDep<'cfg> {
    fn run<'l>(
        &self,
        ctx: &PackageContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>> {
        let package = ctx.metadata();
        let pkg_graph = ctx.package_graph();

        // Exclude omitted packages (including the workspace-hack package itself) from consideration.
        if self
            .hakari_builder
            .omits_package(package.id())
            .expect("valid package ID")
        {
            return Ok(RunStatus::Executed);
        }

        let has_links = package.direct_links().next().is_some();
        let has_hack_dep = pkg_graph
            .directly_depends_on(package.id(), self.hakari_id)
            .expect("valid package ID");
        if has_links && !has_hack_dep {
            out.write(LintLevel::Error, "missing diem-workspace-hack dependency");
        }

        Ok(RunStatus::Executed)
    }
}
