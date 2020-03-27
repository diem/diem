// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! A helper to emit code. Supports indentation and maintains source to target location information.

use std::collections::{BTreeMap, Bound};

use codespan::{ByteIndex, ColumnIndex, Files, LineIndex};

use spec_lang::env::Loc;
use std::cell::RefCell;

struct CodeWriterData {
    /// The generated output string.
    output: String,

    /// Current active indentation.
    indent: usize,

    /// Current active location.
    current_location: Loc,

    /// A sparse mapping from byte index in written output to location in source file.
    /// Any index not in this map is approximated by the next smaller index on lookup.
    output_location_map: BTreeMap<ByteIndex, Loc>,
}

pub struct CodeWriter(RefCell<CodeWriterData>);

impl CodeWriter {
    /// Creates new code writer, with the given default location.
    pub fn new(loc: Loc) -> CodeWriter {
        let zero = ByteIndex(0);
        let mut output_location_map = BTreeMap::new();
        output_location_map.insert(zero, loc.clone());
        Self(RefCell::new(CodeWriterData {
            output: String::new(),
            indent: 0,
            current_location: loc,
            output_location_map,
        }))
    }

    /// Calls a function to process the code written so far. This is embedded into a function
    /// so we ensure correct scoping of borrowed RefCell content.
    pub fn process_result<T, F: FnMut(&str) -> T>(&self, mut f: F) -> T {
        f(&self.0.borrow().output)
    }

    /// Sets the current location. This location will be associated with all subsequently written
    /// code so we can map back from the generated code to this location. If current loc
    /// is already the passed one, nothing will be updated, so it is ok to call this method
    /// repeatedly with the same value.
    pub fn set_location(&self, loc: &Loc) {
        let mut data = self.0.borrow_mut();
        let code_at = ByteIndex(data.output.len() as u32);
        if &data.current_location != loc {
            data.output_location_map.insert(code_at, loc.clone());
            data.current_location = loc.clone();
        }
    }

    /// Given a byte index in the written output, return the best approximation of the source
    /// which generated this output.
    pub fn get_source_location(&self, output_index: ByteIndex) -> Option<Loc> {
        let data = self.0.borrow();
        if let Some(loc) = data
            .output_location_map
            .range((Bound::Unbounded, Bound::Included(&output_index)))
            .next_back()
            .map(|(_, v)| v)
        {
            return Some(loc.clone());
        }
        None
    }

    /// Given line/column location, determine ByteIndex of that location.
    pub fn get_output_byte_index(&self, line: LineIndex, column: ColumnIndex) -> Option<ByteIndex> {
        self.process_result(|s| {
            let mut fmap = Files::new();
            let id = fmap.add("dummy", s);
            fmap.line_span(id, line).ok().map(|line_span| {
                ByteIndex((line_span.start().to_usize() + column.to_usize()) as u32)
            })
        })
    }

    /// Indents any subsequently written output. The current line of output and any subsequent ones
    /// will be indented. Note this works after the last output was `\n` but the line is still
    /// empty.
    pub fn indent(&self) {
        let mut data = self.0.borrow_mut();
        data.indent += 4;
    }

    /// Undo previously done indentation.
    pub fn unindent(&self) {
        let mut data = self.0.borrow_mut();
        assert!(data.indent >= 4);
        data.indent -= 4;
    }

    /// Emit a string. The string will be broken down into lines to apply current indentation.
    pub fn emit(&self, s: &str) {
        let mut first = true;
        // str::lines ignores trailing newline, so deal with this ad-hoc
        let end_newl = s.ends_with('\n');
        for l in s.lines() {
            if first {
                first = false
            } else {
                self.0.borrow_mut().output.push_str("\n");
            }
            self.emit_str(l)
        }
        if end_newl {
            self.0.borrow_mut().output.push_str("\n");
        }
    }

    /// Emits a string and then terminates the line.
    pub fn emit_line(&self, s: &str) {
        self.emit(s);
        self.emit("\n");
    }

    /// Helper for emitting a string for a single line.
    fn emit_str(&self, s: &str) {
        let mut data = self.0.borrow_mut();
        // If we are looking at the beginning of a new line, emit indent now.
        if data.indent > 0 && (data.output.is_empty() || data.output.ends_with('\n')) {
            let n = data.indent;
            data.output.push_str(&" ".repeat(n));
        }
        data.output.push_str(s);
    }
}

/// Macro to emit a simple or formatted string.
macro_rules! emit {
    ($target:expr, $s:expr) => (
       $target.emit($s)
    );
    ($target:expr, $s:expr, $($arg:expr),+ $(,)?) => (
       $target.emit(&format!($s, $($arg),+))
    )
}

/// Macro to emit a simple or formatted string followed by a new line.
macro_rules! emitln {
    ($target:expr) => (
       $target.emit_line("")
    );
    ($target:expr, $s:expr) => (
       $target.emit_line($s)
    );
    ($target:expr, $s:expr, $($arg:expr),+ $(,)?) => (
       $target.emit_line(&format!($s, $($arg),+))
    )
}
