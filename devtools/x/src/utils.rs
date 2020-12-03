// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{config::CargoConfig, installer::install_if_needed, Result};
use anyhow::anyhow;
use log::{info, warn};
use std::{env::var_os, path::Path, process::Command};

/// The number of directories between the project root and the root of this crate.
pub const X_DEPTH: usize = 2;

/// Returns the project root. TODO: switch uses to XCoreContext::project_root instead)
pub fn project_root() -> &'static Path {
    Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(X_DEPTH)
        .unwrap()
}

fn stop_sccache_server() {
    let mut sccache = Command::new("sccache");
    sccache.arg("--stop-server");
    let result = sccache.output();
    if let Ok(output) = result {
        if output.status.success() {
            info!("Stopped already running sccache.");
        }
    }
}

pub fn apply_sccache_if_possible<'a>(
    cargo_config: &'a CargoConfig,
) -> Result<Vec<(&'a str, Option<String>)>> {
    let mut envs = vec![];

    if var_os("SKIP_SCCACHE").is_none() && cargo_config.sccache.is_some() {
        if let Some(sccache_config) = &cargo_config.sccache {
            // Are we work on items in the right location:
            // See: https://github.com/mozilla/sccache#known-caveats
            let correct_location = var_os("CARGO_HOME")
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default()
                == sccache_config.required_cargo_home
                && sccache_config.required_git_home == project_root().to_str().unwrap_or_default();
            if !correct_location {
                warn!("You will not benefit from sccache in this build!!!");
                warn!(
                    "To get the best experience, please move your diem source code to {} and your set your CARGO_HOME to be {}, simply export it in your .profile or .bash_rc",
                    &sccache_config.required_git_home, &sccache_config.required_cargo_home
                );
                warn!(
                    "Current diem root is '{}',  and current CARGO_HOME is '{}'",
                    project_root().to_str().unwrap_or_default(),
                    var_os("CARGO_HOME").unwrap_or_default().to_string_lossy()
                );
            } else {
                if !install_if_needed(cargo_config, "sccache", &sccache_config.installer) {
                    return Err(anyhow!("Failed to install sccache, bailing"));
                }
                stop_sccache_server();
                envs.push(("RUSTC_WRAPPER", Some("sccache".to_owned())));
                envs.push(("CARGO_INCREMENTAL", Some("false".to_owned())));
                envs.push(("SCCACHE_BUCKET", Some(sccache_config.bucket.to_owned())));
                if let Some(ssl) = &sccache_config.ssl {
                    envs.push((
                        "SCCACHE_S3_USE_SSL",
                        if *ssl {
                            Some("true".to_owned())
                        } else {
                            Some("false".to_owned())
                        },
                    ));
                }

                if let Some(url) = &sccache_config.endpoint {
                    envs.push(("SCCACHE_ENDPOINT", Some(url.to_owned())));
                }

                if let Some(extra_envs) = &sccache_config.envs {
                    for (key, value) in extra_envs {
                        envs.push((key, Some(value.to_owned())));
                    }
                }

                if let Some(region) = &sccache_config.region {
                    envs.push(("SCCACHE_REGION", Some(region.to_owned())));
                }

                if let Some(prefix) = &sccache_config.prefix {
                    envs.push(("SCCACHE_S3_KEY_PREFIX", Some(prefix.to_owned())));
                }
                let access_key_id = if let Some(val) = var_os("SCCACHE_AWS_ACCESS_KEY_ID") {
                    Some(val.to_string_lossy().to_string())
                } else {
                    None
                };
                let access_key_secret = if let Some(val) = var_os("SCCACHE_AWS_SECRET_ACCESS_KEY") {
                    Some(val.to_string_lossy().to_string())
                } else {
                    None
                };
                // if either the access or secret key is not set, attempt to perform a public read.
                // do not set this flag if attempting to write, as it will prevent the use of the aws creds.
                if (access_key_id.is_none() || access_key_secret.is_none())
                    && sccache_config.public.unwrap_or(true)
                {
                    envs.push(("SCCACHE_S3_PUBLIC", Some("true".to_owned())));
                }

                //Note: that this is also used to _unset_ AWS_ACCESS_KEY_ID & AWS_SECRET_ACCESS_KEY
                envs.push(("AWS_ACCESS_KEY_ID", access_key_id));
                envs.push(("AWS_SECRET_ACCESS_KEY", access_key_secret));
            }
        }
    }
    Ok(envs)
}
