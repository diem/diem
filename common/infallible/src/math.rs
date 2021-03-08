// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

/// Utility macro for writing secure arithmetic operations in order to avoid
/// integer overflows.
///
/// # Examples
///
/// ```
///# use crate::infallible::checked;
/// let a: i64 = 1;
/// let b: i64 = 2;
/// let c: i64 = 3;
///
/// assert_eq!(checked!(a + b).unwrap(), 3);
/// assert_eq!(checked!(a + b + c).unwrap(), 6);
///
/// // When doing multiple different operations, it's important to use parentheses in order
/// // to guarantee the order of operation!
/// assert_eq!(checked!(a + ((b - c) * c)).unwrap(), -2);
///
/// // When using numeric literals, the compiler might not be able to infer the type properly,
/// // so if it complains, just add the type to the number.
/// assert_eq!(checked!(10_u32 / 2_u32).unwrap(), 5);
/// assert_eq!(checked!(10_u32 * 2_u32).unwrap(), 20);
/// assert_eq!(checked!(10_u32 - 2_u32).unwrap(), 8);
/// assert_eq!(checked!(10_u32 + 2_u32).unwrap(), 12);
///
/// // Casts using `as` operator must appear within parenthesis
/// assert_eq!(checked!(10_u32 + (2_u16 as u32)).unwrap(), 12);
///
/// assert_eq!(checked!(10_u32 / (1_u32 + 1_u32)).unwrap(), 5);
/// assert_eq!(checked!(10_u32 * (1_u32 + 1_u32)).unwrap(), 20);
/// assert_eq!(checked!(10_u32 - (1_u32 + 1_u32)).unwrap(), 8);
/// assert_eq!(checked!(10_u32 + (1_u32 + 1_u32)).unwrap(), 12);
///
/// let max = u32::max_value();
/// assert!(checked!(max + 1_u32).is_err());
/// assert!(checked!(0_u32 - 1_u32).is_err());
/// ```
#[macro_export]
macro_rules! checked {
    ($a:tt + $b:tt) => {{
        #![allow(clippy::integer_arithmetic)]
        $a.checked_add($b).ok_or_else(|| format!("Operation results in overflow/underflow: {} + {}", $a, $b))
    }};
    ($a:tt - $b:tt) => {{
        #![allow(clippy::integer_arithmetic)]
        $a.checked_sub($b).ok_or_else(|| format!("Operation results in overflow/underflow: {} - {}", $a, $b))
    }};
    ($a:tt * $b:tt) => {{
        #![allow(clippy::integer_arithmetic)]
        $a.checked_mul($b).ok_or_else(|| format!("Operation results in overflow/underflow: {} * {}", $a, $b))
    }};
    ($a:tt / $b:tt) => {{
        #![allow(clippy::integer_arithmetic)]
        $a.checked_div($b).ok_or_else(|| format!("Operation results in overflow/underflow: {} / {}", $a, $b))
    }};
    ($a:tt + $($tokens:tt)*) => {{
        checked!( $($tokens)* ).and_then(|b| {
            b.checked_add($a)
                .ok_or_else(|| format!("Operation results in overflow/underflow: {} + {}", b, $a))
        })
    }};
    ($a:tt - $($tokens:tt)*) => {{
        checked!( $($tokens)* ).and_then(|b| {
            b.checked_sub($a)
                .ok_or_else(|| format!("Operation results in overflow/underflow: {} - {}", b, $a))
        })
    }};
    ($a:tt * $($tokens:tt)*) => {{
        checked!( $($tokens)* ).and_then(|b| {
            b.checked_mul($a)
                .ok_or_else(|| format!("Operation results in overflow/underflow: {} * {}", b, $a))
        })
    }};
    ($a:tt / $($tokens:tt)*) => {{
        checked!( $($tokens)* ).and_then(|b| {
            b.checked_div($a)
                .ok_or_else(|| format!("Operation results in overflow/underflow: {} / {}", b, $a))
        })
    }};
}
