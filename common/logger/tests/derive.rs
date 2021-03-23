// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_log_derive::Schema;

#[test]
fn simple() {
    #[derive(Schema)]
    pub struct Test {
        required: usize,
        optional: Option<usize>,
    }

    let t = Test {
        required: 0,
        optional: None,
    };

    t.required(1).optional(2);
}

#[test]
fn lifetime() {
    #[derive(Default, Schema)]
    pub struct Test<'a> {
        s: Option<&'a str>,
    }

    let t = Test::default();

    t.s("foo");
}

#[test]
fn generic() {
    #[derive(Default, Schema)]
    pub struct Test<T: ::serde::Serialize> {
        s: Option<T>,
    }

    let t = Test::default();

    t.s(5);
}

#[test]
fn attrs() {
    #[derive(Default, Schema)]
    pub struct Test {
        #[schema(debug)]
        debug: Option<Vec<usize>>,
        #[schema(display)]
        display: Option<usize>,
    }

    let t = Test::default();

    t.debug(vec![1, 2, 3]).display(4);
}

#[test]
fn error_trait_object() {
    use std::fmt;

    #[derive(Default, Schema)]
    pub struct Test<'a> {
        #[schema(debug)]
        debug_error: Option<&'a dyn ::std::error::Error>,
        #[schema(display)]
        display_error: Option<&'a dyn ::std::error::Error>,
    }

    #[derive(Debug)]
    struct OurError;

    impl fmt::Display for OurError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "Custom Error")
        }
    }

    impl ::std::error::Error for OurError {}

    let debug_error = ::std::io::Error::new(::std::io::ErrorKind::Other, "This is an error");
    let display_error = OurError;
    let t = Test::default()
        .debug_error(&debug_error)
        .display_error(&display_error);

    ::diem_logger::info!(t);
}
