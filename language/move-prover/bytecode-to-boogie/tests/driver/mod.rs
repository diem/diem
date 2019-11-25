// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bytecode_source_map::source_map::{ModuleSourceMap, SourceMap};
use bytecode_to_boogie::translator::BoogieTranslator;
use bytecode_verifier::VerifiedModule;
use goldenfile;
use ir_to_bytecode::{compiler::compile_module, parser::ast::Loc, parser::parse_module};
use libra_tools::tempdir::TempPath;
use libra_types::account_address::AccountAddress;
use prettydiff::{basic::DiffOp, diff_lines};
use std::{env, fs, fs::File, io::Error, io::Read, io::Write, path::Path, process::Command};
use stdlib::{stdlib_modules, stdlib_source_map};

fn compile_files(file_names: Vec<&str>) -> (Vec<VerifiedModule>, SourceMap<Loc>) {
    let mut verified_modules = stdlib_modules().to_vec();
    let mut source_maps = stdlib_source_map().to_vec();
    let files_len = file_names.len();
    let dep_files = &file_names[0..files_len];

    // assuming the last file is a program that might contain a script
    let address = AccountAddress::default();
    for file_name in dep_files {
        let code = fs::read_to_string(file_name).unwrap();
        let module = parse_module(&code).unwrap();
        let (compiled_module, source_map) =
            compile_module(address, module, &verified_modules).expect("module failed to compile");
        let verified_module_res = VerifiedModule::new(compiled_module);

        match verified_module_res {
            Err(e) => {
                println!("vm verification errors!");
                for s in &e.1 {
                    println!("status: {:?}", s);
                }
                panic!("{:?}", e.0);
            }
            Ok(verified_module) => {
                verified_modules.push(verified_module);
                source_maps.push(source_map);
            }
        }
    }
    (verified_modules, source_maps)
}

#[allow(unused_variables)]
pub fn generate_boogie(file_name: &str, target_modules: &[&str]) -> String {
    let mut file_names = vec![];
    file_names.push(file_name);
    let (modules, source_maps) = compile_files(file_names.to_vec());

    if cfg!(feature = "golden") {
        // Verify or update golden file. This will run its own translate focused to the target test,
        // so no need to pass in generated content.
        verify_or_update_golden(&modules, &source_maps, file_name, target_modules);
    }

    // Run regular translation producing output for execution with boogie.
    let mut ts = BoogieTranslator::new(&modules, &source_maps);
    let mut res = String::new();

    // handwritten boogie code
    let written_code = fs::read_to_string("src/bytecode_instrs.bpl").unwrap();
    res.push_str(&written_code);
    res.push_str(&ts.translate());

    res
}

pub fn run_boogie(boogie_str: &str) {
    let temp_path = TempPath::new();
    temp_path.create_as_dir().unwrap();
    let boogie_file_path = temp_path.path().join("output.bpl");
    // In order to debug errors in the output, one may uncomment below to find the output
    // file at a stable location.
    //    let mut boogie_file_path = std::path::PathBuf::new();
    //    boogie_file_path.push("/tmp/output.bpl");
    fs::write(&boogie_file_path, boogie_str).unwrap();
    if let Ok(boogie_path) = env::var("BOOGIE_EXE") {
        if let Ok(z3_path) = env::var("Z3_EXE") {
            let status = Command::new(boogie_path)
                .args(&[
                    &format!("{}{}", "-z3exe:", z3_path).as_str(),
                    "-doModSetAnalysis",
                    "-noinfer",
                    "-noVerify",
                    boogie_file_path.to_str().unwrap(),
                ])
                .status()
                .expect("failed to execute Boogie");
            assert!(status.success());
        }
    }
}

/// Verifies or updates golden file for the given generated boogie source.
///
/// This re-generates the source based on the module targeted by this test.
fn verify_or_update_golden(
    modules: &[VerifiedModule],
    source_maps: &[ModuleSourceMap<Loc>],
    file_name: &str,
    target_modules: &[&str],
) {
    // Translate targeted to the given set of modules, if not empty.
    let mut ts = BoogieTranslator::new(&modules, &source_maps);
    if !target_modules.is_empty() {
        ts = ts.set_target_modules(target_modules);
    }
    let source = ts.translate();

    // The mint created here will automatically verify/update the
    // file once it is dropped.
    let mut mint = goldenfile::Mint::new("tests/goldenfiles");
    let goldenfile_name = format!(
        "{}.bpl",
        Path::new(file_name).file_stem().unwrap().to_str().unwrap()
    );
    let mut goldenfile = mint
        .new_goldenfile_with_differ(goldenfile_name, Box::new(file_diff))
        .unwrap();
    write!(goldenfile, "{}", clean_for_golden(&source)).unwrap();
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
