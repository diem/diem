// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::compile_and_execute;
use vm::assert_ok;

#[test]
fn simple_unpack() {
    let program = String::from(
        "
modules:
module Test {
    resource T { i: u64, b: bool }

    public new_t(): R#Self.T {
        return T { i: 0, b: false };
    }

    public unpack_t(t: R#Self.T) {
        let i: u64;
        let flag: bool;
        T { i, b: flag } = move(t);
        return;
    }

}
script:
import 0x0.Test;
main() {
    let t: R#Test.T;

    t = Test.new_t();
    Test.unpack_t(move(t));

    return;
}",
    );
    assert_ok!(compile_and_execute(&program, vec![]));
}
