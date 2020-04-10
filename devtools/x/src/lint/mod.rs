// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::context::XContext;
use anyhow::anyhow;
use structopt::StructOpt;
use x_lint::{prelude::*, LintEngineConfig};

mod guppy;
mod license;
mod whitespace;

#[derive(Debug, StructOpt)]
pub struct Args {
    #[structopt(long)]
    fail_fast: bool,
}

pub fn run(args: Args, xctx: XContext) -> crate::Result<()> {
    let workspace_config = xctx.config().workspace_config();

    let project_linters: &[&dyn ProjectLinter] = &[&guppy::BannedDirectDeps::new(
        &workspace_config.banned_direct_deps,
    )];

    let package_linters: &[&dyn PackageLinter] = &[&guppy::EnforcedAttributes::new(
        &workspace_config.enforced_attributes,
    )];

    let content_linters: &[&dyn ContentLinter] = &[
        &license::LicenseHeader,
        &whitespace::EofNewline,
        &whitespace::TrailingWhitespace,
    ];

    let engine = LintEngineConfig::new(xctx.core())
        .with_project_linters(project_linters)
        .with_package_linters(package_linters)
        .with_content_linters(content_linters)
        .fail_fast(args.fail_fast)
        .build();

    let results = engine.run()?;

    // TODO: handle skipped results

    for (source, message) in &results.messages {
        println!(
            "[{}] [{}] [{}]: {}",
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
