// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

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
                        .expect("hakari package is in workspace")
                        .display(),
                    diff
                ),
            );
        }

        Ok(RunStatus::Executed)
    }
}
