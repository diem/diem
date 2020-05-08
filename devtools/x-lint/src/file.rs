// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{prelude::*, LintContext};
use std::{ffi::OsStr, fs, io, path::Path};

/// Contains information for a single file.
#[derive(Clone, Debug)]
pub struct FileContext<'l> {
    project_ctx: ProjectContext<'l>,
    file_path: &'l Path,
}

impl<'l> FileContext<'l> {
    pub fn new(project_ctx: ProjectContext<'l>, file_path: &'l Path) -> Self {
        Self {
            project_ctx,
            file_path,
        }
    }

    /// Returns the project context.
    pub fn project_ctx(&self) -> ProjectContext<'l> {
        self.project_ctx
    }

    /// Returns the path of this file, relative to the root of the repository.
    pub fn file_path(&self) -> &'l Path {
        &self.file_path
    }

    /// Returns the extension of the file. Returns `None` if there's no extension or if the
    /// extension isn't valid UTF-8.
    pub fn extension(&self) -> Option<&'l str> {
        self.file_path.extension().map(OsStr::to_str).flatten()
    }

    /// Loads this file and turns it into a `ContentContext`.
    ///
    /// Returns `None` if the file is missing.
    ///
    /// `pub(super)` is to dissuade individual linters from loading file contexts.
    pub(super) fn load(self) -> Result<Option<ContentContext<'l>>> {
        let full_path = self.project_ctx.full_path(self.file_path);
        let contents_opt = read_file(&full_path)
            .map_err(|err| SystemError::io(format!("loading {}", full_path.display()), err))?;
        Ok(contents_opt.map(|content| ContentContext::new(self, content)))
    }
}

impl<'l> LintContext<'l> for FileContext<'l> {
    fn kind(&self) -> LintKind<'l> {
        LintKind::File(self.file_path)
    }
}

fn read_file(full_path: &Path) -> io::Result<Option<Vec<u8>>> {
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
