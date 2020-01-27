// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

/// Simple trait used for pretty printing the various AST
///
/// Unfortunately, the trait implementation cannot be derived. The actual implementation should
/// closely resemble the source syntax. As suchfield does not get printed in a direct manner, and
/// most of the logic is ad hoc
///
/// To avoid missing fields in the printing, be sure to fully pattern match against the struct
/// (without the use of `..`) when implementing `AstDebug`. For example,
///
/// ```rust,ignore
/// impl AstDebug for StructDefinition {
///     fn ast_debug(&self, w: &mut AstWriter) {
///         let StructDefinition {
///             resource_opt,
///             name,
///             type_parameters,
///             fields,
///         } = self;
///         ...
///     }
/// }
/// ```
pub trait AstDebug {
    fn ast_debug(&self, w: &mut AstWriter);
}

impl<T: AstDebug> AstDebug for Box<T> {
    fn ast_debug(&self, w: &mut AstWriter) {
        self.as_ref().ast_debug(w)
    }
}

pub fn print<T: AstDebug>(t: &T) {
    let mut writer = AstWriter::normal();
    t.ast_debug(&mut writer);
    print!("{}", writer);
}

pub fn print_verbose<T: AstDebug>(t: &T) {
    let mut writer = AstWriter::verbose();
    t.ast_debug(&mut writer);
    print!("{}", writer);
}

pub struct AstWriter {
    verbose: bool,
    margin: usize,
    lines: Vec<String>,
}

impl AstWriter {
    fn new(verbose: bool) -> Self {
        Self {
            verbose,
            margin: 0,
            lines: vec![String::new()],
        }
    }

    fn normal() -> Self {
        Self::new(false)
    }

    fn verbose() -> Self {
        Self::new(true)
    }

    fn cur(&mut self) -> &mut String {
        self.lines.last_mut().unwrap()
    }

    pub fn new_line(&mut self) {
        self.lines.push(String::new());
    }

    pub fn write(&mut self, s: &str) {
        let margin = self.margin;
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

    pub fn indent<F: FnOnce(&mut AstWriter) -> ()>(&mut self, inc: usize, f: F) {
        self.new_line();
        self.margin += inc;
        f(self);
        self.margin -= inc;
        self.new_line();
    }

    pub fn block<F: FnOnce(&mut AstWriter) -> ()>(&mut self, f: F) {
        self.write(" {");
        self.indent(4, f);
        self.write("}");
    }

    pub fn annotate<F: FnOnce(&mut AstWriter) -> (), Annot: AstDebug>(
        &mut self,
        f: F,
        annot: &Annot,
    ) {
        if self.verbose {
            self.write("(");
        }
        f(self);
        if self.verbose {
            self.write(": ");
            annot.ast_debug(self);
            self.write(")");
        }
    }

    pub fn list<T, F: FnMut(&mut AstWriter, T) -> bool>(
        &mut self,
        items: impl std::iter::IntoIterator<Item = T>,
        sep: &str,
        mut f: F,
    ) {
        let iter = items.into_iter();
        let len = match iter.size_hint() {
            (lower, None) => {
                assert!(lower == 0);
                return;
            }
            (_, Some(len)) => len,
        };
        for (idx, item) in iter.enumerate() {
            let needs_newline = f(self, item);
            if idx + 1 != len {
                self.write(sep);
                if needs_newline {
                    self.new_line()
                }
            }
        }
    }

    pub fn comma<T, F: FnMut(&mut AstWriter, T) -> ()>(
        &mut self,
        items: impl std::iter::IntoIterator<Item = T>,
        mut f: F,
    ) {
        self.list(items, ", ", |w, item| {
            f(w, item);
            false
        })
    }

    pub fn semicolon<T, F: FnMut(&mut AstWriter, T) -> ()>(
        &mut self,
        items: impl std::iter::IntoIterator<Item = T>,
        mut f: F,
    ) {
        self.list(items, ";", |w, item| {
            f(w, item);
            true
        })
    }
}

impl std::fmt::Display for AstWriter {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for line in &self.lines {
            writeln!(f, "{}", line)?;
        }
        Ok(())
    }
}
