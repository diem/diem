// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    errors::*,
    evaluator::{EvaluationOutput, EvaluationResult, Stage, Status},
};
use filecheck;
use std::{slice::SliceConcatExt, str::FromStr};

/// A directive specifies a pattern in the output.
/// Directives are extracted from comments starting with "//".
#[derive(Debug, Clone)]
pub enum Directive {
    Transaction,
    /// Matches the specified stage in the output. Acts as a barrier.
    Stage(Stage),
    /// Used to build the filecheck checker. Right now all comments except the ones that are
    /// recognized as other directives goes here.
    Check(String),
}

impl FromStr for Directive {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let s = s.trim_start().trim_end();
        if !s.starts_with("//") {
            return Err(ErrorKind::Other("directives must start with //".to_string()).into());
        }
        let s = s[2..].trim_start();
        if s.starts_with("stage:") {
            let s = s[6..].trim_start().trim_end();
            if s.is_empty() {
                return Err(ErrorKind::Other("stage cannot be empty".to_string()).into());
            }
            return Ok(Directive::Stage(s.parse::<Stage>()?));
        }
        if s == "transaction" {
            // TODO: implement transaction directive
            unimplemented!();
        }
        Ok(Directive::Check(s.to_string()))
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
                if i >= res.outputs.len() {
                    return Err(ErrorKind::Other(format!(
                        "no stage '{:?}' in the output",
                        barrier
                    ))
                    .into());
                }
                match &res.outputs[i] {
                    EvaluationOutput::Stage(stage) => {
                        if stage < barrier {
                            i += 1;
                            continue;
                        } else if stage > barrier {
                            return Err(ErrorKind::Other(format!(
                                "no stage '{:?}' in the current transaction",
                                barrier
                            ))
                            .into());
                        } else {
                            did_run_checks |=
                                run_filecheck(&outputs.join("\n"), &checks.join("\n"))?;
                            checks.clear();
                            outputs.clear();
                            break;
                        }
                    }
                    EvaluationOutput::Output(s) | EvaluationOutput::Error(s) => {
                        outputs.push(s.to_string());
                        i += 1;
                    }
                    EvaluationOutput::Transaction => {
                        i += 1;
                    }
                }
            },
            // TODO: implement transaction directive
            Directive::Transaction => unimplemented!(),
        }
    }

    for output in &res.outputs[i..] {
        match output {
            EvaluationOutput::Output(s) | EvaluationOutput::Error(s) => {
                outputs.push(s.to_string());
            }
            EvaluationOutput::Stage(_) | EvaluationOutput::Transaction => {}
        }
    }
    did_run_checks |= run_filecheck(&outputs.join("\n"), &checks.join("\n"))?;

    if res.status == Status::Failure && !did_run_checks {
        return Err(ErrorKind::Other(format!(
            "program failed at transaction {}, stage {:?}, no directives found, assuming failure",
            res.get_transaction_count(),
            res.get_last_stage().unwrap(),
        ))
        .into());
    }

    Ok(())
}
