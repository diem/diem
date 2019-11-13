// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::{borrow::Cow, convert, ffi::OsStr, path::Path, str};

pub(super) fn has_newline_at_eof(file: &Path, contents: &str) -> Result<(), Cow<'static, str>> {
    if skip_whitespace_checks(file) {
        return Ok(());
    }

    if !contents.ends_with('\n') {
        Err("missing a newline at EOF".into())
    } else {
        Ok(())
    }
}

pub(super) fn has_trailing_whitespace(
    file: &Path,
    contents: &str,
) -> Result<(), Cow<'static, str>> {
    if skip_whitespace_checks(file) {
        return Ok(());
    }

    for (ln, line) in contents
        .lines()
        .enumerate()
        .map(|(ln, line)| (ln + 1, line))
    {
        if line.trim_end() != line {
            return Err(Cow::Owned(format!("trailing whitespace on line {}", ln)));
        }
    }

    if contents
        .lines()
        .rev()
        .take_while(|line| line.is_empty())
        .count()
        > 0
    {
        return Err("trailing whitespace at EOF".into());
    }

    Ok(())
}

fn skip_whitespace_checks(file: &Path) -> bool {
    match file
        .extension()
        .map(OsStr::to_str)
        .and_then(convert::identity)
    {
        Some("exp") => true,
        _ => false,
    }
}
