// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow;
use move_testing_base::Task;
use std::env;
use std::fmt::{self, Write as FmtWrite};
use std::fs::read_to_string;
use std::fs::File;
use std::io::{self, Write as IOWrite};
use std::iter;
use std::path::Path;
use std::path::PathBuf;
// use termcolor::{Buffer, BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};
use difference::Changeset;
use yaml_rust::YamlLoader;

use crate::genesis::PRECOMPILED_STD_LIB;
use crate::parser::parse_move_testing_flow;
use crate::types::{GenesisOption, MoveTestingEnv};

pub const UPDATE_OUTPUTS: &str = "UPDATE_OUTPUTS";
pub const CREATE_OUTPUTS: &str = "CREATE_OUTPUTS";

fn env_var(var: &str) -> String {
    env::var(var)
        .unwrap_or_else(|_| "".to_string())
        .to_ascii_lowercase()
}

fn should_update_outputs() -> bool {
    let update = env_var(UPDATE_OUTPUTS);
    update == "1" || update == "true"
}

fn should_create_outputs() -> bool {
    let create = env_var(CREATE_OUTPUTS);
    create == "1" || create == "true"
}

fn write_horizontal_line(output: &mut impl FmtWrite, term_width: usize) -> fmt::Result {
    writeln!(
        output,
        "{}",
        iter::repeat('=').take(term_width).collect::<String>()
    )
}

enum Error {
    OutputMismatch(Changeset),
    ExpFileNotFound(String, String),
    Other(anyhow::Error),
}

impl<E> From<E> for Error
where
    anyhow::Error: From<E>,
{
    fn from(err: E) -> Self {
        Error::Other(err.into())
    }
}

fn run_move_expected_output_test_impl(test_file_path: &Path) -> Result<(), Error> {
    let text = read_to_string(test_file_path)?;

    let yaml = YamlLoader::load_from_str(&text)?;
    let flow = parse_move_testing_flow(&yaml)?;

    let mut env = MoveTestingEnv::new();

    match flow.config.genesis {
        GenesisOption::Std => {
            for (module_id, blob) in &*PRECOMPILED_STD_LIB {
                env.storage
                    .publish_or_overwrite_module(module_id.clone(), blob.clone());
            }
        }
        GenesisOption::None => (),
    }

    for (id, task) in flow.tasks.into_iter().enumerate() {
        if id > 0 {
            writeln!(env.output, "")?;
            writeln!(env.output, "")?;
        }
        writeln!(env.output, "Task {}", id)?;
        env = task.run(env)?;
    }

    let mut exp_file_path = PathBuf::from(test_file_path);
    exp_file_path.set_extension("exp");

    match read_to_string(&exp_file_path) {
        Ok(expected) => {
            if should_update_outputs() {
                let mut f = File::create(&exp_file_path)?;
                f.write_all(env.output.as_bytes())?;
            } else {
                let diff = Changeset::new(&expected, &env.output, "\n");
                if diff.distance != 0 {
                    return Err(Error::OutputMismatch(diff));
                }
            }
        }
        Err(err) => match err.kind() {
            io::ErrorKind::NotFound => {
                if should_create_outputs() {
                    let mut f = File::create(&exp_file_path)?;
                    f.write_all(env.output.as_bytes())?;
                } else {
                    return Err(Error::ExpFileNotFound(
                        exp_file_path.to_string_lossy().to_string(),
                        env.output,
                    ));
                }
            }
            _ => return Err(Error::Other(err.into())),
        },
    }

    Ok(())
}

pub fn indent(text: &str, spaces: usize) -> String {
    let mut buffer = String::new();
    for line in text.lines() {
        for _i in 0..spaces {
            buffer.push(' ');
        }
        buffer.push_str(line);
        writeln!(buffer, "").unwrap();
    }
    buffer
}

pub fn run_move_expected_output_test(test_file_path: &Path) -> Option<String> {
    match run_move_expected_output_test_impl(test_file_path) {
        Ok(_) => None,
        Err(err) => {
            let mut buffer = String::new();

            let term_width = match term_size::dimensions() {
                Some((w, _h)) => w,
                _ => 80,
            };

            write_horizontal_line(&mut buffer, term_width).unwrap();
            writeln!(buffer, "{}", test_file_path.to_string_lossy()).unwrap();
            writeln!(buffer).unwrap();

            match err {
                Error::ExpFileNotFound(exp_file_path, output) => {
                    writeln!(buffer, "    Failed to open expected output file:").unwrap();
                    writeln!(buffer, "        {}", exp_file_path).unwrap();
                    writeln!(buffer).unwrap();

                    writeln!(buffer, "    Output:").unwrap();
                    writeln!(buffer, "{}", indent(&format!("{}", output), 8)).unwrap();
                    writeln!(buffer).unwrap();

                    writeln!(buffer, "    Hint: run with environment variable {}=1 to create output files for new tests.", CREATE_OUTPUTS).unwrap();
                }
                Error::OutputMismatch(diff) => {
                    writeln!(buffer, "    Output mismatch:").unwrap();
                    writeln!(buffer, "{}", indent(&format!("{}", diff), 8)).unwrap();
                    writeln!(buffer).unwrap();
                    writeln!(buffer, "    Hint: run with environment variable {}=1 to update output files for new tests.", UPDATE_OUTPUTS).unwrap();
                }
                Error::Other(err) => {
                    writeln!(buffer, "    Unexpected error: {}", err).unwrap();
                    writeln!(buffer).unwrap();
                    writeln!(buffer, "    Check your test configuration or report if you think this is caused by a bug.").unwrap();
                }
            }

            writeln!(buffer).unwrap();
            write_horizontal_line(&mut buffer, term_width).unwrap();
            writeln!(buffer).unwrap();

            Some(buffer)
        }
    }
}
