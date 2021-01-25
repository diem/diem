// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

pub mod text_builder;
pub mod tui_interface;

use crate::tui::tui_interface::TUIInterface;
use std::{
    error::Error,
    io::{self, Stdin, Write},
};
use termion::{
    event::Key,
    input::{Keys, TermRead},
};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    style::Style,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct TUI<Interface: TUIInterface> {
    current_line_number: u16,
    current_column: u16,
    interface: Interface,
    keys: Keys<Stdin>,
}

impl<Interface: TUIInterface> TUI<Interface> {
    pub fn new(interface: Interface) -> Self {
        let keys = io::stdin().keys();
        Self {
            current_line_number: 0,
            current_column: 0,
            interface,
            keys,
        }
    }

    pub fn redraw<W: Write>(&mut self, f: &mut Frame<TermionBackend<W>>) {
        // Create a split window, each pane using 50% of the screen
        let window = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(f.size());

        // Get the bottom offset of the window that the cursor will be displayed
        let window_size = window[0].bottom();
        // The window will need to be scrolled if the cursor is more than halfway down the screen.
        let scroll = if self.current_line_number > window_size / 2 {
            self.current_line_number
                .checked_sub(window_size / 2)
                .unwrap()
        } else {
            0
        };

        // Get the output for the current line/column of the cursore
        let current_interface = self
            .interface
            .on_redraw(self.current_line_number, self.current_column);

        // Left window logic
        // Create a paragraph for our text, and scroll it if need be (by the amount computed above)
        let input = Paragraph::new(current_interface.left_screen)
            .style(Style::default())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(Interface::LEFT_TITLE),
            )
            .scroll((scroll, 0));

        // Set the cursor position in the left window. Numbers incremented by 1 since the screen
        // border is at position 0 in both x and y coordinates. If we scrolled the text, we need
        // to subtract that from the line number so that the cursor is over the correct line.
        f.set_cursor(
            self.current_column.checked_add(1).unwrap(),
            self.current_line_number
                .checked_add(1)
                .unwrap()
                .checked_sub(scroll)
                .unwrap(),
        );
        f.render_widget(input, window[0]);

        // Right window logic
        // Render the output text. No other logic needed.
        let output = Paragraph::new(current_interface.right_screen)
            .style(Style::default())
            .block(
                Block::default()
                    .title(Interface::RIGHT_TITLE)
                    .borders(Borders::ALL),
            );
        f.render_widget(output, window[1])
    }

    /// Handles keyboard input, and updates state according to those key presses.
    /// Down, Up => move the cursor up or down a line
    /// Left, Right => move the cursor to the previous (resp. next) character on the current line
    /// ESC, q => exit
    pub fn handle_input(&mut self) -> Result<bool, Box<dyn Error>> {
        if let Some(key) = self.keys.next() {
            match key.unwrap() {
                // Exit
                Key::Esc | Key::Char('q') => {
                    return Ok(true);
                }
                // Update current line number to move the cursor up a line. If the column number is
                // past the end of the new line we're on, bound the column number so it isn't past
                // the last character on the new line.
                Key::Up => {
                    self.current_line_number = self.current_line_number.saturating_sub(1);
                    self.current_column = self
                        .interface
                        .bound_column(self.current_line_number, self.current_column);
                }
                // Update current line number to move the cursor down a line. If the column number is
                // past the end of the new line we're on, bound the column number so it isn't past
                // the last character on the new line.
                Key::Down => {
                    self.current_line_number = self
                        .interface
                        .bound_line(self.current_line_number.checked_add(1).unwrap());
                    self.current_column = self
                        .interface
                        .bound_column(self.current_line_number, self.current_column);
                }
                // Move the cursor to the next character on the current line. Number is bounded by
                // the length of the current line.
                Key::Right => {
                    self.current_column = self.interface.bound_column(
                        self.current_line_number,
                        self.current_column.checked_add(1).unwrap(),
                    );
                }
                // Move the cursor to the previous character on the current line. Number is bounded by
                // the length of the current line.
                Key::Left => {
                    self.current_column = self.current_column.saturating_sub(1);
                }
                _ => {}
            }
        }
        Ok(false)
    }
}
