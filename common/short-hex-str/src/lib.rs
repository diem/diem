// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use mirai_annotations::debug_checked_precondition;
use serde::{Serialize, Serializer};
use std::{fmt, str};
use thiserror::Error;

/// An efficient container for formatting a byte slice as a hex-formatted string,
/// stored on the stack.
///
/// Using `ShortHexStr` instead of `hex::encode` is about 3-4x faster on a recent
/// MBP 2019 (~48 ns/iter vs ~170 ns/iter) in an artifical micro benchmark.
#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Clone, Copy)]
pub struct ShortHexStr([u8; ShortHexStr::LENGTH]);

#[derive(Error, Debug)]
#[error("Input bytes are too short")]
pub struct InputTooShortError;

impl ShortHexStr {
    pub const SOURCE_LENGTH: usize = 4;
    pub const LENGTH: usize = 2 * ShortHexStr::SOURCE_LENGTH;

    /// Format a new `ShortHexStr` from a byte slice.
    ///
    /// Returns `Err(InputTooShortError)` if the input byte slice length is less
    /// than `SOURCE_LENGTH` bytes.
    pub fn try_from_bytes(src_bytes: &[u8]) -> Result<ShortHexStr, InputTooShortError> {
        if src_bytes.len() >= ShortHexStr::SOURCE_LENGTH {
            let src_short_bytes = &src_bytes[0..ShortHexStr::SOURCE_LENGTH];
            let mut dest_bytes = [0u8; ShortHexStr::LENGTH];

            // We include a tiny hex encode here instead of using the `hex` crate's
            // `encode_to_slice`, since the compiler seems unable to inline across
            // the crate boundary.
            hex_encode(&src_short_bytes, &mut dest_bytes);
            Ok(Self(dest_bytes))
        } else {
            Err(InputTooShortError)
        }
    }

    pub fn as_str(&self) -> &str {
        // We could also do str::from_utf8_unchecked here to avoid the unnecessary
        // runtime check. Shaves ~6-7 ns/iter in a micro bench but the unsafe is
        // probably not worth the hassle.
        str::from_utf8(&self.0).expect(
            "This can never fail since &self.0 will only ever contain the \
             following characters: '0123456789abcdef', which are all valid \
             ASCII characters and therefore all valid UTF-8",
        )
    }
}

impl fmt::Debug for ShortHexStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Display for ShortHexStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for ShortHexStr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

/// Maps a nibble to its corresponding hex-formatted ASCII character.
const HEX_CHARS_LOWER: &[u8; 16] = b"0123456789abcdef";

/// Format a byte as hex. Returns a tuple containing the first character and then
/// the second character as ASCII bytes.
#[inline(always)]
fn byte2hex(byte: u8) -> (u8, u8) {
    let hi = HEX_CHARS_LOWER[((byte >> 4) & 0x0f) as usize];
    let lo = HEX_CHARS_LOWER[(byte & 0x0f) as usize];
    (hi, lo)
}

/// Hex encode a byte slice into the destination byte slice.
#[inline(always)]
fn hex_encode(src: &[u8], dst: &mut [u8]) {
    debug_checked_precondition!(dst.len() == 2 * src.len());

    for (byte, out) in src.iter().zip(dst.chunks_mut(2)) {
        let (hi, lo) = byte2hex(*byte);
        out[0] = hi;
        out[1] = lo;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use proptest::prelude::*;
    use std::{str, u8};

    #[test]
    fn test_hex_encode() {
        let src = [0x12_u8, 0x34, 0xfe, 0xba];
        let mut actual = [0u8; 8];
        hex_encode(&src, &mut actual);
        let expected = b"1234feba";
        assert_eq!(&actual, expected);
    }

    #[test]
    fn test_byte2hex_equivalence() {
        for byte in 0..=u8::MAX {
            let (hi, lo) = byte2hex(byte);
            let formatted_bytes = [hi, lo];
            let actual = str::from_utf8(&formatted_bytes[..]).unwrap();
            let expected = hex::encode(&[byte][..]);
            assert_eq!(actual, expected.as_str());
        }
    }

    proptest! {
        #[test]
        fn test_address_short_str_equivalence(addr in any::<[u8; 16]>()) {
            let short_str_old = hex::encode(&addr[0..ShortHexStr::SOURCE_LENGTH]);
            let short_str_new = ShortHexStr::try_from_bytes(&addr).unwrap();
            prop_assert_eq!(short_str_old.as_str(), short_str_new.as_str());
        }
    }
}
