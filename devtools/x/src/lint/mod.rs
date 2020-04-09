// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{config::Config, utils::project_root};
use anyhow::anyhow;
use structopt::StructOpt;
use x_lint::{prelude::*, LintEngineConfig};

mod license;
mod whitespace;

#[derive(Debug, StructOpt)]
pub struct Args {
    #[structopt(long)]
    fail_fast: bool,
}

pub fn run(args: Args, _config: Config) -> crate::Result<()> {
    let content_linters: &[&dyn ContentLinter] = &[
        &license::LicenseHeader,
        &whitespace::EofNewline,
        &whitespace::TrailingWhitespace,
    ];

    let engine = LintEngineConfig::new(project_root())
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
