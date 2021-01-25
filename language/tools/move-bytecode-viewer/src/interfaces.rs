// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;

/// There are two interfaces--the `LeftScreen` and `RightScreen`--that need to be implemented for
/// the bytecode viewer and these can be mix-and-matched for different implementations.
///
/// The `LeftScreen` is the text that is displayed in the left-hand screen, and is scrollable. The
/// scrolling in this window will output `BytecodeInfo` objects. These objects are then consumed by
/// the `RightScreen` `source_for_code_location` function, which outputs text containing a context
/// (`left`, `remainder`) around the source code location identified by the passed in
/// `BytecodeInfo`.

#[derive(Clone, Debug)]
pub struct SourceContext {
    pub left: String,
    pub highlight: String,
    pub remainder: String,
}

/// The `LeftScreen` trait is used to index the code.
pub trait LeftScreen {
    /// This is the type used for indexing the source code in the `RightScreen` trait. e.g., for
    /// bytecode and using the source map, this would be a tuple of `(FunctionDefinitionIndex,
    /// CodeOffset)`.
    type SourceIndex;

    /// Given a `line` and the `column` within that line, this returns a `SourceIndex` to be used
    /// by the `RightScreen` trait.
    fn get_source_index_for_line(&self, line: usize, column: usize) -> Option<&Self::SourceIndex>;

    /// Return the backing string to be displayed on the left screen.
    fn backing_string(&self) -> String;
}

/// The `RightScreen` trait takes the indices output by the left screen (cursor movements that have
/// been possibly translated in some way, e.g., to (fdef_index, code_offset) pairs) and
/// translates these indices into a `SourceContext` view of the text held by the implementor of the
/// `RightScreen` trait.
pub trait RightScreen<Indexer: LeftScreen> {
    /// Take a `SourceIndex` from the `Indexer` and turn it into a context that will be diplayed on
    /// the right screen.
    fn source_for_code_location(
        &self,
        bytecode_info: &Indexer::SourceIndex,
    ) -> Result<SourceContext>;
    fn backing_string(&self) -> String;
}
