// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{config::Config, utils::project_root};
use anyhow::anyhow;
use std::{
    borrow::Cow,
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    str,
};
use structopt::StructOpt;

mod license;
mod whitespace;

#[derive(Debug, StructOpt)]
pub struct Args {
    #[structopt(long)]
    fail_fast: bool,
}

/// List of lints to run
static LINTS: &[fn(&Path, &str) -> Result<(), Cow<'static, str>>] = &[
    license::has_license_header,
    whitespace::has_newline_at_eof,
    whitespace::has_trailing_whitespace,
];

pub fn run(args: Args, _config: Config) -> crate::Result<()> {
    let mut errors = false;
    let project_root = project_root();
    let mut full_file_path = PathBuf::new();

    for file in get_file_listing()? {
        full_file_path.push(&project_root);
        full_file_path.push(&file);

        let contents = match fs::read_to_string(&full_file_path) {
            Ok(s) => s,
            // Skip non utf8 files
            Err(_) => continue,
        };

        // Run each lint on the file
        for lint in LINTS {
            if let Err(e) = lint(&file, &contents) {
                errors = true;
                let msg = format!("{}: {}", file.display(), e);
                if args.fail_fast {
                    return Err(anyhow!(msg));
                } else {
                    println!("{}", msg);
                }
            }
        }
    }

    if errors {
        Err(anyhow!("there were lint errors"))
    } else {
        Ok(())
    }
}

fn get_file_listing() -> crate::Result<Vec<PathBuf>> {
    let project_root = project_root();
    let output = Command::new("git")
        .current_dir(&project_root)
        .arg("ls-files")
        .stderr(Stdio::inherit())
        .output()?;
    if !output.status.success() {
        return Err(anyhow!("failed to run git command"));
    }

    Ok(str::from_utf8(&output.stdout)?
        .split('\n')
        .filter(|s| !s.is_empty())
        .filter(|s| !s.starts_with("testsuite/libra-fuzzer/artifacts/"))
        .map(Path::new)
        .map(Path::to_path_buf)
        .collect())
}
