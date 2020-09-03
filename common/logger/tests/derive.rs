// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_log_derive::Schema;

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
