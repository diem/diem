// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    checker::*,
    compiler::Compiler,
    config::global::Config as GlobalConfig,
    evaluator::eval,
    preprocessor::{build_transactions, split_input},
};
use std::{env, fs::read_to_string, io::Write, iter, path::Path};
use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};

fn at_most_n_chars(s: impl IntoIterator<Item = char>, n: usize) -> String {
    let mut it = s.into_iter();
    let mut s = String::new();
    for _ in 0..n {
        match it.next() {
            Some(c) => s.push(c),
            None => return s,
        }
    }
    if it.next().is_some() {
        s.push_str("...")
    }
    s
}

fn at_most_n_before_and_m_after(
    s: &str,
    n: usize,
    start: usize,
    end: usize,
    m: usize,
) -> (String, String, String) {
    let before = at_most_n_chars(s[..start].chars().rev(), n)
        .chars()
        .rev()
        .collect();
    let matched = s[start..end].to_string();
    let after = at_most_n_chars(s[end..].chars(), m).chars().collect();
    (before, matched, after)
}

fn env_var(var_name: &str) -> String {
    env::var(var_name)
        .unwrap_or_else(|_| "".to_string())
        .to_ascii_lowercase()
}

fn pretty_mode() -> bool {
    let pretty = env_var("PRETTY");
    pretty == "1" || pretty == "true"
}

// Runs all tests under the test/testsuite directory.
pub fn functional_tests<TComp: Compiler>(
    compiler: TComp,
    path: &Path,
) -> datatest_stable::Result<()> {
    let input = read_to_string(path)?;

    let lines: Vec<String> = input.lines().map(|line| line.to_string()).collect();

    let (config, directives, transactions) = split_input(&lines)?;
    let config = GlobalConfig::build(&config)?;
    let transactions = build_transactions(&config, &transactions)?;

    let log = eval(&config, compiler, &transactions)?;

    let res = match_output(&log, &directives);

    let errs = match res.status {
        MatchStatus::Success => return Ok(()),
        MatchStatus::Failure(errs) => errs,
    };

    // Set up colored output stream for error rendering.
    let bufwtr = BufferWriter::stdout(ColorChoice::Auto);
    let mut output = bufwtr.buffer();

    // Helpers for directives and matches.
    macro_rules! print_directive {
        ($idx: expr) => {{
            let d = &directives[$idx];
            write!(output, "{} | {}", d.line + 1, &lines[d.line][..d.start])?;
            output.set_color(ColorSpec::new().set_underline(true))?;
            write!(output, "{}", &lines[d.line][d.start..d.end])?;
            output.reset()?;
            write!(output, "{}", &lines[d.line][d.end..])
        }};
    }

    macro_rules! print_match {
        ($indent: expr, $is_positive: expr, $m: expr) => {{
            let m: &Match = $m;
            let indent: &str = $indent;
            let prefix = format!("[{}] ", m.entry_id);
            let (before, matched, after) =
                at_most_n_before_and_m_after(&res.text[m.entry_id], 30, m.start, m.end, 50);
            write!(output, "{}", indent)?;
            write!(output, "{}{}", prefix, before)?;
            output.set_color(ColorSpec::new().set_underline(true).set_fg(Some(
                if $is_positive {
                    Color::Green
                } else {
                    Color::Red
                },
            )))?;
            write!(output, "{}", matched)?;
            output.reset()?;
            writeln!(output, "{}", after)?;

            let offset = prefix.chars().count() + before.chars().count();
            write!(output, "{}", indent)?;
            write!(
                output,
                "{}",
                iter::repeat(' ').take(offset).collect::<String>()
            )?;
            print_directive!(m.pat_id)?;
            writeln!(output)
        }};
    }

    writeln!(output)?;
    writeln!(
        output,
        "{}",
        iter::repeat('=').take(100).collect::<String>()
    )?;
    writeln!(output, "{}", path.display())?;
    writeln!(output)?;

    // Render the evaluation log.
    output.set_color(ColorSpec::new().set_bold(true).set_fg(Some(Color::Yellow)))?;
    write!(output, "info: ")?;
    output.set_color(ColorSpec::new().set_bold(true))?;
    writeln!(output, "Evaluation Outputs")?;
    output.reset()?;
    if pretty_mode() {
        writeln!(
            output,
            "{}",
            format!("{}", log)
                .lines()
                .map(|line| format!("    {}\n", line))
                .collect::<String>()
        )?;
    } else {
        for (id, entry) in res.text.iter().enumerate() {
            writeln!(output, "    [{}] {}", id, entry)?;
        }
        writeln!(output)?;
        writeln!(
            output,
            "    Note: enable pretty printing by setting 'env PRETTY=1'."
        )?;
        writeln!(output)?;
    }
    writeln!(output)?;

    // Render previously successful matches if any.
    if !res.matches.is_empty() {
        output.set_color(ColorSpec::new().set_bold(true).set_fg(Some(Color::Yellow)))?;
        write!(output, "info: ")?;
        output.set_color(ColorSpec::new().set_bold(true))?;
        writeln!(output, "Successful Matches")?;
        output.reset()?;
        for m in &res.matches {
            print_match!("    ", true, m)?;
            writeln!(output)?;
        }
        writeln!(output)?;
    }

    // Render errors.
    for err in errs {
        output.set_color(ColorSpec::new().set_bold(true).set_fg(Some(Color::Red)))?;
        write!(output, "error: ")?;
        output.reset()?;
        match err {
            MatchError::UnmatchedErrors(errs) => {
                output.set_color(ColorSpec::new().set_bold(true))?;
                writeln!(output, "Unmatched Errors")?;
                output.reset()?;
                for id in errs.iter() {
                    write!(output, "    [{}] ", id)?;
                    writeln!(output, "{}", at_most_n_chars(res.text[*id].chars(), 80))?;
                }
            }
            MatchError::NegativeMatch(m) => {
                output.set_color(ColorSpec::new().set_bold(true))?;
                writeln!(output, "Negative Match")?;
                output.reset()?;
                print_match!("    ", false, &m)?;
            }
            MatchError::UnmatchedDirectives(dirs) => {
                output.set_color(ColorSpec::new().set_bold(true))?;
                writeln!(output, "Unmatched Directives")?;
                output.reset()?;
                for idx in &dirs {
                    write!(output, "    ")?;
                    print_directive!(*idx)?;
                    writeln!(output)?;
                }
                writeln!(output)?;
                writeln!(output)?;
            }
        }
    }
    writeln!(output)?;
    bufwtr.print(&output)?;

    panic!("test failed")
}
