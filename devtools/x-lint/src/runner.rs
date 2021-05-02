// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{prelude::*, LintContext};
use camino::Utf8Path;
use x_core::XCoreContext;

/// Configuration for the lint engine.
#[derive(Clone, Debug)]
pub struct LintEngineConfig<'cfg> {
    core: &'cfg XCoreContext,
    project_linters: &'cfg [&'cfg dyn ProjectLinter],
    package_linters: &'cfg [&'cfg dyn PackageLinter],
    file_path_linters: &'cfg [&'cfg dyn FilePathLinter],
    content_linters: &'cfg [&'cfg dyn ContentLinter],
    fail_fast: bool,
}

impl<'cfg> LintEngineConfig<'cfg> {
    pub fn new(core: &'cfg XCoreContext) -> Self {
        Self {
            core,
            project_linters: &[],
            package_linters: &[],
            file_path_linters: &[],
            content_linters: &[],
            fail_fast: false,
        }
    }

    pub fn with_project_linters(
        &mut self,
        project_linters: &'cfg [&'cfg dyn ProjectLinter],
    ) -> &mut Self {
        self.project_linters = project_linters;
        self
    }

    pub fn with_package_linters(
        &mut self,
        package_linters: &'cfg [&'cfg dyn PackageLinter],
    ) -> &mut Self {
        self.package_linters = package_linters;
        self
    }

    pub fn with_file_path_linters(
        &mut self,
        file_path_linters: &'cfg [&'cfg dyn FilePathLinter],
    ) -> &mut Self {
        self.file_path_linters = file_path_linters;
        self
    }

    pub fn with_content_linters(
        &mut self,
        content_linters: &'cfg [&'cfg dyn ContentLinter],
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
    project_ctx: ProjectContext<'cfg>,
}

impl<'cfg> LintEngine<'cfg> {
    pub fn new(config: LintEngineConfig<'cfg>) -> Self {
        let project_ctx = ProjectContext::new(config.core);
        Self {
            config,
            project_ctx,
        }
    }

    pub fn run(&self) -> Result<LintResults> {
        let mut skipped = vec![];
        let mut messages = vec![];

        // TODO: add support for file linters.

        // Run project linters.
        if !self.config.project_linters.is_empty() {
            for linter in self.config.project_linters {
                let source = self.project_ctx.source(linter.name());
                let mut formatter = LintFormatter::new(source, &mut messages);
                match linter.run(&self.project_ctx, &mut formatter)? {
                    RunStatus::Executed => {
                        // Lint ran successfully.
                    }
                    RunStatus::Skipped(reason) => {
                        skipped.push((source, reason));
                    }
                }

                if self.config.fail_fast && !messages.is_empty() {
                    // At least one issue was found.
                    return Ok(LintResults { skipped, messages });
                }
            }
        }

        // Run package linters.
        if !self.config.package_linters.is_empty() {
            let package_graph = self.project_ctx.package_graph()?;

            for (workspace_path, metadata) in package_graph.workspace().iter_by_path() {
                let package_ctx = PackageContext::new(
                    &self.project_ctx,
                    package_graph,
                    workspace_path,
                    metadata,
                )?;
                for linter in self.config.package_linters {
                    let source = package_ctx.source(linter.name());
                    let mut formatter = LintFormatter::new(source, &mut messages);
                    match linter.run(&package_ctx, &mut formatter)? {
                        RunStatus::Executed => {
                            // Lint ran successfully.
                        }
                        RunStatus::Skipped(reason) => {
                            skipped.push((source, reason));
                        }
                    }

                    if self.config.fail_fast && !messages.is_empty() {
                        // At least one issue was found.
                        return Ok(LintResults { skipped, messages });
                    }
                }
            }
        }

        // Run file path linters.
        if !self.config.file_path_linters.is_empty() {
            let file_list = self.file_list()?;

            let file_ctxs = file_list.map(|path| FilePathContext::new(&self.project_ctx, path));

            for file_ctx in file_ctxs {
                for linter in self.config.file_path_linters {
                    let source = file_ctx.source(linter.name());
                    let mut formatter = LintFormatter::new(source, &mut messages);
                    match linter.run(&file_ctx, &mut formatter)? {
                        RunStatus::Executed => {
                            // Lint ran successfully.
                        }
                        RunStatus::Skipped(reason) => {
                            skipped.push((source, reason));
                        }
                    }

                    if self.config.fail_fast && !messages.is_empty() {
                        // At least one issue was found.
                        return Ok(LintResults { skipped, messages });
                    }
                }
            }
        }

        // Run content linters.
        if !self.config.content_linters.is_empty() {
            let file_list = self.file_list()?;

            // TODO: This should probably be a worker queue with a thread pool or something.

            let file_ctxs = file_list.map(|path| FilePathContext::new(&self.project_ctx, path));

            for file_ctx in file_ctxs {
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
                let content_ctx = match file_ctx.load()? {
                    Some(content_ctx) => content_ctx,
                    None => {
                        // This file is missing -- can't run content linters on it.
                        continue;
                    }
                };

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
                        return Ok(LintResults { skipped, messages });
                    }
                }
            }
        }

        Ok(LintResults { skipped, messages })
    }

    // ---
    // Helper methods
    // ---

    fn file_list(&self) -> Result<impl Iterator<Item = &'cfg Utf8Path> + 'cfg> {
        let git_cli = self.config.core.git_cli()?;
        let tracked_files = git_cli.tracked_files()?;
        // TODO: make global exclusions configurable
        Ok(tracked_files
            .iter()
            .filter(|f| !f.starts_with("testsuite/diem-fuzzer/artifacts/")))
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub struct LintResults<'l> {
    pub skipped: Vec<(LintSource<'l>, SkipReason<'l>)>,
    pub messages: Vec<(LintSource<'l>, LintMessage)>,
}
