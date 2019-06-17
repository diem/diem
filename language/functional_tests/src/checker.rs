// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    errors::*,
    evaluator::{EvaluationResult, Stage, Status},
};
use filecheck;
use std::slice::SliceConcatExt;

/// A directive specifies a pattern in the output.
/// Directives are extracted from comments starting with "//".
#[derive(Debug, Clone)]
pub enum Directive {
    /// Matches the specified stage in the output. Acts as a barrier.
    Stage(Stage),
    /// Used to build the filecheck checker. Right now all comments except the ones that are
    /// recognized as other directives goes here.
    Check(String),
}

impl Directive {
    /// Tries to parse the given string into a directive. Returns an option indicating whether
    /// the given input is a directive or not. Errors when the input looks like a directive but
    /// is ill-formed.
    pub fn try_parse(s: &str) -> Result<Option<Directive>> {
        let s1 = s.trim_start();
        if !s1.starts_with("//") {
            return Ok(None);
        }
        let s2 = s1[2..].trim_start();
        if s2.starts_with("stage: ") {
            let s3 = s2[7..].trim_start().trim_end();
            return Ok(Some(Directive::Stage(Stage::parse(s3)?)));
        }
        Ok(Some(Directive::Check(s.to_string())))
    }
}

/// Check the output using filecheck checker.
pub fn run_filecheck(output: &str, checks: &str) -> Result<bool> {
    let mut builder = filecheck::CheckerBuilder::new();
    builder.text(checks)?;
    let checker = builder.finish();
    // filecheck allows one to pass in a variable map, however we're not using it
    if !checker.check(output, filecheck::NO_VARIABLES)? {
        return Err(ErrorKind::CheckerFailure.into());
    }
    Ok(!checker.is_empty())
}

/// Verifies the directives against the given evaluation result.
pub fn check(res: &EvaluationResult, directives: &[Directive]) -> Result<()> {
    let mut checks: Vec<String> = vec![];
    let mut outputs: Vec<String> = vec![];
    let mut did_run_checks = false;

    let mut i = 0;

    for directive in directives {
        match directive {
            Directive::Check(check) => {
                checks.push(check.clone());
            }
            Directive::Stage(barrier) => loop {
                if i >= res.stages.len() {
                    return Err(ErrorKind::Other(format!(
                        "no stage '{:?}' in the output",
                        barrier
                    ))
                    .into());
                }
                let (stage, output) = &res.stages[i];
                if stage < barrier {
                    outputs.push(output.to_string());
                    i += 1;
                } else if stage == barrier {
                    did_run_checks |= run_filecheck(&outputs.join("\n"), &checks.join("\n"))?;
                    checks.clear();
                    outputs.clear();
                    outputs.push(output.to_string());
                    i += 1;
                    break;
                } else {
                    return Err(ErrorKind::Other(format!(
                        "no stage '{:?}' in the output",
                        barrier
                    ))
                    .into());
                }
            },
        }
    }

    for (_, output) in res.stages[i..].iter() {
        outputs.push(output.clone());
    }
    did_run_checks |= run_filecheck(&outputs.join("\n"), &checks.join("\n"))?;

    if res.status == Status::Failure && !did_run_checks {
        return Err(ErrorKind::Other(format!(
            "program failed at stage '{:?}', no directives found, assuming failure",
            res.stages.last().unwrap().0
        ))
        .into());
    }

    Ok(())
}
