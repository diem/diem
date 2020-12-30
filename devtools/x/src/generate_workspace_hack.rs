// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{context::XContext, Result};
use anyhow::bail;
use log::{info, warn};
use structopt::{clap::arg_enum, StructOpt};

#[derive(Debug, StructOpt)]
pub struct Args {
    /// Mode to run in
    #[structopt(long, case_insensitive = true, possible_values = &WorkspaceHackMode::variants(), default_value = "write")]
    mode: WorkspaceHackMode,
}

arg_enum! {
    #[derive(Clone, Copy, Debug)]
    /// Valid modes for generate-workspace-hack
    pub enum WorkspaceHackMode {
        Write,
        Diff,
        Check,
        Verify,
    }
}

pub fn run(args: Args, xctx: XContext) -> Result<()> {
    let mut hakari_builder = xctx.core().hakari_builder()?;

    match args.mode {
        WorkspaceHackMode::Verify => {
            hakari_builder.set_verify_mode(true);
            let hakari = hakari_builder.compute();
            if hakari.output_map.is_empty() {
                info!("workspace-hack is valid");
            } else {
                for ((platform_idx, package_id), v) in &hakari.computed_map {
                    for &(build_platform, inner_map) in v.inner_maps().iter() {
                        if inner_map.len() > 1 {
                            println!(
                                "platform idx {:?} on {:?}, package ID: {}",
                                platform_idx, build_platform, package_id
                            );
                            for (feature_set, packages) in inner_map {
                                let features: Vec<_> = feature_set.iter().copied().collect();
                                println!("  for features {}:", features.join(", "));
                                for (package, f) in packages {
                                    println!("    * {} ({:?})", package.name(), f);
                                }
                            }
                        }
                    }
                }
                warn!("workspace-hack doesn't unify everything successfully");
            }
        }
        _other => {
            let hakari = hakari_builder.compute();
            let existing_toml = hakari
                .read_toml()
                .expect("hakari package specified by builder")?;
            let new_toml = hakari.to_toml_string(&xctx.core().hakari_toml_options())?;

            match args.mode {
                WorkspaceHackMode::Write => {
                    // Write out the contents to the TOML file.
                    existing_toml.write_to_file(&new_toml)?;
                }
                WorkspaceHackMode::Diff => {
                    let patch = existing_toml.diff_toml(&new_toml);
                    // TODO: add global coloring options to x
                    let formatter = hakari::diffy::PatchFormatter::new().with_color();
                    let diff = formatter.fmt_patch(&patch);
                    println!("{}", diff);
                }
                WorkspaceHackMode::Check => {
                    if existing_toml.is_changed(&new_toml) {
                        bail!("existing TOML is different from generated version (run with --mode diff for diff)");
                    }
                }
                WorkspaceHackMode::Verify => unreachable!("already processed in outer match"),
            }
        }
    };

    Ok(())
}
