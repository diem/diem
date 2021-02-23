// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Context;
use regex::Regex;
use x_lint::prelude::*;

/// Allow certain characters in file paths.
#[derive(Debug)]
pub struct AllowedPaths {
    allowed_regex: Regex,
}

impl AllowedPaths {
    pub fn new(allowed_paths: &str) -> crate::Result<Self> {
        Ok(Self {
            allowed_regex: Regex::new(allowed_paths)
                .with_context(|| "error while parsing allowed-paths regex")?,
        })
    }
}

impl Linter for AllowedPaths {
    fn name(&self) -> &'static str {
        "allowed-paths"
    }
}

impl FilePathLinter for AllowedPaths {
    fn run<'l>(
        &self,
        ctx: &FilePathContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>> {
        if !self.allowed_regex.is_match(ctx.file_path().as_str()) {
            out.write(
                LintLevel::Error,
                format!(
                    "path doesn't match allowed regex: {}",
                    self.allowed_regex.as_str()
                ),
            );
        }

        Ok(RunStatus::Executed)
    }
}
