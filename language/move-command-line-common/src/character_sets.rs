// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

/// Determine if a character is an allowed eye-visible (printable) character.
///
/// The only allowed printable characters are the printable ascii characters (SPACE through ~) and
/// tabs. All other characters are invalid and we return false.
pub fn is_permitted_printable_char(c: char) -> bool {
    let x = c as u32;
    let is_above_space = x >= 0x20; // Don't allow meta characters
    let is_below_tilde = x <= 0x7E; // Don't allow DEL meta character
    let is_tab = x == 0x09; // Allow tabs
    (is_above_space && is_below_tilde) || is_tab
}

/// Determine if a character is a permitted newline character.
///
/// The only permitted newline character is \n. All others are invalid.
pub fn is_permitted_newline_char(c: char) -> bool {
    let x = c as u32;
    x == 0x0A
}

/// Determine if a character is permitted character.
///
/// A permitted character is either a permitted printable character, or a permitted
/// newline. Any other characters are disallowed from appearing in the file.
pub fn is_permitted_char(c: char) -> bool {
    is_permitted_printable_char(c) || is_permitted_newline_char(c)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_permitted_characters() {
        let mut good_chars = (0x20..=0x7E).collect::<Vec<u8>>();
        good_chars.push(0x0A); // \n
        good_chars.push(0x09); // \t
        for c in good_chars {
            assert!(super::is_permitted_char(c as char));
        }
    }

    #[test]
    fn test_forbidden_characters() {
        let mut bad_chars = (0x0..0x09).collect::<Vec<u8>>();
        bad_chars.append(&mut (0x0B..=0x1F).collect::<Vec<u8>>());
        bad_chars.push(0x7F);
        for c in bad_chars {
            assert!(!super::is_permitted_char(c as char));
        }
    }
}
