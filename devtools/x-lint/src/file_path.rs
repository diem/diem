// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{prelude::*, LintContext};
use camino::Utf8Path;
use std::{fs, io};

/// Represents a linter that runs once per file path.
pub trait FilePathLinter: Linter {
    /// Executes this linter against the given file path context.
    fn run<'l>(
        &self,
        ctx: &FilePathContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>>;
}

/// Contains information for a single file path.
#[derive(Clone, Debug)]
pub struct FilePathContext<'l> {
    project_ctx: &'l ProjectContext<'l>,
    file_path: &'l Utf8Path,
}

impl<'l> FilePathContext<'l> {
    /// Constructs a new context.
    pub fn new(project_ctx: &'l ProjectContext<'l>, file_path: &'l Utf8Path) -> Self {
        Self {
            project_ctx,
            file_path,
        }
    }

    /// Returns the project context.
    pub fn project_ctx(&self) -> &'l ProjectContext<'l> {
        self.project_ctx
    }

    /// Returns the path of this file, relative to the root of the repository.
    pub fn file_path(&self) -> &'l Utf8Path {
        self.file_path
    }

    /// Returns the extension of the file. Returns `None` if there's no extension.
    pub fn extension(&self) -> Option<&'l str> {
        self.file_path.extension()
    }

    /// Loads this file and turns it into a `ContentContext`.
    ///
    /// Returns `None` if the file is missing.
    ///
    /// `pub(super)` is to dissuade individual linters from loading file contexts.
    pub(super) fn load(self) -> Result<Option<ContentContext<'l>>> {
        let full_path = self.project_ctx.full_path(self.file_path);
        let contents_opt = read_file(&full_path)
            .map_err(|err| SystemError::io(format!("loading {}", full_path), err))?;
        Ok(contents_opt.map(|content| ContentContext::new(self, content)))
    }
}

impl<'l> LintContext<'l> for FilePathContext<'l> {
    fn kind(&self) -> LintKind<'l> {
        LintKind::FilePath(self.file_path)
    }
}

fn read_file(full_path: &Utf8Path) -> io::Result<Option<Vec<u8>>> {
    match fs::read(full_path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(err) => {
            if err.kind() == io::ErrorKind::NotFound {
                // Files can be listed by source control but missing -- this is normal.
                Ok(None)
            } else {
                Err(err)
            }
        }
    }
}
