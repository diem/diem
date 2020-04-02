// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::lint::engine::{prelude::*, LintContext};
use once_cell::sync::OnceCell;
use std::{
    ffi::OsStr,
    path::Path,
    process::{Command, Stdio},
};

/// Configuration for the lint engine.
#[derive(Clone, Debug)]
pub struct LintEngineConfig<'cfg> {
    project_root: &'cfg Path,
    content_linters: &'cfg [&'cfg (dyn ContentLinter + 'cfg)],
    fail_fast: bool,
}

impl<'cfg> LintEngineConfig<'cfg> {
    pub fn new(project_root: &'cfg Path) -> Self {
        Self {
            project_root,
            content_linters: &[],
            fail_fast: false,
        }
    }

    pub fn with_content_linters(
        &mut self,
        content_linters: &'cfg [&'cfg (dyn ContentLinter + 'cfg)],
    ) -> &mut Self {
        self.content_linters = content_linters;
        self
    }

    pub fn fail_fast(&mut self, fail_fast: bool) -> &mut Self {
        self.fail_fast = fail_fast;
        self
    }

    pub fn build(&self) -> LintEngine<'cfg> {
        LintEngine::new(self.clone())
    }
}

/// Executor for linters.
#[derive(Debug)]
pub struct LintEngine<'cfg> {
    config: LintEngineConfig<'cfg>,

    // Caches to allow results to live as long as the engine itself.
    files_stdout: OnceCell<Vec<u8>>,
}

impl<'cfg> LintEngine<'cfg> {
    pub fn new(config: LintEngineConfig<'cfg>) -> Self {
        Self {
            config,
            files_stdout: OnceCell::new(),
        }
    }

    pub fn run(&self) -> Result<LintResults> {
        let mut skipped = vec![];
        let mut messages = vec![];

        // TODO: add support for project and file lints.

        let project_ctx = ProjectContext::new(self.config.project_root);

        if !self.config.content_linters.is_empty() {
            let file_list = self.get_file_list()?;

            // TODO: This should probably be a worker queue with a thread pool or something.

            let file_ctxs = file_list
                .iter()
                .map(|path| FileContext::new(project_ctx, path));

            'outer: for file_ctx in file_ctxs {
                let linters_to_run = self
                    .config
                    .content_linters
                    .iter()
                    .copied()
                    .filter_map(|linter| match linter.pre_run(&file_ctx) {
                        Ok(RunStatus::Executed) => Some(Ok(linter)),
                        Ok(RunStatus::Skipped(reason)) => {
                            let source = file_ctx.source(linter.name());
                            skipped.push((source, reason));
                            None
                        }
                        Err(err) => Some(Err(err)),
                    })
                    .collect::<Result<Vec<_>>>()?;

                if linters_to_run.is_empty() {
                    // No linters to run for this file -- no point loading it.
                    continue;
                }

                // Load up the content for this file.
                let content_ctx = file_ctx.load()?;

                for linter in linters_to_run {
                    let source = content_ctx.source(linter.name());
                    let mut formatter = LintFormatter::new(source, &mut messages);

                    match linter.run(&content_ctx, &mut formatter)? {
                        RunStatus::Executed => {
                            // Yay! Lint ran successfully.
                        }
                        RunStatus::Skipped(reason) => {
                            skipped.push((source, reason));
                        }
                    }

                    if self.config.fail_fast && !messages.is_empty() {
                        // At least one issue was found.
                        break 'outer;
                    }
                }
            }
        }

        Ok(LintResults { skipped, messages })
    }

    // ---
    // Helper methods
    // ---

    fn get_file_list(&self) -> Result<Vec<&Path>> {
        // TODO: It'd be pretty cool to be able to cache the `&Path` instances as well. The options
        // are:
        // 1. Cache them as `PathBuf` instances. That would involve lots of allocations.
        // 2. Store the `&Path` instances alongside `files_stdout`. That would be a self-referential
        //    struct (so would require the use of something like
        //    [`rental`](https://docs.rs/rental/).
        // 3. Reorganize the code to use some sort of arena allocator which is passed into the lint
        //    context (and/or stored alongside with `rental`.)
        let stdout = self.files_stdout()?;

        Ok(stdout
            .split(|b| b == &0)
            .filter_map(|s| {
                // TODO: make global exclusions configurable.
                if !s.is_empty() && !s.starts_with(b"testsuite/libra-fuzzer/artifacts/") {
                    // `OsStr::from_bytes` only works on Unix, since "OS strings" on Windows are
                    // actually UTF-16ish. Would be cool to get it working on Windows at some point
                    // too, but it needs to be done with some care.
                    //
                    // For more, see:
                    // * https://doc.rust-lang.org/std/ffi/index.html#conversions
                    // * https://www.mercurial-scm.org/wiki/EncodingStrategy and
                    // * https://en.wikipedia.org/wiki/Mojibake.
                    use std::os::unix::ffi::OsStrExt;

                    let s = OsStr::from_bytes(s);
                    Some(Path::new(s))
                } else {
                    None
                }
            })
            .collect())
    }

    fn files_stdout(&self) -> Result<&[u8]> {
        self.files_stdout
            .get_or_try_init(|| {
                // TODO: abstract out SCM and command-running functionality.
                let output = Command::new("git")
                    .current_dir(self.config.project_root)
                    // The -z causes files to not be quoted, and to be separated by \0.
                    .args(&["ls-files", "-z"])
                    .stderr(Stdio::inherit())
                    .output()?;
                if !output.status.success() {
                    return Err(SystemError::Exec {
                        cmd: "git ls-files",
                        status: output.status,
                    });
                }
                Ok(output.stdout)
            })
            .map(|stdout| stdout.as_slice())
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub struct LintResults<'l> {
    pub skipped: Vec<(LintSource<'l>, SkipReason<'l>)>,
    pub messages: Vec<(LintSource<'l>, LintMessage)>,
}
