// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    checker::{check, run_filecheck, Directive},
    evaluator::{EvaluationOutput, EvaluationResult, OutputType, Stage, Status},
};
use vm::{
    file_format::{empty_module, CompiledModuleMut},
    vm_string::VMString,
};

#[test]
fn parse_directives() {
    for s in &[
        "abc",
        "// not a directive",
        "//",
        "// stage:   runtime  bad  ",
        "// stage: bad stage",
        "// stage: ",
    ] {
        s.parse::<Directive>().unwrap_err();
    }

    for s in &[
        "// check: abc",
        "  // check: abc",
        "//not: foo",
        "// sameln: abc",
        "// nextln: abc",
        "// unordered: abc",
        "// regex: X=aaa",
        "// stage: parser",
        "// stage: compiler",
        "// stage: verifier",
        "// stage: runtime",
        "// stage:   runtime  ",
    ] {
        s.parse::<Directive>().unwrap();
    }
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

fn make_directives(s: &str) -> Vec<Directive> {
    s.lines()
        .filter_map(|s| {
            if let Ok(directive) = s.parse::<Directive>() {
                return Some(directive);
            }
            None
        })
        .collect()
}

fn make_output(module: CompiledModuleMut) -> EvaluationOutput {
    EvaluationOutput::Output(Box::new(OutputType::CompiledModule(
        module.freeze().unwrap(),
    )))
}

#[rustfmt::skip]
#[test]
fn check_basic() {
    let mut module = empty_module();
    module.user_strings = vec![VMString::new("foo")];
    let foo_mod = make_output(module.clone());
    module.user_strings = vec![VMString::new("bar")];
    let bar_mod = make_output(module.clone());
    module.user_strings = vec![VMString::new("baz")];
    let baz_mod = make_output(module.clone());

    let res = EvaluationResult {
        outputs: vec![
            EvaluationOutput::Transaction,
            EvaluationOutput::Stage(Stage::Compiler),
            foo_mod,
            EvaluationOutput::Stage(Stage::Verifier),
            baz_mod,
            EvaluationOutput::Stage(Stage::Runtime),
            bar_mod,
        ],
        status: Status::Success,
        use_debug_output: false,
    };

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
    let mut module = empty_module();
    module.user_strings = vec![VMString::new("foo")];
    let foo_mod = make_output(module.clone());
    module.user_strings = vec![VMString::new("baz")];
    let baz_mod = make_output(module.clone());

    let res = EvaluationResult {
        outputs: vec![
            EvaluationOutput::Transaction,
            EvaluationOutput::Stage(Stage::Compiler),
            foo_mod,
            EvaluationOutput::Stage(Stage::Verifier),
            baz_mod,
        ],
        status: Status::Success,
        use_debug_output: false,
    };

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
    let mut module = empty_module();
    module.user_strings = vec![VMString::new("baz")];
    let baz_mod = make_output(module.clone());
    let res = EvaluationResult {
        outputs: vec![
            EvaluationOutput::Transaction,
            EvaluationOutput::Stage(Stage::Verifier),
            baz_mod,
        ],
        status: Status::Success,
        use_debug_output: false,
    };

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
