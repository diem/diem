// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    checker::{check, run_filecheck, Directive},
    evaluator::{EvaluationResult, Stage, Status},
};

#[test]
fn parse_directives() {
    assert!(Directive::try_parse("abc").unwrap().is_none());
    Directive::try_parse("// check: abc").unwrap().unwrap();
    Directive::try_parse("  // check: abc").unwrap().unwrap();
    Directive::try_parse("//not: foo").unwrap().unwrap();
    Directive::try_parse("// stage: parser").unwrap().unwrap();

    Directive::try_parse("// stage: compiler").unwrap().unwrap();
    Directive::try_parse("// stage: verifier").unwrap().unwrap();
    Directive::try_parse("// stage: runtime").unwrap().unwrap();
    Directive::try_parse("// stage:   runtime  ")
        .unwrap()
        .unwrap();

    Directive::try_parse("// stage:   runtime  bad  ").unwrap_err();
    Directive::try_parse("// stage: bad stage").unwrap_err();
    Directive::try_parse("// stage: ").unwrap_err();
}

#[rustfmt::skip]
#[test]
fn filecheck() {
    run_filecheck("AAA BBB CCC", r"
        // check: AAA
        // check: CCC
    ").unwrap();

    run_filecheck("AAA BBB CCC", r"
        // check: AAA
        // not: BBB
        // check: CCC
    ").unwrap_err();
}

macro_rules! eval_result {
    ($status: expr, $($stage: expr, $output: expr),* $(,)*) => {
        {
            EvaluationResult {
                stages: vec![$(($stage, $output.to_string())),*],
                status: $status,
            }
        }
    };
}

fn make_directives(s: &str) -> Vec<Directive> {
    s.lines()
        .filter_map(|s| {
            if let Ok(directive) = Directive::try_parse(s) {
                return directive;
            }
            None
        })
        .collect()
}

#[rustfmt::skip]
#[test]
fn check_basic() {
    let res = eval_result!(
        Status::Success,
        Stage::Compiler, "foo",
        Stage::Verifier, "baz",
        Stage::Runtime, "bar"
    );

    check(&res, &make_directives(r"
        // check: foo
        // stage: runtime
        // check: bar
    ")).unwrap();

    check(&res, &make_directives(r"
        // stage: compiler
        // stage: verifier
        // check: bar
    ")).unwrap();

    check(&res, &make_directives(r"
        // stage: verifier
        // check: foo
    ")).unwrap_err();

    check(&res, &make_directives(r"
        // check: foo
        // check: bar
    ")).unwrap();

    check(&res, &make_directives(r"
        // check: baz
        // check: foo
    ")).unwrap_err();
}

#[rustfmt::skip]
#[test]
fn check_match_twice() {
    let res = eval_result!(
        Status::Success,
        Stage::Compiler, "foo",
        Stage::Verifier, "bar",
    );

    check(&res, &make_directives(r"
        // check: foo
        // check: foo
    ")).unwrap_err();

    check(&res, &make_directives(r"
        // stage: compiler
        // check: foo
        // check: foo
        // stage: verifier
    ")).unwrap_err();
}

#[rustfmt::skip]
#[test]
fn check_no_stage() {
    let res = eval_result!(
        Status::Success,
        Stage::Verifier, "",
    );

    check(&res, &make_directives(r"
        // stage: verifier
    ")).unwrap();

    check(&res, &make_directives(r"
        // stage: compiler
    ")).unwrap_err();

    check(&res, &make_directives(r"
        // stage: runtime
    ")).unwrap_err();
}
