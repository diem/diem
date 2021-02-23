// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Context;
use globset::{Glob, GlobSet, GlobSetBuilder};
use x_lint::prelude::*;

#[derive(Clone, Copy, Debug)]
pub(super) struct EofNewline<'cfg> {
    exceptions: &'cfg GlobSet,
}

impl<'cfg> EofNewline<'cfg> {
    pub fn new(exceptions: &'cfg GlobSet) -> Self {
        Self { exceptions }
    }
}

impl<'cfg> Linter for EofNewline<'cfg> {
    fn name(&self) -> &'static str {
        "eof-newline"
    }
}

impl<'cfg> ContentLinter for EofNewline<'cfg> {
    fn pre_run<'l>(&self, file_ctx: &FilePathContext<'l>) -> Result<RunStatus<'l>> {
        Ok(skip_whitespace_checks(self.exceptions, file_ctx))
    }

    fn run<'l>(
        &self,
        ctx: &ContentContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>> {
        let content = match ctx.content() {
            Some(text) => text,
            None => return Ok(RunStatus::Skipped(SkipReason::NonUtf8Content)),
        };
        if !content.is_empty() && !content.ends_with('\n') {
            out.write(LintLevel::Error, "missing newline at EOF");
        }
        Ok(RunStatus::Executed)
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct TrailingWhitespace<'cfg> {
    exceptions: &'cfg GlobSet,
}

impl<'cfg> TrailingWhitespace<'cfg> {
    pub fn new(exceptions: &'cfg GlobSet) -> Self {
        Self { exceptions }
    }
}

impl<'cfg> Linter for TrailingWhitespace<'cfg> {
    fn name(&self) -> &'static str {
        "trailing-whitespace"
    }
}

impl<'cfg> ContentLinter for TrailingWhitespace<'cfg> {
    fn pre_run<'l>(&self, file_ctx: &FilePathContext<'l>) -> Result<RunStatus<'l>> {
        Ok(skip_whitespace_checks(self.exceptions, file_ctx))
    }

    fn run<'l>(
        &self,
        ctx: &ContentContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>> {
        let content = match ctx.content() {
            Some(text) => text,
            None => return Ok(RunStatus::Skipped(SkipReason::NonUtf8Content)),
        };

        for (ln, line) in content.lines().enumerate().map(|(ln, line)| (ln + 1, line)) {
            if line.trim_end() != line {
                out.write(
                    LintLevel::Error,
                    format!("trailing whitespace at line {}", ln),
                );
            }
        }

        if content
            .lines()
            .rev()
            .take_while(|line| line.is_empty())
            .count()
            > 0
        {
            out.write(LintLevel::Error, "trailing whitespace at EOF");
        }

        Ok(RunStatus::Executed)
    }
}

pub(super) fn build_exceptions(patterns: &[String]) -> crate::Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let glob = Glob::new(pattern).with_context(|| {
            format!(
                "error while processing whitespace exception glob '{}'",
                pattern
            )
        })?;
        builder.add(glob);
    }
    builder
        .build()
        .with_context(|| "error while building globset for whitespace patterns")
}

fn skip_whitespace_checks<'l>(exceptions: &GlobSet, file: &FilePathContext<'l>) -> RunStatus<'l> {
    if exceptions.is_match(file.file_path()) {
        return RunStatus::Skipped(SkipReason::UnsupportedFile(file.file_path()));
    }

    RunStatus::Executed
}
