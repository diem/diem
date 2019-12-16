// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bytecode_to_boogie::cli::Options;
use bytecode_to_boogie::driver::Driver;
use goldenfile;
use itertools::Itertools;
use libra_tools::tempdir::TempPath;
use prettydiff::{basic::DiffOp, diff_lines};
use std::{env, fs, fs::File, io::Error, io::Read, io::Write, path::Path};

pub fn test(sources: &[&str]) {
    // Configure options.
    let mut args: Vec<&str> = vec![
        "-v=debug",
        // Right now we configure boogie to only do type check.
        "-B=-noinfer",
        "-B=-noVerify",
    ];
    args.extend(sources.iter());
    let mut options = Options::default();
    options.initialize_from_args(&args.iter().map(|s| s.to_string()).collect_vec());
    options.setup_logging_for_test();

    // Run the translator.
    let mut driver = Driver::new(&options);
    let (prelude, generated) = driver.run_for_test();

    if env::var("VERIFY_BPL_GOLDEN").is_ok() {
        // Verify or update golden files. We use the last name in the sources file name list
        // for the golden file base. Note we do not pass in the prelude below, but only
        // put the proper generated code into the golden file.
        verify_or_update_golden(sources[sources.len() - 1], &generated);
    }

    // Run boogie on the result.
    let mut boogie_str = prelude;
    boogie_str.push_str(&generated);
    let temp_path = TempPath::new();
    temp_path.create_as_dir().unwrap();
    let base_name = format!(
        "{}.bpl",
        Path::new(sources[sources.len() - 1])
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
    );
    let boogie_file_path = temp_path
        .path()
        .join(base_name)
        .to_str()
        .unwrap()
        .to_string();

    // In order to debug errors in the output, one may uncomment below to find the output
    // file at a stable, persistent location.
    //    let mut boogie_file_path = std::path::PathBuf::new();
    //    boogie_file_path.push("/tmp/output.bpl").to_string();

    fs::write(&boogie_file_path, boogie_str).expect("cannot write boogie file");

    if env::var("BOOGIE_EXE").is_ok() {
        // Call boogie to verify results. Right now, we call boogie for type checking only.
        assert!(driver.call_boogie(&boogie_file_path));
    }
}

/// Verifies or updates golden file for the given generated boogie source.
fn verify_or_update_golden(file_name: &str, boogie_str: &str) {
    // The mint created here will automatically verify/update the
    // file once it is dropped.
    let mut mint = goldenfile::Mint::new("tests/goldenfiles");
    let goldenfile_name = format!(
        "{}.bpl",
        Path::new(file_name).file_stem().unwrap().to_str().unwrap()
    );
    let mut goldenfile = mint
        .new_goldenfile_with_differ(goldenfile_name, Box::new(file_diff))
        .expect("failed creating golden file");
    write!(goldenfile, "{}", clean_for_golden(&boogie_str)).expect("failed writing golden file");
}

/// Clean a content to be usable as a golden file. Currently, we ensure there are no
/// trailing whitespaces and no empty last line, because this is required by git-checks.sh.
fn clean_for_golden(content: &str) -> String {
    let mut res = String::new();
    for line in content.lines() {
        res.push_str(line.trim_end());
        res.push_str("\n");
    }
    res = res.trim_end().to_string(); // removes empty lines at end
    res.push_str("\n"); // adds back a single newline
    res
}

/// Implements a custom differ for golden tests. The existing differ prints the full
/// text (also unchanged lines) which is infeasible for large sources. The diff output
/// is rather rudimentary, but this is fine because the diff is best analyzed anyway
/// using the users standard diff tools.
fn file_diff(old: &Path, new: &Path) {
    let print_lines = |lines: &[&str], prefix: &str| {
        for line in lines {
            println!("{}{}", prefix, line);
        }
    };

    let print_context = |lines: &[&str]| {
        if lines.len() <= 3 {
            print_lines(lines, "= ");
        } else {
            print_lines(&lines[..1], "= ");
            println!("= ... ({} lines)", lines.len() - 2);
            print_lines(&lines[lines.len() - 1..], "= ");
        }
    };

    let read_file = |path: &Path| -> Result<String, Error> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(contents)
    };

    let old_content = match read_file(old) {
        Ok(c) => c,
        _ => {
            panic!(
                "
Cannot read golden file at {}. In order to create initial golden files,
run the test with env variable REGENERATE_GOLDENFILES=1 set.
",
                old.display()
            );
        }
    };
    let new_content = read_file(new).unwrap();
    if old_content == new_content {
        return;
    }
    let diff = diff_lines(&new_content, &old_content);
    for d in diff.diff() {
        match d {
            DiffOp::Equal(lines) => print_context(lines),
            DiffOp::Insert(lines) => print_lines(lines, "+ "),
            DiffOp::Remove(lines) => print_lines(lines, "- "),
            DiffOp::Replace(old, new) => {
                print_lines(old, "- ");
                print_lines(new, "+ ");
            }
        }
    }
    panic!(
        "
Old and new differ!
Use env variable REGENERATE_GOLDENFILES=1 to regenerate golden files.
Then use your favorite diff tool to verify you are good with the changes.
"
    );
}
