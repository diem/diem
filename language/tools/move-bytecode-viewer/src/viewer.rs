// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]
use crate::{
    interfaces::{LeftScreen, RightScreen},
    tui::{
        text_builder::TextBuilder,
        tui_interface::{TUIInterface, TUIOutput},
    },
};
use tui::{
    style::{Color, Style},
    text::Spans,
};

#[derive(Debug, Clone)]
pub struct Viewer<BytecodeViewer: LeftScreen, SourceViewer: RightScreen<BytecodeViewer>> {
    bytecode_text: Vec<String>,
    source_viewer: SourceViewer,
    bytecode_viewer: BytecodeViewer,
}

impl<BytecodeViewer: LeftScreen, SourceViewer: RightScreen<BytecodeViewer>>
    Viewer<BytecodeViewer, SourceViewer>
{
    pub fn new(source_viewer: SourceViewer, bytecode_viewer: BytecodeViewer) -> Self {
        Self {
            bytecode_text: bytecode_viewer
                .backing_string()
                .split('\n')
                .map(|x| x.to_string())
                .collect(),
            source_viewer,
            bytecode_viewer,
        }
    }
}

impl<BytecodeViewer: LeftScreen, SourceViewer: RightScreen<BytecodeViewer>> TUIInterface
    for Viewer<BytecodeViewer, SourceViewer>
{
    const LEFT_TITLE: &'static str = "Bytecode";
    const RIGHT_TITLE: &'static str = "Source Code";

    fn on_redraw(&mut self, line_number: u16, column_number: u16) -> TUIOutput {
        // Highlight style
        let style: Style = Style::default().bg(Color::Red);
        let report = match self
            .bytecode_viewer
            .get_source_index_for_line(line_number as usize, column_number as usize)
        {
            None => {
                let mut builder = TextBuilder::new();
                builder.add(self.source_viewer.backing_string(), Style::default());
                builder.finish()
            }
            Some(info) => {
                let source_context = self.source_viewer.source_for_code_location(info).unwrap();

                let mut builder = TextBuilder::new();
                builder.add(source_context.left, Style::default());
                builder.add(source_context.highlight, style);
                builder.add(source_context.remainder, Style::default());
                builder.finish()
            }
        };

        TUIOutput {
            left_screen: self
                .bytecode_text
                .iter()
                .map(|x| Spans::from(x.clone()))
                .collect(),
            right_screen: report,
        }
    }

    fn bound_line(&self, line_number: u16) -> u16 {
        std::cmp::min(
            line_number,
            self.bytecode_text.len().checked_sub(1).unwrap() as u16,
        )
    }

    fn bound_column(&self, line_number: u16, column_number: u16) -> u16 {
        std::cmp::min(
            column_number,
            self.bytecode_text[line_number as usize].len() as u16,
        )
    }
}
