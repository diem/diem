// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::compile_and_execute;
use vm::assert_ok;

#[test]
fn simple_main() {
    let program = String::from(
        "
        modules:
        module M {
            public max(a: u64, b: u64): u64 {
                if (copy(a) > copy(b)) {
                    return copy(a);
                } else {
                    return copy(b);
                }
                return 0;
            }

            public sum(a: u64, b: u64): u64 {
                let c: u64;
                c = copy(a) + copy(b);
                return copy(c);
            }
        }
        script:
        import 0x0000000000000000000000000000000000000000000000000000000000000000.M;

        main() {
            let a: u64;
            let b: u64;
            let c: u64;
            let d: u64;

            a = 10;
            b = 2;
            c = M.max(copy(a), copy(b));
            d = M.sum(copy(a), copy(b));
            assert(copy(c) == 10, 42);
            assert(copy(d) == 12, 42);
            return;
        }
        ",
    );
    assert_ok!(compile_and_execute(&program, vec![]));
}

#[test]
fn diff_type_args() {
    let program = String::from(
        "
        modules:
        module M {
            public case(a: u64, b: bool): u64 {
                if (copy(b)) {
                    return copy(a);
                } else {
                    return 42;
                }
                return 112;
            }
        }
        script:
        import 0x0.M;

        main() {
            let a: u64;
            a = 10;
            a = M.case(move(a), false);
            assert(copy(a) == 42, 41);
            return;
        }
        ",
    );
    assert_ok!(compile_and_execute(&program, vec![]));
}

#[test]
fn multiple_return_values() {
    let program = String::from(
        "
        modules:
        module M {
            public id3(a: u64, b: bool, c: address): u64 * bool * address {
                return move(a), move(b), move(c);
            }
        }
        script:
        import 0x0.M;

        main() {
            let a: u64;
            let b: bool;
            let c: address;

            a = 10;
            b = false;
            c = 0x0;

            a, b, c = M.id3(move(a), move(b), move(c));
            assert(move(a) == 10, 42);
            assert(move(b) == false, 43);
            assert(move(c) == 0x0, 44);
            return;
        }
        ",
    );
    assert_ok!(compile_and_execute(&program, vec![]));
}
