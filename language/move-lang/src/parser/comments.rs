// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{diag, diagnostics::Diagnostics};
use move_command_line_common::character_sets::is_permitted_char;
use move_ir_types::location::*;
use move_symbol_pool::Symbol;
use std::collections::BTreeMap;

/// Types to represent comments.
pub type CommentMap = BTreeMap<Symbol, MatchedFileCommentMap>;
pub type MatchedFileCommentMap = BTreeMap<u32, String>;
pub type FileCommentMap = BTreeMap<(u32, u32), String>;

// We restrict strings to only ascii visual characters (0x20 <= c <= 0x7E) or a permitted newline
// character--\n--or a tab--\t.
pub fn verify_string(fname: Symbol, string: &str) -> Result<(), Diagnostics> {
    match string
        .chars()
        .enumerate()
        .find(|(_, c)| !is_permitted_char(*c))
    {
        None => Ok(()),
        Some((idx, chr)) => {
            let loc = Loc::new(fname, idx as u32, idx as u32);
            let msg = format!(
                "Invalid character '{}' found when reading file. Only ASCII printable characters, \
                 tabs (\\t), and line endings (\\n) are permitted.",
                chr
            );
            Err(Diagnostics::from(vec![diag!(
                Syntax::InvalidCharacter,
                (loc, msg)
            )]))
        }
    }
}
