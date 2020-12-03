// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{prelude::*, LintContext};
use std::str;

/// Represents a linter that checks file contents.
pub trait ContentLinter: Linter {
    /// Pre-run step -- avoids loading the contents if possible.
    ///
    /// The default implementation returns `Ok(RunStatus::Executed)`; individual lints may configure
    /// a more restricted set.
    fn pre_run<'l>(&self, _file_ctx: &FilePathContext<'l>) -> Result<RunStatus<'l>> {
        Ok(RunStatus::Executed)
    }

    /// Executes the lint against the given content context.
    fn run<'l>(
        &self,
        ctx: &ContentContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>>;
}

#[derive(Debug)]
pub struct ContentContext<'l> {
    file_ctx: FilePathContext<'l>,
    content: Content,
}

#[allow(dead_code)]
impl<'l> ContentContext<'l> {
    /// The number of bytes that will be searched for null bytes in a file to figure out if it is
    /// binary.
    ///
    /// The value is [the same as Git's](https://stackoverflow.com/a/6134127).
    pub const BINARY_FILE_CUTOFF: usize = 8000;

    pub(super) fn new(file_ctx: FilePathContext<'l>, content: Vec<u8>) -> Self {
        Self {
            file_ctx,
            content: Content::new(content),
        }
    }

    /// Returns the file context.
    pub fn file_ctx(&self) -> &FilePathContext<'l> {
        &self.file_ctx
    }

    /// Returns the content, or `None` if this is a non-UTF-8 file.
    pub fn content(&self) -> Option<&str> {
        match &self.content {
            Content::Utf8(text) => Some(text.as_ref()),
            Content::NonUtf8(_) => None,
        }
    }

    /// Returns the raw bytes for the content.
    pub fn content_bytes(&self) -> &[u8] {
        match &self.content {
            Content::Utf8(text) => text.as_bytes(),
            Content::NonUtf8(bin) => bin.as_ref(),
        }
    }

    /// Returns true if this is a binary file.
    pub fn is_binary(&self) -> bool {
        match &self.content {
            Content::Utf8(_) => {
                // UTF-8 files are not binary by definition.
                false
            }
            Content::NonUtf8(bin) => bin[..Self::BINARY_FILE_CUTOFF].contains(&0),
        }
    }
}

impl<'l> LintContext<'l> for ContentContext<'l> {
    fn kind(&self) -> LintKind<'l> {
        LintKind::Content(self.file_ctx.file_path())
    }
}

#[derive(Debug)]
enum Content {
    Utf8(Box<str>),
    NonUtf8(Box<[u8]>),
}

impl Content {
    fn new(bytes: Vec<u8>) -> Self {
        match String::from_utf8(bytes) {
            Ok(s) => Content::Utf8(s.into()),
            Err(err) => Content::NonUtf8(err.into_bytes().into()),
        }
    }

    #[allow(dead_code)]
    fn len(&self) -> usize {
        match self {
            Content::Utf8(text) => text.len(),
            Content::NonUtf8(bin) => bin.len(),
        }
    }
}
