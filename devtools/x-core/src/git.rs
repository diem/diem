// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::errors::*;
use determinator::Paths0;
use indoc::formatdoc;
use once_cell::sync::OnceCell;
use std::{
    path::Path,
    process::{Command, Stdio},
};

/// Support for source control operations through running Git commands.
///
/// This assumes that the underlying Git repository doesn't change in the middle of an operation,
/// and caches data as a result. If mutation operations are added, the caches would need to be
/// invalidated.
#[derive(Clone, Debug)]
pub struct GitCli {
    root: &'static Path,
    // Caches.
    tracked_files: OnceCell<Paths0>,
}

impl GitCli {
    /// Creates a new instance of the Git CLI.
    pub fn new(root: &'static Path) -> Result<Self> {
        let git_cli = Self {
            root,
            tracked_files: OnceCell::new(),
        };
        git_cli.validate()?;
        Ok(git_cli)
    }

    /// Returns the files tracked by Git in this working copy.
    ///
    /// The return value can be iterated on to get a list of paths.
    pub fn tracked_files(&self) -> Result<&Paths0> {
        self.tracked_files.get_or_try_init(|| {
            // TODO: abstract out SCM and command-running functionality.
            let output = self
                .git_command()
                // The -z causes files to not be quoted, and to be separated by \0.
                .args(&["ls-files", "-z"])
                .stderr(Stdio::inherit())
                .output()
                .map_err(|err| SystemError::io("running git ls-files", err))?;
            if !output.status.success() {
                return Err(SystemError::Exec {
                    cmd: "git ls-files",
                    status: output.status,
                });
            }

            // TODO: Get this working on Windows.
            Ok(Paths0::new_unix(output.stdout))
        })
    }

    // ---
    // Helper methods
    // ---

    fn validate(&self) -> Result<()> {
        // Check that the project root and the Git root match.
        let output = self
            .git_command()
            .args(&["rev-parse", "--show-toplevel"])
            .stderr(Stdio::inherit())
            .output()
            .map_err(|err| SystemError::io("running git rev-parse --show-toplevel", err))?;
        if !output.status.success() {
            let msg = formatdoc!(
                "unable to find a git repo at {}
                (hint: did you download an archive from GitHub? x requires a git clone)",
                self.root.display()
            );
            return Err(SystemError::git_root(msg));
        }

        let mut git_root_bytes = output.stdout;
        // Pop the newline off the git root bytes.
        git_root_bytes.pop();
        let git_root = match String::from_utf8(git_root_bytes) {
            Ok(git_root) => git_root,
            Err(_) => {
                return Err(SystemError::git_root(
                    "git rev-parse --show-toplevel returned a non-Unicode path",
                ));
            }
        };
        if self.root != Path::new(&git_root) {
            let msg = formatdoc!(
                "git root expected to be at {}, but actually found at {}
                (hint: did you download an archive from GitHub? x requires a git clone)",
                self.root.display(),
                git_root,
            );
            return Err(SystemError::git_root(msg));
        }
        Ok(())
    }

    fn git_command(&self) -> Command {
        // TODO: add support for the GIT environment variable?
        let mut command = Command::new("git");
        command.current_dir(self.root);
        command
    }
}
