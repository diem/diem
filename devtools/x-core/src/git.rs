// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::errors::*;
use camino::{Utf8Path, Utf8PathBuf};
use determinator::Utf8Paths0;
use guppy::{graph::PackageGraph, MetadataCommand};
use indoc::formatdoc;
use log::{debug, info};
use once_cell::sync::OnceCell;
use std::{
    borrow::Cow,
    ffi::{OsStr, OsString},
    fmt,
    process::{Command, Stdio},
};

/// Support for source control operations through running Git commands.
///
/// This assumes that the underlying Git repository doesn't change in the middle of an operation,
/// and caches data as a result. If mutation operations are added, the caches would need to be
/// invalidated.
#[derive(Clone, Debug)]
pub struct GitCli {
    root: &'static Utf8Path,
    // Caches.
    tracked_files: OnceCell<Utf8Paths0>,
}

impl GitCli {
    /// Creates a new instance of the Git CLI.
    pub fn new(root: &'static Utf8Path) -> Result<Self> {
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
    pub fn tracked_files(&self) -> Result<&Utf8Paths0> {
        self.tracked_files.get_or_try_init(|| {
            // TODO: abstract out SCM and command-running functionality.
            let output = self
                .git_command()
                // The -z causes files to not be quoted, and to be separated by \0.
                .args(&["ls-files", "-z"])
                .output()
                .map_err(|err| SystemError::io("running git ls-files", err))?;
            if !output.status.success() {
                return Err(SystemError::Exec {
                    cmd: "git ls-files",
                    status: output.status,
                });
            }

            Utf8Paths0::from_bytes(output.stdout)
                .map_err(|(path, err)| SystemError::NonUtf8Path { path, err })
        })
    }

    /// Returns the merge base of the current commit (`HEAD`) with the specified commit.
    pub fn merge_base(&self, commit_ref: &str) -> Result<GitHash> {
        let output = self
            .git_command()
            .args(&["merge-base", "HEAD", commit_ref])
            .output()
            .map_err(|err| {
                SystemError::io(format!("running git merge-base HEAD {}", commit_ref), err)
            })?;
        if !output.status.success() {
            return Err(SystemError::Exec {
                cmd: "git merge-base",
                status: output.status,
            });
        }

        // The output is a hex-encoded hash followed by a newline.
        let stdout = &output.stdout[..(output.stdout.len() - 1)];
        GitHash::from_hex(stdout)
    }

    /// Returns the files changed between the given commits, or the current directory if the new
    /// commit isn't specified.
    ///
    /// For more about the diff filter, see `man git-diff`'s help for `--diff-filter`.
    pub fn files_changed_between<'a>(
        &self,
        old: impl Into<Cow<'a, OsStr>>,
        new: impl Into<Option<Cow<'a, OsStr>>>,
        // TODO: make this more well-typed/express more of the diff model in Rust
        diff_filter: Option<&str>,
    ) -> Result<Utf8Paths0> {
        let mut command = self.git_command();
        command.args(&["diff", "-z", "--name-only"]);
        if let Some(diff_filter) = diff_filter {
            command.arg(format!("--diff-filter={}", diff_filter));
        }
        command.arg(old.into());
        if let Some(new) = new.into() {
            command.arg(new);
        }

        let output = command
            .output()
            .map_err(|err| SystemError::io("running git diff", err))?;
        if !output.status.success() {
            return Err(SystemError::Exec {
                cmd: "git diff",
                status: output.status,
            });
        }

        Utf8Paths0::from_bytes(output.stdout)
            .map_err(|(path, err)| SystemError::NonUtf8Path { path, err })
    }

    /// Returns a package graph for the given commit, using a scratch repo if necessary.
    pub fn package_graph_at(&self, commit_ref: &GitHash) -> Result<PackageGraph> {
        // Create or initialize the scratch worktree.
        let scratch = self.get_or_init_scratch(commit_ref)?;

        // Compute the package graph for the scratch worktree.
        MetadataCommand::new()
            .current_dir(scratch)
            .build_graph()
            .map_err(|err| SystemError::guppy("building package graph", err))
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
                self.root
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
        if self.root != git_root {
            let msg = formatdoc!(
                "git root expected to be at {}, but actually found at {}
                (hint: did you download an archive from GitHub? x requires a git clone)",
                self.root,
                git_root,
            );
            return Err(SystemError::git_root(msg));
        }
        Ok(())
    }

    // TODO: abstract out command running and error handling
    fn git_command(&self) -> Command {
        // TODO: add support for the GIT environment variable?
        let mut command = Command::new("git");
        command.current_dir(self.root).stderr(Stdio::inherit());
        command
    }

    /// Gets the scratch worktree if it exists, or initializes it if it doesn't.
    ///
    /// The scratch worktree is meant to be persistent across invocations of `x`. This is done for
    /// performance reasons.
    fn get_or_init_scratch(&self, hash: &GitHash) -> Result<Utf8PathBuf> {
        let mut scratch_dir = self.root.join("target");
        scratch_dir.extend(&["x-scratch", "tree"]);

        if scratch_dir.is_dir() && self.is_git_repo(&scratch_dir)? {
            debug!("Using existing scratch worktree at {}", scratch_dir,);

            // Check out the given hash in the scratch worktree.
            let output = self
                .git_command()
                .current_dir(&scratch_dir)
                // TODO: also git clean?
                .args(&["reset", &format!("{:x}", hash), "--hard"])
                .output()
                .map_err(|err| SystemError::io("running git checkout in scratch tree", err))?;
            if !output.status.success() {
                return Err(SystemError::Exec {
                    cmd: "git checkout",
                    status: output.status,
                });
            }
        } else {
            if scratch_dir.is_dir() {
                std::fs::remove_dir_all(&scratch_dir)
                    .map_err(|err| SystemError::io("cleaning old scratch_dir", err))?;
            }

            // Try creating a scratch worktree at that location.
            info!("Setting up scratch worktree in {}", scratch_dir);
            let output = self
                .git_command()
                .args(&["worktree", "add"])
                .arg(&scratch_dir)
                .args(&[&format!("{:x}", hash), "--detach"])
                .output()
                .map_err(|err| SystemError::io("running git worktree add", err))?;
            if !output.status.success() {
                return Err(SystemError::Exec {
                    cmd: "git worktree add",
                    status: output.status,
                });
            }
        }

        // TODO: some sort of cross-process locking may be necessary in the future. Don't worry
        // about it for now.
        Ok(scratch_dir)
    }

    pub fn is_git_repo(&self, dir: &Utf8Path) -> Result<bool> {
        let output = self
            .git_command()
            .current_dir(dir)
            .args(&["rev-parse", "--git-dir"])
            .output()
            .map_err(|err| SystemError::io("checking if a directory is a git repo", err))?;

        Ok(output.status.success())
    }
}

/// A Git hash.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct GitHash([u8; 20]);

impl GitHash {
    /// Creates a new Git hash from a hex-encoded string.
    pub fn from_hex(hex: impl AsRef<[u8]>) -> Result<Self> {
        let hex = hex.as_ref();
        Ok(GitHash(hex::FromHex::from_hex(hex).map_err(|err| {
            SystemError::from_hex(format!("parsing a Git hash: {:?}", hex), err)
        })?))
    }
}

impl<'a, 'b> From<&'a GitHash> for Cow<'b, OsStr> {
    fn from(git_hash: &'a GitHash) -> Cow<'b, OsStr> {
        OsString::from(format!("{:x}", git_hash)).into()
    }
}

impl fmt::LowerHex for GitHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(&self.0))
    }
}
