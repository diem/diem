// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{errors::*, parser::syntax::make_loc};
use move_ir_types::location::*;

pub fn decode(loc: Loc, s: &str) -> Result<Vec<u8>, Errors> {
    let mut text = s.to_string();
    let adjust = if text.len() % 2 != 0 {
        text.insert(0, '0');
        1
    } else {
        0
    };
    match hex::decode(&text) {
        Ok(vec) => Ok(vec),
        Err(hex::FromHexError::InvalidHexCharacter { c, index }) => {
            let filename = loc.file();
            let start_offset = loc.span().start().0 as usize;
            let offset = start_offset + 2 - adjust + index;
            let loc = make_loc(filename, offset, offset);
            Err(vec![vec![(
                loc,
                format!("Invalid hexadecimal character: '{}'", c),
            )]])
        }
        Err(_) => unreachable!("unexpected error parsing hex byte string value"),
    }
}
