// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{cargo::Cargo, context::XContext, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// The number of directories between the project root and the root of this crate.
pub const X_DEPTH: usize = 2;

/// Returns the project root. TODO: switch uses to XCoreContext::project_root instead)
pub fn project_root() -> &'static Path {
    Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(X_DEPTH)
        .unwrap()
}

pub fn locate_project(xctx: &XContext) -> Result<PathBuf> {
    #[derive(Deserialize)]
    struct LocateProject {
        root: PathBuf,
    };

    let output = Cargo::new(xctx.config().cargo_config(), "locate-project").run_with_output()?;
    Ok(serde_json::from_slice::<LocateProject>(&output)?.root)
}

pub fn project_is_root(xctx: &XContext) -> Result<bool> {
    let mut project = locate_project(xctx)?;
    project.pop();

    Ok(project == project_root())
}
