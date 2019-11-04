// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::{borrow::Cow, path::Path};

static LICENSE_HEADER: &str = "Copyright (c) The Libra Core Contributors\n\
                               SPDX-License-Identifier: Apache-2.0\n\
                               ";

pub(super) fn has_license_header(file: &Path, contents: &str) -> Result<(), Cow<'static, str>> {
    enum FileType {
        Rust,
        Shell,
        Proto,
    }

    let file_type = if let Some(extension) = file.extension() {
        if extension == "rs" {
            FileType::Rust
        } else if extension == "sh" {
            FileType::Shell
        } else if extension == "proto" {
            FileType::Proto
        } else {
            return Ok(());
        }
    } else {
        return Ok(());
    };

    // Determine if the file is missing the license header
    let missing_header = match file_type {
        FileType::Rust | FileType::Proto => {
            let maybe_license = contents
                .lines()
                .skip_while(|line| line.is_empty())
                .take(2)
                .map(|s| s.trim_start_matches("// "));
            !LICENSE_HEADER.lines().eq(maybe_license)
        }
        FileType::Shell => {
            let maybe_license = contents
                .lines()
                .skip_while(|line| line.starts_with("#!"))
                .skip_while(|line| line.is_empty())
                .take(2)
                .map(|s| s.trim_start_matches("# "));
            !LICENSE_HEADER.lines().eq(maybe_license)
        }
    };

    if missing_header {
        return Err("missing a license header".into());
    }

    Ok(())
}
