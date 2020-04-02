// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::lint::engine::prelude::*;

#[derive(Clone, Copy, Debug)]
pub(super) struct EofNewline;

impl Linter for EofNewline {
    fn name(&self) -> &'static str {
        "eof-newline"
    }
}

impl ContentLinter for EofNewline {
    fn pre_run<'l>(&self, file_ctx: &FileContext<'l>) -> Result<RunStatus<'l>> {
        Ok(skip_whitespace_checks(file_ctx.extension()))
    }

    fn run<'l>(
        &self,
        ctx: &ContentContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>> {
        let content = match ctx.content() {
            Some(text) => text,
            None => return Ok(RunStatus::Skipped(SkipReason::NonUtf8)),
        };
        if !content.ends_with('\n') {
            out.write(LintMessage::new(LintLevel::Error, "missing newline at EOF"));
        }
        Ok(RunStatus::Executed)
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct TrailingWhitespace;

impl Linter for TrailingWhitespace {
    fn name(&self) -> &'static str {
        "trailing-whitespace"
    }
}

impl ContentLinter for TrailingWhitespace {
    fn pre_run<'l>(&self, file_ctx: &FileContext<'l>) -> Result<RunStatus<'l>> {
        Ok(skip_whitespace_checks(file_ctx.extension()))
    }

    fn run<'l>(
        &self,
        ctx: &ContentContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>> {
        let content = match ctx.content() {
            Some(text) => text,
            None => return Ok(RunStatus::Skipped(SkipReason::NonUtf8)),
        };

        for (ln, line) in content.lines().enumerate().map(|(ln, line)| (ln + 1, line)) {
            if line.trim_end() != line {
                out.write(LintMessage::new(
                    LintLevel::Error,
                    format!("trailing whitespace at line {}", ln),
                ));
            }
        }

        if content
            .lines()
            .rev()
            .take_while(|line| line.is_empty())
            .count()
            > 0
        {
            out.write(LintMessage::new(
                LintLevel::Error,
                "trailing whitespace at EOF",
            ));
        }

        Ok(RunStatus::Executed)
    }
}

fn skip_whitespace_checks(ext: Option<&str>) -> RunStatus {
    match ext {
        Some("exp") => RunStatus::Skipped(SkipReason::UnsupportedExtension(ext)),
        _ => RunStatus::Executed,
    }
}
