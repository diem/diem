// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::*;
use move_ir::{assert_no_error, assert_other_error};

#[test]
fn publish_existing_module() {
    let mut test_env = TestEnvironment::default();
    let sender = hex::encode(test_env.accounts.get_address(0));
    let program = b"
modules:
module Currency {
        resource Coin{money: R#Self.Coin}
        public new(m: R#Self.Coin): R#Self.Coin {
            return Coin{money: move(m)};
        }
        public value(this :&R#Self.Coin): u64 {
            let ref;
            let val;
            ref = &copy(this).money;
            val = Self.value(move(ref));
            release(move(this));
            return move(val);
        }
}
module Currency {
        resource Coin{money: R#Self.Coin}
        public new(m: R#Self.Coin): R#Self.Coin {
            return Coin{money: move(m)};
        }
        public value(this: &R#Self.Coin): u64 {
            let ref;
            let val;
            ref = &copy(this).money;
            val = Self.value(move(ref));
            release(move(this));
            return move(val);
        }
}
script:
main() {
    return;
}";
    // TODO: check error type once we add more macros.
    assert_other_error!(
        test_env.run(to_script(program, vec![])),
        format!("Publish to an existing module 0x{}.Currency", sender)
    )
}
