// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{errors::*, parser::syntax::make_loc};

pub fn decode_byte_string(
    filename: &'static str,
    start_offset: usize,
    text: &str,
) -> Result<Vec<u8>, Error> {
    let mut coder = ByteStringCoder::new(filename, start_offset);
    for (idx, c) in text.chars().enumerate() {
        coder.next_char(idx, c)?;
    }
    coder.finish(text.len())
}

/// Escape sequence.
enum Sequence {
    /// \x00
    Hex(String),
    /// The start of an escape sequence. \
    Generic,
}

#[derive(Default)]
struct ByteStringCoder {
    filename: &'static str,
    start_offset: usize,
    buffer: Vec<u8>,
    escape_sequence: Option<Sequence>,
}

impl ByteStringCoder {
    fn new(filename: &'static str, start_offset: usize) -> ByteStringCoder {
        ByteStringCoder {
            filename,
            start_offset,
            buffer: vec![],
            escape_sequence: None,
        }
    }

    fn push(&mut self, ch: char) {
        let mut buffer = vec![0; ch.len_utf8()];
        ch.encode_utf8(&mut buffer);
        self.buffer.extend(buffer);
    }

    fn err(&self, start: usize, end: usize, err_text: String) -> Result<(), Error> {
        Err(vec![(
            make_loc(
                self.filename,
                self.start_offset + start,
                self.start_offset + end,
            ),
            err_text,
        )])
    }

    fn next_char(&mut self, idx: usize, ch: char) -> Result<(), Error> {
        if let Some(escape_sequence) = &mut self.escape_sequence {
            match escape_sequence {
                Sequence::Generic => {
                    self.escape_sequence = None;
                    match ch {
                        'n' => self.push('\n'),
                        'r' => self.push('\r'),
                        't' => self.push('\t'),
                        '\\' => self.push('\\'),
                        '0' => self.push('\0'),
                        '"' => self.push('"'),
                        'x' => self.escape_sequence = Some(Sequence::Hex(String::new())),
                        _ => {
                            return self.err(
                                idx - 1,
                                idx + 1,
                                format!("Invalid escape sequence: '\\{}'", ch),
                            );
                        }
                    }
                }
                Sequence::Hex(hex) => {
                    hex.push(ch);
                    if hex.len() == 2 {
                        match hex::decode(&hex) {
                            Ok(buffer) => self.buffer.extend(buffer),
                            Err(hex::FromHexError::InvalidHexCharacter { c, index }) => {
                                let err_text = format!("Invalid hexadecimal character: '{}'", c);

                                return self.err(idx - 1 + index, idx - 1 + index, err_text);
                            }
                            Err(_) => {
                                unreachable!("unexpected error parsing hex byte string value")
                            }
                        }
                        self.escape_sequence = None;
                    }
                }
            }
        } else if ch == '\\' {
            self.escape_sequence = Some(Sequence::Generic);
        } else if ch.is_ascii() {
            self.push(ch)
        } else {
            self.err(
                idx,
                idx,
                format!(
                    "Invalid character ‘{}’. Only ascii characters are supported.",
                    ch
                ),
            )?;
        }

        Ok(())
    }

    fn finish(self, text_len: usize) -> Result<Vec<u8>, Error> {
        // Handles incomplete escape sequences at the end of the string.
        if let Some(escape_sequence) = &self.escape_sequence {
            match escape_sequence {
                Sequence::Hex(h) => {
                    let err_text =
                        format!("Invalid escape: '\\x{}'. Single HEX literal must be represented by two symbols. [\\x00-\\xFF].", h);
                    self.err(text_len - h.len() - 2, text_len, err_text)?;
                }
                Sequence::Generic => {
                    // no-op
                }
            }
        }

        Ok(self.buffer)
    }
}
