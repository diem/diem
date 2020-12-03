// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    checker::*,
    common::LineSp,
    compiler::Compiler,
    evaluator::{eval, EvaluationLog, EvaluationOutput},
    preprocessor::{build_transactions, extract_global_config, split_input},
};
use difference::Changeset;
use regex::Regex;
use std::{
    env,
    fmt::Write as FmtWrite,
    fs::{read_to_string, File},
    io::Write,
    iter,
    path::{Path, PathBuf},
};
use termcolor::{Buffer, BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};

pub const PRETTY: &str = "PRETTY";
pub const FILTER: &str = "FILTER";
pub const UPDATE_BASELINE: &str = "UPDATE_BASELINE";

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
    let pretty = env_var(PRETTY);
    pretty == "1" || pretty == "true"
}

fn update_baseline() -> bool {
    let update = env_var(UPDATE_BASELINE);
    update == "1" || update == "true"
}

fn print_stage(haystack: &str) -> bool {
    env::var(FILTER)
        .map(|needle| {
            let needle = Regex::new(&needle).unwrap();
            needle.is_match(haystack)
        })
        .unwrap_or(true)
}

fn write_horizontal_line(output: &mut Buffer, term_width: usize) -> std::io::Result<()> {
    writeln!(
        output,
        "{}",
        iter::repeat('=').take(term_width).collect::<String>()
    )
}

fn write_test_header(
    output: &mut Buffer,
    term_width: usize,
    test_file_path: &Path,
) -> std::io::Result<()> {
    writeln!(output)?;
    write_horizontal_line(output, term_width)?;
    writeln!(output, "{}", test_file_path.display())?;
    writeln!(output)
}

fn check_or_update_expected_output(
    bufwtr: &BufferWriter,
    output: &mut Buffer,
    term_width: usize,
    test_file_path: &Path,
    exp_file_path: &Path,
    log: &EvaluationLog,
) -> datatest_stable::Result<()> {
    let mut text = String::new();
    for (idx, entry) in log.to_text_for_matching().into_iter().enumerate() {
        writeln!(&mut text, "[{}] {}", idx, entry)?;
    }

    let expected = read_to_string(&exp_file_path)?;

    let changeset = Changeset::new(&expected, &text, "\n");
    // TODO: make this less sensitive to spaces.
    if changeset.distance != 0 {
        if update_baseline() {
            let mut f = File::create(&exp_file_path)?;
            f.write_all(text.as_bytes())?;
        } else {
            write_test_header(output, term_width, test_file_path)?;

            writeln!(output, "{}", changeset)?;

            writeln!(
                output,
                "    Note: run with `env UPDATE_BASELINE=1` to update the exp files."
            )?;
            writeln!(output)?;

            write_horizontal_line(output, term_width)?;
            writeln!(output)?;

            bufwtr.print(output)?;
            panic!("test failed")
        }
    }

    Ok(())
}

fn run_checker_directives(
    bufwtr: &BufferWriter,
    output: &mut Buffer,
    term_width: usize,
    test_file_path: &Path,
    lines: &[String],
    log: &EvaluationLog,
    directives: &[LineSp<Directive>],
) -> datatest_stable::Result<()> {
    let res = match_output(&log, directives);

    let errs = match res.status {
        MatchStatus::Success => return Ok(()),
        MatchStatus::Failure(errs) => errs,
    };

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

    write_test_header(output, term_width, test_file_path)?;

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
            log.outputs
                .iter()
                .enumerate()
                .map(|(id, entry)| {
                    match entry {
                        EvaluationOutput::Error(err) => {
                            format!("[{}] Error: {}\n", id, err.root_cause())
                        }
                        _ => format!("[{}] {}\n", id, entry),
                    }
                })
                .filter(|x| print_stage(&x))
                .collect::<String>()
                .lines()
                .map(|line| format!("    {}\n", line))
                .collect::<String>()
        )?;
    } else {
        for (id, entry) in res.text.iter().enumerate() {
            if print_stage(entry) {
                writeln!(output, "    [{}] {}", id, entry)?;
            }
        }
        writeln!(output)?;
        writeln!(
            output,
            "    Note: enable pretty printing by setting 'env PRETTY=1'."
        )?;
        writeln!(
            output,
            "          You can filter logs by setting 'env FILTER=\"<regex pattern>\"'."
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
    write_horizontal_line(output, term_width)?;
    writeln!(output)?;
    bufwtr.print(&output)?;

    panic!("test failed")
}

// Runs all tests under the test/testsuite directory.
pub fn functional_tests<TComp: Compiler>(
    compiler: TComp,
    test_file_path: &Path,
) -> datatest_stable::Result<()> {
    let mut exp_file_path = PathBuf::from(test_file_path);
    exp_file_path.set_extension("exp");
    let exp_mode = exp_file_path.exists();

    let input = read_to_string(test_file_path)?;

    let lines: Vec<String> = input.lines().map(|line| line.to_string()).collect();
    let config = extract_global_config(&lines, exp_mode)?;
    let (directives, transactions) = split_input(&lines, &config)?;
    let commands = build_transactions(&config, &transactions)?;

    let log = eval(&config, compiler, &commands)?;

    // Set up colored output stream for error rendering.
    let bufwtr = BufferWriter::stdout(ColorChoice::Auto);
    let mut output = bufwtr.buffer();

    let term_width = match term_size::dimensions() {
        Some((w, _h)) => w,
        _ => 80,
    };

    if exp_mode {
        check_or_update_expected_output(
            &bufwtr,
            &mut output,
            term_width,
            test_file_path,
            &exp_file_path,
            &log,
        )?;
    } else {
        run_checker_directives(
            &bufwtr,
            &mut output,
            term_width,
            test_file_path,
            &lines,
            &log,
            &directives,
        )?;
    }

    Ok(())
}
