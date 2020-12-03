// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use goldenfile::Mint;
use std::{cell::RefCell, fmt::Debug, fs::File, io::Write, path::PathBuf};

pub const GOLDEN_DIR_PATH: &str = "goldens";
pub const EXT_NAME: &str = "exp";

pub(crate) struct GoldenOutputs {
    #[allow(dead_code)]
    mint: Mint,
    file: RefCell<File>,
}

fn golden_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(GOLDEN_DIR_PATH.to_string());
    path
}

impl GoldenOutputs {
    pub fn new(name: &str) -> Self {
        let mut mint = Mint::new(golden_path());
        let mut file_path = PathBuf::new();
        file_path.push(name);
        let file = RefCell::new(
            mint.new_goldenfile(file_path.with_extension(EXT_NAME))
                .unwrap(),
        );
        Self { file, mint }
    }

    pub fn log(&self, msg: &str) {
        self.file.borrow_mut().write_all(msg.as_bytes()).unwrap();
    }
}

impl Debug for GoldenOutputs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}
