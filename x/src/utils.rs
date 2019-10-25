use crate::Result;
use anyhow::anyhow;
use serde::Deserialize;
use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

pub fn project_root() -> PathBuf {
    Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
        .to_path_buf()
}

pub fn locate_project() -> Result<PathBuf> {
    #[derive(Deserialize)]
    struct LocateProject {
        root: PathBuf,
    };

    let output = Command::new("cargo")
        .arg("locate-project")
        .stderr(Stdio::inherit())
        .output()?;
    if !output.status.success() {
        return Err(anyhow!("unable to locate project"));
    }

    Ok(serde_json::from_slice::<LocateProject>(&output.stdout)?.root)
}

pub fn project_is_root() -> Result<bool> {
    let mut project = locate_project()?;
    project.pop();

    Ok(project == project_root())
}

pub fn get_local_package() -> Result<String> {
    let output = Command::new("cargo")
        .arg("pkgid")
        .stderr(Stdio::inherit())
        .output()?;
    if !output.status.success() {
        return Err(anyhow!("unable to get package id"));
    }

    let output = std::str::from_utf8(&output.stdout)?;
    let pkgid = Path::new(&output).file_name().unwrap().to_str().unwrap();

    let name = if let Some(idx) = pkgid.find(':') {
        let (pkgid, _) = pkgid.split_at(idx);
        pkgid.split_at(pkgid.find('#').unwrap()).1[1..].to_string()
    } else {
        pkgid.split_at(pkgid.find('#').unwrap()).0.to_string()
    };

    Ok(name)
}
