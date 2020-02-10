// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{shared::fake_natives::transaction as TXN, shared::Address};
use regex::{Captures, NoExpand, Regex};
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
    // get_txn_sender() ~> 0x0::Transaction::sender()
    let contents = replace!(contents, r"get_txn_sender\(", NoExpand(&txn(TXN::SENDER)));
    // assert(c, u) ~> 0x0::Transaction::assert(c, u)
    let contents = replace!(contents, r"assert\(", NoExpand(&txn(TXN::ASSERT)));
    // move(x) ~> move x
    let contents = replace!(contents, r"move\((\w+)\)", "move $1");
    // copy(x) ~> copy x
    let contents = replace!(contents, r"copy\((\w+)\)", "copy $1");
    // resource StructName ~> resource struct StructName
    let contents = replace!(contents, r"resource\s+(\w)", "resource struct $1");
    // unrestricted ~> copyable
    let contents = replace!(contents, r":\s*unrestricted", NoExpand(": copyable"));
    // import ~> use
    let contents = replace!(contents, r"import", NoExpand("use"));
    // Self. is unnecessary
    let contents = replace!(contents, r"Self\.", NoExpand(""));
    // Module|Address. ~> Module|Address::
    let contents = replace!(contents, r"(([A-Z]\w*)|(\}\})|(0x\d+))\.", "$1::");
    // add fun keyword to functions
    let contents = replace!(
        contents,
        r"(((public|native| )*))(\w+\(.*\).*\{)",
        |cap: &Captures| format!("{}fun {}", &cap[1], &cap[4])
    );
    fs::write(out_path, contents.as_bytes()).unwrap();
}
