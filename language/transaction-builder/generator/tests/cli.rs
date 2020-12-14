// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::{io::Write, process::Command};
use tempfile::tempdir;

const EXPECTED_OUTPUT : &str = "224 1 161 28 235 11 1 0 0 0 7 1 0 2 2 2 4 3 6 16 4 22 2 5 24 29 7 53 96 8 149 1 16 0 0 0 1 1 0 0 2 0 1 0 0 3 2 3 1 1 0 4 1 3 0 1 5 1 6 12 1 8 0 5 6 8 0 5 3 10 2 10 2 0 5 6 12 5 3 10 2 10 2 1 9 0 11 68 105 101 109 65 99 99 111 117 110 116 18 87 105 116 104 100 114 97 119 67 97 112 97 98 105 108 105 116 121 27 101 120 116 114 97 99 116 95 119 105 116 104 100 114 97 119 95 99 97 112 97 98 105 108 105 116 121 8 112 97 121 95 102 114 111 109 27 114 101 115 116 111 114 101 95 119 105 116 104 100 114 97 119 95 99 97 112 97 98 105 108 105 116 121 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 1 1 1 4 1 12 11 0 17 0 12 5 14 5 10 1 10 2 11 3 11 4 56 0 11 5 17 2 2 1 7 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 1 3 88 68 88 3 88 68 88 0 4 3 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 1 135 214 18 0 0 0 0 0 4 0 4 0 \n";

#[test]
#[ignore]
fn test_examples_in_readme() -> std::io::Result<()> {
    let file = std::io::BufReader::new(std::fs::File::open("README.md")?);
    let quotes = get_bash_quotes(file)?;
    // Check that we have the expected number of examples starting with "```bash".
    assert_eq!(quotes.len(), 9);

    let mut quotes = quotes.into_iter();

    for _ in 0..4 {
        let dir = tempdir().unwrap();
        let mut test_script = std::fs::File::create(dir.path().join("test.sh"))?;
        write!(&mut test_script, "{}", quotes.next().unwrap())?;
        write!(&mut test_script, "{}", quotes.next().unwrap())?;
        let output = Command::new("bash")
            .current_dir("../../..") // root of Diem
            .env("DEST", dir.path())
            .arg("-e")
            .arg("-x")
            .arg(dir.path().join("test.sh"))
            .output()?;
        eprintln!("{}", std::str::from_utf8(&output.stderr).unwrap());
        assert!(output.status.success());
        assert_eq!(
            std::str::from_utf8(&output.stdout).unwrap(),
            EXPECTED_OUTPUT
        );
    }

    // Last quote (Rust) is yet incomplete.
    let dir = tempdir().unwrap();
    let mut test_script = std::fs::File::create(dir.path().join("test.sh"))?;
    write!(&mut test_script, "{}", quotes.next().unwrap())?;
    let output = Command::new("bash")
        .current_dir("../../..") // root of Diem
        .env("DEST", dir.path())
        .arg("-e")
        .arg("-x")
        .arg(dir.path().join("test.sh"))
        .output()?;
    eprintln!("{}", std::str::from_utf8(&output.stderr).unwrap());
    assert!(output.status.success());
    Ok(())
}

#[allow(clippy::while_let_on_iterator)]
fn get_bash_quotes<R>(reader: R) -> std::io::Result<Vec<String>>
where
    R: std::io::BufRead,
{
    let mut result = Vec::new();
    let mut lines = reader.lines();

    while let Some(line) = lines.next() {
        let line = line?;
        if line.starts_with("```bash") {
            let mut quote = String::new();
            while let Some(line) = lines.next() {
                let line = line?;
                if line.starts_with("```") {
                    break;
                }
                quote += &line;
                quote += "\n";
            }
            result.push(quote);
        }
    }

    Ok(result)
}
