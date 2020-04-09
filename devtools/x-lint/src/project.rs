// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{LintContext, LintKind};
use std::path::{Path, PathBuf};

/// Overall linter context for a project.
#[derive(Copy, Clone, Debug)]
pub struct ProjectContext<'l> {
    project_root: &'l Path,
}

#[allow(dead_code)]
impl<'l> ProjectContext<'l> {
    pub fn new(project_root: &'l Path) -> Self {
        Self { project_root }
    }

    /// Returns the project root.
    pub fn project_root(&self) -> &'l Path {
        self.project_root
    }

    /// Returns the absolute path from the project root.
    pub fn full_path(&self, path: impl AsRef<Path>) -> PathBuf {
        self.project_root.join(path.as_ref())
    }
}

impl<'l> LintContext<'l> for ProjectContext<'l> {
    fn kind(&self) -> LintKind<'l> {
        LintKind::Project
    }
}
