// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{shared::fake_natives::transaction as TXN, shared::Address};
use regex::{NoExpand, Regex};
use std::fs;
use std::path::Path;

fn txn(n: &str) -> String {
    format!("{}::{}::{}(", Address::LIBRA_CORE, TXN::MOD, n)
}

macro_rules! replace {
    ($input:ident, $pat:expr, $replacer:expr) => {{
        let regex = Regex::new($pat).unwrap();
        regex.replace_all(&$input, $replacer)
    }};
}

#[allow(clippy::trivial_regex)]
pub fn fix_syntax_and_write(out_path: &Path, contents: String) {
    let contents = replace!(contents, r"get_txn_sender\(", NoExpand(&txn(TXN::SENDER)));
    let contents = replace!(contents, r"assert\(", NoExpand(&txn(TXN::ASSERT)));
    let contents = replace!(contents, r"move\((\w+)\)", "move $1");
    let contents = replace!(contents, r"copy\((\w+)\)", "copy $1");
    let contents = replace!(contents, r"resource\s+(\w)", "resource struct $1");
    let contents = replace!(contents, r"import", NoExpand("use"));
    let contents = replace!(contents, r"Self\.", NoExpand(""));
    let contents = replace!(contents, r"(([A-Z]\w*)|(\}\})|(0x\d+))\.", "$1::");
    fs::write(out_path, contents.as_bytes()).unwrap();
}
