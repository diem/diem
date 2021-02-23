// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use camino::Utf8Path;
use colored_diff::PrettyDifference;
use serde::{Deserialize, Serialize};
use toml::{de, ser};
use x_lint::prelude::*;

/// Checks on the root toml.
#[derive(Debug)]
pub struct RootToml;

impl Linter for RootToml {
    fn name(&self) -> &'static str {
        "root-toml"
    }
}

impl ContentLinter for RootToml {
    fn pre_run<'l>(&self, file_ctx: &FilePathContext<'l>) -> Result<RunStatus<'l>> {
        let file_path = file_ctx.file_path();
        if file_path == "Cargo.toml" {
            Ok(RunStatus::Executed)
        } else {
            Ok(RunStatus::Skipped(SkipReason::UnsupportedFile(file_path)))
        }
    }

    fn run<'l>(
        &self,
        ctx: &ContentContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>> {
        let contents: RootTomlContents<'_> = de::from_slice(ctx.content_bytes())
            .map_err(|err| SystemError::de("deserializing root Cargo.toml", err))?;
        let workspace = contents.workspace;

        // Use guppy to produce a canonical list of workspace paths.
        // This does two things:
        // * ensure that workspace members are sorted.
        // * ensure that every workspace member is listed. Cargo itself doesn't require every
        //   workspace member to be listed (just leaf packages), but other tools might.
        let package_graph = ctx.file_ctx().project_ctx().package_graph()?;
        let expected = Workspace {
            members: package_graph
                .workspace()
                .iter_by_path()
                .map(|(path, _)| path)
                .collect(),
        };

        if workspace.members != expected.members {
            out.write(
                LintLevel::Error,
                toml_mismatch_message(
                    &expected,
                    &workspace,
                    "workspace member list not canonical",
                )?,
            );
        }

        // TODO: autofix support would be really nice!

        Ok(RunStatus::Executed)
    }
}

/// Creates a lint message indicating the differences between the two TOML structs.
pub(super) fn toml_mismatch_message<T: Serialize>(
    expected: &T,
    actual: &T,
    header: &str,
) -> Result<String> {
    let expected = to_toml_string(expected)
        .map_err(|err| SystemError::ser("serializing expected workspace members", err))?;
    let actual = to_toml_string(actual)
        .map_err(|err| SystemError::ser("serializing actual workspace members", err))?;
    // TODO: print out a context diff instead of the full diff.
    Ok(format!(
        "{}:\n\n{}",
        header,
        PrettyDifference {
            expected: &expected,
            actual: &actual
        }
    ))
}

/// Serializes some data to toml using this project's standard code style.
fn to_toml_string<T: Serialize>(data: &T) -> Result<String, ser::Error> {
    let mut dst = String::with_capacity(128);
    let mut serializer = ser::Serializer::new(&mut dst);
    serializer
        .pretty_array(true)
        .pretty_array_indent(4)
        .pretty_array_trailing_comma(true)
        .pretty_string_literal(true)
        .pretty_string(false);
    data.serialize(&mut serializer)?;
    Ok(dst)
}

#[derive(Debug, Deserialize)]
struct RootTomlContents<'a> {
    #[serde(borrow)]
    workspace: Workspace<'a>,
    // Add other fields as necessary.
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Workspace<'a> {
    #[serde(borrow)]
    members: Vec<&'a Utf8Path>,
    // Add other fields as necessary.
}
