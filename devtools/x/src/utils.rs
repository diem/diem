// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{config::CargoConfig, installer::install_cargo_component_if_needed, Result};
use anyhow::anyhow;
use camino::Utf8Path;
use log::{info, warn};
use std::{
    env::var_os,
    process::{Command, Stdio},
};

/// The number of directories between the project root and the root of this crate.
pub const X_DEPTH: usize = 2;

/// Returns the project root. TODO: switch uses to XCoreContext::project_root instead)
pub fn project_root() -> &'static Utf8Path {
    Utf8Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(X_DEPTH)
        .unwrap()
}

/// If the project is configured for sccache, and the env variable SKIP_SCCACHE is unset then returns true.
/// If the warn_if_not_correct_location parameter is set to true, warnings will be logged if the project is configured for sccache
/// but the CARGO_HOME or project root are not in the right locations.
pub fn sccache_should_run(cargo_config: &CargoConfig, warn_if_not_correct_location: bool) -> bool {
    if var_os("SKIP_SCCACHE").is_none() {
        if let Some(sccache_config) = &cargo_config.sccache {
            // Are we work on items in the right location:
            // See: https://github.com/mozilla/sccache#known-caveats
            let correct_location = var_os("CARGO_HOME").unwrap_or_default()
                == sccache_config.required_cargo_home.as_str()
                && sccache_config.required_git_home == project_root();
            if !correct_location && warn_if_not_correct_location {
                warn!("You will not benefit from sccache in this build!!!");
                warn!(
                    "To get the best experience, please move your diem source code to {} and your set your CARGO_HOME to be {}, simply export it in your .profile or .bash_rc",
                    &sccache_config.required_git_home, &sccache_config.required_cargo_home
                );
                warn!(
                    "Current diem root is '{}',  and current CARGO_HOME is '{}'",
                    project_root(),
                    var_os("CARGO_HOME").unwrap_or_default().to_string_lossy()
                );
            }
            correct_location
        } else {
            false
        }
    } else {
        false
    }
}

/// Logs the output of "sccache --show-stats"
pub fn log_sccache_stats() {
    info!("Sccache statistics:");
    let mut sccache = Command::new("sccache");
    sccache.arg("--show-stats");
    sccache.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    if let Err(error) = sccache.output() {
        warn!("Could not log sccache statistics: {}", error);
    }
}

pub fn stop_sccache_server() {
    let mut sccache = Command::new("sccache");
    sccache.arg("--stop-server");
    sccache.stdout(Stdio::piped()).stderr(Stdio::piped());
    match sccache.output() {
        Ok(output) => {
            if output.status.success() {
                info!("Stopped already running sccache.");
            } else {
                let std_err = String::from_utf8_lossy(&output.stderr);
                //sccache will fail
                if !std_err.contains("couldn't connect to server") {
                    warn!("Failed to stopped already running sccache.");
                    warn!("status: {}", output.status);
                    warn!("stdout: {}", String::from_utf8_lossy(&output.stdout));
                    warn!("stderr: {}", std_err);
                }
            }
        }
        Err(error) => {
            warn!("Failed to stop running sccache: {}", error)
        }
    }
}

pub fn apply_sccache_if_possible(
    cargo_config: &CargoConfig,
) -> Result<Vec<(&str, Option<String>)>> {
    let mut envs = vec![];

    if sccache_should_run(cargo_config, true) {
        if let Some(sccache_config) = &cargo_config.sccache {
            if !install_cargo_component_if_needed(
                cargo_config,
                "sccache",
                &sccache_config.installer,
            ) {
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
            let access_key_id =
                var_os("SCCACHE_AWS_ACCESS_KEY_ID").map(|val| val.to_string_lossy().to_string());
            let access_key_secret = var_os("SCCACHE_AWS_SECRET_ACCESS_KEY")
                .map(|val| val.to_string_lossy().to_string());
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
    Ok(envs)
}
