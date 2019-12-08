// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::transaction::Script;
use std::fmt;

#[test]
fn test_code_fmt() {
    let expect_output = r#"Script {
    code: "6d6f7665",
    args: [],
}"#;
    let script = Script::new(b"move".to_vec(), vec![]);
    let mut output = String::new();
    fmt::write(&mut output, format_args!("{:#?}", script))
        .expect("Error occurred while trying to format Script.");
    assert_eq!(output, expect_output);
    println!("{:#?}", script);
}
