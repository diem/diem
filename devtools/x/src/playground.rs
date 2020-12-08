// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Playground for arbitrary code.
//!
//! This lets users experiment with new lints and other throwaway code.
//! Add your code in the spots marked `// --- ADD PLAYGROUND CODE HERE ---`.
//!
//! This file should not have any production-related code checked into it.

#![allow(unused_variables)]

use crate::context::XContext;
use anyhow::anyhow;
use structopt::StructOpt;
use x_lint::prelude::*;

#[derive(Copy, Clone, Debug)]
struct PlaygroundProject;

impl Linter for PlaygroundProject {
    fn name(&self) -> &'static str {
        "playground-project"
    }
}

impl ProjectLinter for PlaygroundProject {
    fn run<'l>(
        &self,
        ctx: &ProjectContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>> {
        // --- ADD PLAYGROUND CODE HERE ---

        Ok(RunStatus::Executed)
    }
}

#[derive(Copy, Clone, Debug)]
struct PlaygroundPackage;

impl Linter for PlaygroundPackage {
    fn name(&self) -> &'static str {
        "playground-package"
    }
}

impl PackageLinter for PlaygroundPackage {
    fn run<'l>(
        &self,
        ctx: &PackageContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>> {
        // --- ADD PLAYGROUND CODE HERE ---

        Ok(RunStatus::Executed)
    }
}

#[derive(Copy, Clone, Debug)]
struct PlaygroundFilePath;

impl Linter for PlaygroundFilePath {
    fn name(&self) -> &'static str {
        "playground-file-path"
    }
}

impl FilePathLinter for PlaygroundFilePath {
    fn run<'l>(
        &self,
        ctx: &FilePathContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>> {
        // --- ADD PLAYGROUND CODE HERE ---

        Ok(RunStatus::Executed)
    }
}

#[derive(Copy, Clone, Debug)]
struct PlaygroundContent;

impl Linter for PlaygroundContent {
    fn name(&self) -> &'static str {
        "playground-content"
    }
}

impl ContentLinter for PlaygroundContent {
    fn run<'l>(
        &self,
        ctx: &ContentContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>> {
        // --- ADD PLAYGROUND CODE HERE ---

        Ok(RunStatus::Executed)
    }
}

// ---

#[derive(Debug, StructOpt)]
pub struct Args {
    /// Dummy arg that doesn't do anything
    #[structopt(long)]
    dummy: bool,
}

pub fn run(args: Args, xctx: XContext) -> crate::Result<()> {
    let engine = LintEngineConfig::new(xctx.core())
        .with_project_linters(&[&PlaygroundProject])
        .with_package_linters(&[&PlaygroundPackage])
        .with_file_path_linters(&[&PlaygroundFilePath])
        .with_content_linters(&[&PlaygroundContent])
        .build();

    let results = engine.run()?;

    for (source, message) in &results.messages {
        println!(
            "[{}] [{}] [{}]: {}\n",
            message.level(),
            source.name(),
            source.kind(),
            message.message()
        );
    }

    if !results.messages.is_empty() {
        Err(anyhow!("there were lint errors"))
    } else {
        Ok(())
    }
}
