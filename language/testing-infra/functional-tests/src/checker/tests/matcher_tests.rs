// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{checker::*, evaluator::*};
use std::fmt::{self};
use thiserror::Error;

macro_rules! log {
    ($($e: expr),*) => {{
        let mut log = EvaluationLog::new();
        $(log.append($e);)*
        log
    }};
}

#[derive(Error)]
struct DummyError(String);

impl fmt::Display for DummyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Debug for DummyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

macro_rules! err {
    ($s: expr) => {{
        EvaluationOutput::Error(Box::new(DummyError(($s).to_string()).into()))
    }};
}

macro_rules! dirs {
    ($($s: expr),*) => {{
        let mut directives = vec![];
        $(directives.extend(Directive::parse_line($s).unwrap().into_iter().map(|sp| sp.into_inner()));)*
        directives
    }};
}

#[test]
fn match_check_simple_1() {
    let log = log![err!("foo bar")];
    let directives = dirs!["// check: \"o b\""];
    let res = match_output(&log, &directives);
    assert!(res.is_success());
    assert_eq!(res.matches.len(), 1);
}

#[test]
fn match_check_simple_2() {
    let log = log![err!("foo bar")];
    let directives = dirs!["// check: foo bar"];
    let res = match_output(&log, &directives);
    assert!(res.is_success());
    assert_eq!(res.matches.len(), 2);
}

#[test]
fn match_not() {
    let log = log![err!("foo"), err!("bar"), err!("baz")];
    let directives = dirs!["// check: bar", "// not: baz"];
    let res = match_output(&log, &directives);
    assert!(res.is_failure());
    assert_eq!(res.matches.len(), 1);
}

#[test]
fn match_mixed_1() {
    let log = log![err!("foo bar"), err!("abc")];
    let directives = dirs!["// check: foo", "// not: rab", "// check: abc"];
    let res = match_output(&log, &directives);
    assert!(res.is_success());
    assert_eq!(res.matches.len(), 2);
}

#[test]
fn match_mixed_2() {
    let log = log![err!("abc de"), err!("foo 3"), err!("5 bar 6"), err!("7")];
    println!("{:?}", log);
    let directives = dirs![
        "// not: 1",
        "// not: 2",
        "// check: 3",
        "// not: 4",
        "// check: 5",
        "// not: 6"
    ];
    let res = match_output(&log, &directives);
    assert!(res.is_failure());
    assert_eq!(res.matches.len(), 2);
}

#[test]
fn unmatched_directives_1() {
    let log = log![err!("foo bar")];
    let directives = dirs!["// check: foo", "// check: bbar"];
    let res = match_output(&log, &directives);
    assert!(res.is_failure());
    assert_eq!(res.matches.len(), 1);
}

#[test]
fn unmatched_directives_2() {
    let log = log![err!("foo bar"), err!("baz")];
    let directives = dirs!["// check: oo", "// check: baz", "// check: zz"];
    let res = match_output(&log, &directives);
    assert!(res.is_failure());
    assert_eq!(res.matches.len(), 2);
}

#[test]
fn unmatched_errors_1() {
    let log = log![err!("foo")];
    let directives: Vec<Directive> = vec![];
    let res = match_output(&log, &directives);
    assert!(res.is_failure());
    assert_eq!(res.matches.len(), 0);
}

#[test]
fn unmatched_errors_2() {
    let log = log![err!("foo"), err!("bar")];
    let directives = dirs!["// check: bar"];
    let res = match_output(&log, &directives);
    assert!(res.is_failure());
    assert_eq!(res.matches.len(), 1);
}
