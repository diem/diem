// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

/// A wrapper around `std::num::NonZeroUsize` to no longer worry about `unwrap()`
#[macro_export]
macro_rules! NonZeroUsize {
    ($num:expr) => {
        NonZeroUsize!($num, "Must be non-zero")
    };
    ($num:expr, $message:literal) => {
        std::num::NonZeroUsize::new($num).expect($message)
    };
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_nonzero() {
        assert_eq!(1, NonZeroUsize!(1).get());
        assert_eq!(std::usize::MAX, NonZeroUsize!(std::usize::MAX).get());
    }

    #[test]
    #[should_panic(expected = "Must be non-zero")]
    fn test_zero() {
        NonZeroUsize!(0);
    }

    #[test]
    #[should_panic(expected = "Custom message")]
    fn test_zero_custom_message() {
        NonZeroUsize!(0, "Custom message");
    }
}
