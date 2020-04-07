// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{cargo::Cargo, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

pub fn project_root() -> &'static Path {
    Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
}

pub fn locate_project() -> Result<PathBuf> {
    #[derive(Deserialize)]
    struct LocateProject {
        root: PathBuf,
    };

    let output = Cargo::new("locate-project").run_with_output()?;
    Ok(serde_json::from_slice::<LocateProject>(&output)?.root)
}

pub fn project_is_root() -> Result<bool> {
    let mut project = locate_project()?;
    project.pop();

    Ok(project == project_root())
}

pub fn get_local_package() -> Result<String> {
    let output = Cargo::new("pkgid").run_with_output()?;
    let pkgid = Path::new(std::str::from_utf8(&output)?)
        .file_name()
        .unwrap()
        .to_str()
        .unwrap();

    let name = if let Some(idx) = pkgid.find(':') {
        let (pkgid, _) = pkgid.split_at(idx);
        pkgid.split_at(pkgid.find('#').unwrap()).1[1..].to_string()
    } else {
        pkgid.split_at(pkgid.find('#').unwrap()).0.to_string()
    };

    Ok(name)
}
