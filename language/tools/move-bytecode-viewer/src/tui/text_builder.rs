// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]
use tui::{
    style::Style,
    text::{Span, Spans},
};

/// A `TextBuilder` is used to build up a paragraph, where some parts of it may need to have
/// different styling, and where this styling may not conform to line boundaries.
#[derive(Debug, Clone, Default)]
pub struct TextBuilder<'a> {
    // A vec of "lines" each line is a vector of spans.
    // We use Vec<Span> here instead of Spans since otherwise we wouldn't be able to join lines in
    // the `add` function.
    chunks: Vec<Vec<Span<'a>>>,
}

impl<'a> TextBuilder<'a> {
    /// Create a new text builder
    pub fn new() -> Self {
        Self { chunks: Vec::new() }
    }

    /// Add `text` with the given `style`ing to the text builder. This functions tracks newlines in
    /// the text already recorded (in the `chunks` field), and will splice lines between the
    /// previous text and the new `text` being added. It respects the `style` of both the old text
    /// and the newly added text.
    pub fn add(&mut self, text: String, style: Style) {
        let chunk = |string: String| {
            string
                .split('\n')
                .map(|x| x.to_string())
                .map(|x| vec![Span::styled(x, style)])
                .collect::<Vec<Vec<Span>>>()
        };
        let last_chunk_ends_with_nl = self
            .chunks
            .last()
            .map(|last_span| {
                last_span
                    .last()
                    .map(|last_span| last_span.content.ends_with('\n'))
                    .unwrap_or(false)
            })
            .unwrap_or(true);

        if !last_chunk_ends_with_nl {
            let mut iter = text.splitn(2, '\n');
            iter.next().into_iter().for_each(|line_continuation| {
                self.chunks
                    .last_mut()
                    .unwrap()
                    .push(Span::styled(line_continuation.to_string(), style));
            });
            iter.next().into_iter().for_each(|remainder| {
                self.chunks.extend(chunk(remainder.to_string()).into_iter());
            });
        } else {
            self.chunks.extend(chunk(text))
        }
    }

    /// Return back the final Spans, each `Spans` represents a line in the paragraph.
    pub fn finish(self) -> Vec<Spans<'a>> {
        self.chunks.into_iter().map(Spans::from).collect()
    }
}
