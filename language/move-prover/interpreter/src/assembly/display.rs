// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::{fmt, iter};

const INDENT_PER_LEVEL: usize = 2;

pub struct AstWriter {
    level: usize,
    lines: Vec<String>,
}

impl AstWriter {
    pub fn new() -> Self {
        Self {
            level: 0,
            lines: vec![String::new()],
        }
    }

    fn cur(&mut self) -> &mut String {
        self.lines.last_mut().unwrap()
    }

    fn indent<F: FnOnce(&mut AstWriter)>(&mut self, f: F) {
        self.new_line();
        self.level += 1;
        f(self);
        self.level -= 1;
        self.new_line();
    }

    pub fn new_line(&mut self) {
        self.lines.push(String::new());
    }

    pub fn write(&mut self, s: &str) {
        let margin = self.level * INDENT_PER_LEVEL;
        let cur = self.cur();
        if cur.is_empty() {
            (0..margin).for_each(|_| cur.push(' '));
        }
        cur.push_str(s);
    }

    pub fn writeln(&mut self, s: &str) {
        self.write(s);
        self.new_line();
    }

    pub fn block<F: FnOnce(&mut AstWriter)>(&mut self, f: F) {
        self.write("{");
        self.indent(f);
        self.write("}");
    }

    #[allow(dead_code)]
    pub fn join<T, F: FnMut(&mut AstWriter, T) -> bool>(
        &mut self,
        items: impl iter::IntoIterator<Item = T> + iter::ExactSizeIterator,
        sep: &str,
        mut f: F,
    ) {
        let len = items.len();
        for (idx, item) in items.into_iter().enumerate() {
            let needs_newline = f(self, item);
            if idx + 1 != len {
                self.write(sep);
                if needs_newline {
                    self.new_line();
                }
            }
        }
    }

    pub fn numbered_list<T, F: FnMut(&mut AstWriter, String, T)>(
        &mut self,
        items: impl iter::IntoIterator<Item = T> + iter::ExactSizeIterator,
        mut f: F,
    ) {
        let len = items.len();
        let num_digits = format!("{}", len).len();
        for (idx, item) in items.into_iter().enumerate() {
            let num = format!("{:>width$}!", idx, width = num_digits);
            f(self, num, item);
        }
    }
}

impl fmt::Display for AstWriter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in &self.lines {
            writeln!(f, "{}", line)?;
        }
        Ok(())
    }
}

pub trait AstDebug {
    fn ast_debug(&self, w: &mut AstWriter);

    fn print_ast(&self) -> String {
        let mut writer = AstWriter::new();
        self.ast_debug(&mut writer);
        format!("{}", writer)
    }
}
