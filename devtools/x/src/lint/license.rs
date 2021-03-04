// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use x_lint::prelude::*;

static LICENSE_HEADER: &str = "Copyright (c) The Diem Core Contributors\n\
                               SPDX-License-Identifier: Apache-2.0\n\
                               ";

#[derive(Copy, Clone, Debug)]
pub(super) struct LicenseHeader;

impl Linter for LicenseHeader {
    fn name(&self) -> &'static str {
        "license-header"
    }
}

impl ContentLinter for LicenseHeader {
    fn pre_run<'l>(&self, file_ctx: &FilePathContext<'l>) -> Result<RunStatus<'l>> {
        // TODO: Add a way to pass around state between pre_run and run, so that this computation
        // only needs to be done once.
        match FileType::new(file_ctx) {
            Some(_) => Ok(RunStatus::Executed),
            None => Ok(RunStatus::Skipped(SkipReason::UnsupportedExtension(
                file_ctx.extension(),
            ))),
        }
    }

    fn run<'l>(
        &self,
        ctx: &ContentContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>> {
        let content = match ctx.content() {
            Some(content) => content,
            None => {
                // This is not a UTF-8 file -- don't analyze it.
                return Ok(RunStatus::Skipped(SkipReason::NonUtf8Content));
            }
        };

        let file_type = FileType::new(ctx.file_ctx()).expect("None filtered out in pre_run");
        // Determine if the file is missing the license header
        let missing_header = match file_type {
            FileType::Rust | FileType::Proto => {
                let maybe_license = content
                    .lines()
                    .skip_while(|line| line.is_empty())
                    .take(2)
                    .map(|s| s.trim_start_matches("// "));
                !LICENSE_HEADER.lines().eq(maybe_license)
            }
            FileType::Shell => {
                let maybe_license = content
                    .lines()
                    .skip_while(|line| line.starts_with("#!"))
                    .skip_while(|line| line.is_empty())
                    .take(2)
                    .map(|s| s.trim_start_matches("# "));
                !LICENSE_HEADER.lines().eq(maybe_license)
            }
        };

        if missing_header {
            out.write(LintLevel::Error, "missing license header");
        }

        Ok(RunStatus::Executed)
    }
}

enum FileType {
    Rust,
    Shell,
    Proto,
}

impl FileType {
    fn new(ctx: &FilePathContext<'_>) -> Option<Self> {
        match ctx.extension() {
            Some("rs") => Some(FileType::Rust),
            Some("sh") => Some(FileType::Shell),
            Some("proto") => Some(FileType::Proto),
            _ => None,
        }
    }
}
