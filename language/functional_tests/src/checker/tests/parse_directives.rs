// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{checker::*, common::Sp, errors::*};

fn check_sp(sp: &Sp<Directive>, d: &Directive, start: usize, end: usize) -> Result<()> {
    assert_eq!(sp.start, start);
    assert_eq!(sp.end, end);
    assert_eq!(sp.inner, *d);
    Ok(())
}

#[test]
fn check_one() -> Result<()> {
    let directives = Directive::parse_line("// check: abc")?;
    assert!(directives.len() == 1);
    check_sp(&directives[0], &Directive::Check("abc".to_string()), 10, 13)
}

#[test]
fn not_one() -> Result<()> {
    let directives = Directive::parse_line("// not: abc")?;
    assert!(directives.len() == 1);

    check_sp(&directives[0], &Directive::Not("abc".to_string()), 8, 11)
}

#[test]
fn check_two() -> Result<()> {
    let directives = Directive::parse_line("// check: abc  f")?;
    assert!(directives.len() == 2);

    check_sp(&directives[0], &Directive::Check("abc".to_string()), 10, 13)?;
    check_sp(&directives[1], &Directive::Check("f".to_string()), 15, 16)
}

#[test]
fn not_two() -> Result<()> {
    let directives = Directive::parse_line("// not: abc  f")?;
    assert!(directives.len() == 2);

    check_sp(&directives[0], &Directive::Not("abc".to_string()), 8, 11)?;
    check_sp(&directives[1], &Directive::Not("f".to_string()), 13, 14)
}

#[test]
fn compact() -> Result<()> {
    let directives = Directive::parse_line("//not:a b c")?;
    assert!(directives.len() == 3);

    check_sp(&directives[0], &Directive::Not("a".to_string()), 6, 7)?;
    check_sp(&directives[1], &Directive::Not("b".to_string()), 8, 9)?;
    check_sp(&directives[2], &Directive::Not("c".to_string()), 10, 11)
}

#[test]
fn check_quoted() -> Result<()> {
    let directives = Directive::parse_line(r#"// check: "abc  def\\\t\n\r\"" "#)?;
    assert!(directives.len() == 1);

    check_sp(
        &directives[0],
        &Directive::Check("abc  def\\\t\n\r\"".to_string()),
        10,
        30,
    )
}

#[test]
fn check_two_quoted() -> Result<()> {
    let directives = Directive::parse_line(r#"// check: " " "\"" "#)?;
    assert!(directives.len() == 2);

    check_sp(&directives[0], &Directive::Check(" ".to_string()), 10, 13)?;
    check_sp(&directives[1], &Directive::Check("\"".to_string()), 14, 18)
}

#[test]
fn check_quoted_and_unquoted_mixed() -> Result<()> {
    let directives = Directive::parse_line(r#"// check: " " abc  "" "#)?;
    assert!(directives.len() == 3);

    check_sp(&directives[0], &Directive::Check(" ".to_string()), 10, 13)?;
    check_sp(&directives[1], &Directive::Check("abc".to_string()), 14, 17)?;
    check_sp(&directives[2], &Directive::Check("".to_string()), 19, 21)
}

#[test]
fn check_empty() -> Result<()> {
    Directive::parse_line("// check:").unwrap_err();
    Ok(())
}

#[test]
fn not_empty() -> Result<()> {
    Directive::parse_line("// not:").unwrap_err();
    Ok(())
}
