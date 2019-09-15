// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::*;
use move_ir::{assert_no_error, assert_other_error};

#[test]
fn mutate_simple_reference() {
    let mut test_env = TestEnvironment::default();
    let sender = hex::encode(test_env.accounts.get_address(0));

    // This test creates two references that refer to the same local value.
    // Case 1: Mutating one of them will invalidate the other reference.
    // Case 2: You can keep mutating on the same reference for multiple times.

    let program = format!(
        "
modules:
module M {{
    struct A{{f: u64}}

    public new(x: u64): Self.A {{
        return A{{f: move(x)}};
    }}

    public t(a: &mut Self.A) {{
        let f1;
        let f2;

        f1 = &mut copy(a).f;
        f2 = &mut copy(a).f;
        *move(f1) = 4;
        *move(f2) = 5;

        _ = move(a);
        return;
    }}
}}
script:
import 0x{0}.M;
main() {{
    let x;
    let x_ref;
    let f1;
    let f2;
    x = M.new(0);
    x_ref = &mut x;
    f1 = M.t(move(x_ref));
}}",
        sender
    );
    assert_other_error!(
        test_env.run(to_script(program.as_bytes(), vec![])),
        "In function \'t\': Invalid usage of \'a\'. Field \'f\' is being borrowed by: \'f1\'"
    );

    let program2 = format!(
        "
modules:
module M2 {{
    struct A{{f: u64}}

    public new(x: u64): Self.A {{
        return A{{f: move(x)}};
    }}

    public f(a: &mut Self.A): &mut u64 {{
        let f;
        f = &mut copy(a).f;
        _ = move(a);
        return move(f);
    }}
}}
script:
import 0x{0}.M2;
main() {{
    let x;
    let x_ref;
    let f1;
    x = M2.new(0);
    x_ref = &mut x;
    f1 = M2.f(copy(x_ref));
    *copy(f1) = 4;
    *move(f1) = 5;
    _ = move(x_ref);
    return;
}}",
        sender
    );
    assert_no_error!(test_env.run(to_script(program2.as_bytes(), vec![])))
}

#[test]
fn mutate_nested_reference() {
    let mut test_env = TestEnvironment::default();
    let sender = hex::encode(test_env.accounts.get_address(0));

    // This test creates a module that looks like following:
    //  A: f
    //     |
    //    {g: u64}
    //
    // The two test cases are following:
    // 1. let f = &mut f;
    //    let g = &mut copy(f).g;
    //    *copy(f).g = 1;
    //    *copy(f) = ...;
    //    This should be success because the mutation of kid shouldn't invalidate the read of
    //    parent.
    //
    // 2.  let f = &mut f;
    //     let g = &mut copy(f).g;
    //     *copy(f) = ...;
    //     *copy(f).g = 1; <-- This line should fail because mutation of parent will invalidate kid.
    let program = format!(
        "
modules:
module M {{
    struct A{{f: Self.B}}
    struct B{{g: u64}}

    public A(f: Self.B): Self.A {{
        return A{{f: move(f)}};
    }}

    public B(g: u64): Self.B {{
        return B{{g: move(g)}};
    }}

    public t(a: &mut Self.A) {{
        let f_ref;
        let g_ref;
        let b;

        f_ref = &mut copy(a).f;
        g_ref = &mut copy(f_ref).g;
        *move(g_ref) = 5;
        b = Self.B(2);
        *move(f_ref) = move(b);
        _ = move(a);
        return;
    }}
}}
script:
import 0x{0}.M;
main() {{
    let b;
    let x;
    let x_ref;
    let f_ref;
    let g_ref;

    b = M.B(0);
    x = M.A(move(b));
    x_ref = &mut x;
    M.t(move(x_ref));
    return;
}}",
        sender,
    );
    assert_no_error!(test_env.run(to_script(program.as_bytes(), vec![])));

    // Mutate kid after parent
    let program2 = format!(
        "
modules:
module M2 {{
    struct A{{f: Self.B}}
    struct B{{g: u64}}

    public A(f: Self.B): Self.A {{
        return A{{f: move(f)}};
    }}

    public B(g: u64): Self.B {{
        return B{{g: move(g)}};
    }}

    public t(a: &mut Self.A) {{
        let f_ref;
        let g_ref;
        let b;

        f_ref = &mut copy(a).f;
        g_ref = &mut copy(f_ref).g;

        b = Self.B(2);
        *move(f_ref) = move(b);
        *move(g_ref) = 5;

        _ = move(a);

        return;
    }}
}}
script:
import 0x{0}.M2;
main() {{
    let b;
    let x;
    let x_ref;
    let f_ref;
    let g_ref;

    b = M2.B(0);
    x = M2.A(move(b));
    x_ref = &mut x;
    M2.t(move(x_ref));

    return;
}}",
        sender,
    );
    assert_other_error!(
        test_env.run(to_script(program2.as_bytes(), vec![])),
        "In function \'t\': Invalid usage of \'f_ref\'. Field \'g\' is being borrowed by: \'g_ref\'"
    )
}

#[test]
fn mutate_sibling_reference() {
    let mut test_env = TestEnvironment::default();
    let sender = hex::encode(test_env.accounts.get_address(0));
    // This test create a following struct:
    // A:       f
    //         / \
    //        /   \
    // B:    g     h
    // The tests are as following:
    // let f = &mut f;
    // let g = &mut copy(f).g;
    // let h = &mut copy(f).h;
    // 1. *copy(g) = 5;
    //    let h2 = *copy(h);
    //    let f2 = *copy(f);
    //    This should success because mutating a kid will not invalidate its parent nor its siblings
    // 2. *copy(f) = _;
    //    let g2 = *copy(g); <-- This should fail because mutating a parent will invalidate all of
    // its    kids.
    //    let h2 = *copy(h); <-- This will fail because of the exact same reasoning.
    let program = format!(
        "
modules:
module M {{
    struct A{{f: Self.B}}
    struct B{{g: u64, h: u64}}

    public A(f: Self.B): Self.A {{
        return A{{f: move(f)}};
    }}

    public B(g: u64, h: u64): Self.B {{
        return B{{g: move(g), h: move(h)}};
    }}

    public t(a: &mut Self.A) {{
        let f;
        let g;
        let h;
        let f_ref;
        let g_ref;
        let h_ref;

        f_ref = &mut copy(a).f;
        g_ref = &mut copy(f_ref).g;
        h_ref = &mut copy(f_ref).h;
        *move(g_ref) = 5;
        h = *move(h_ref);
        assert(move(h) == 1, 42);
        f = *move(f_ref);
        _ = move(a);
        return;
    }}
}}
script:
import 0x{0}.M;
main() {{
    let b;
    let x;
    let x_ref;
    let f_ref;
    let g_ref;
    let h_ref;
    let h;
    let f;

    b = M.B(0, 1);
    x = M.A(move(b));
    x_ref = &mut x;
    M.t(move(x_ref));

    return;
}}",
        sender,
    );
    assert_no_error!(test_env.run(to_script(program.as_bytes(), vec![])));

    // Mutating parent will invalidate both kids
    let program2 = format!(
        "
modules:
module M2 {{
    struct A{{f: Self.B}}
    struct B{{g: u64, h: u64}}

    public A(f: Self.B): Self.A {{
        return A{{f: move(f)}};
    }}

    public B(g: u64, h: u64): Self.B {{
        return B{{g: move(g), h: move(h)}};
    }}

    public t(a: &mut Self.A) {{
        let f_ref;
        let h_ref;
        let b;
        let h;

        f_ref = &mut copy(a).f;
        h_ref = &mut copy(f_ref).h;

        b = Self.B(2, 3);
        *move(f_ref) = move(b);
        h = *move(h_ref);
        _ = move(a);
    }}
}}
script:
import 0x{0}.M2;
main() {{
    let b;
    let x;
    let x_ref;

    b = M2.B(0, 1);
    x = M2.A(move(b));
    x_ref = &mut x;
    M2.t(move(x_ref));
    return;
}}",
        sender,
    );
    assert_other_error!(
        test_env.run(to_script(program2.as_bytes(), vec![])),
        "In function \'t\': Invalid usage of \'f_ref\'. Field \'h\' is being borrowed by: \'h_ref\'"
    );

    let program3 = format!(
        "
modules:
module M3 {{
    struct A{{f: Self.B}}
    struct B{{g: u64, h: u64}}

    public A(f: Self.B): Self.A {{
        return A{{f: move(f)}};
    }}

    public B(g: u64, h: u64): Self.B {{
        return B{{g: move(g), h: move(h)}};
    }}

    public t(a: &mut Self.A) {{
        let f_ref;
        let g_ref;
        let b;
        let g;

        f_ref = &mut copy(a).f;
        g_ref = &mut copy(f_ref).g;

        b = Self.B(2, 3);
        *move(f_ref) = move(b);
        g = *move(g_ref);
        _ = move(a);
    }}
}}
script:
import 0x{0}.M3;
main() {{
    let b;
    let x;
    let x_ref;

    b = M3.B(0, 1);
    x = M3.A(move(b));
    x_ref = &mut x;
    M3.t(move(x_ref));
    return;
}}",
        sender,
    );
    assert_other_error!(
        test_env.run(to_script(program3.as_bytes(), vec![])),
        "In function \'t\': Invalid usage of \'f_ref\'. Field \'g\' is being borrowed by: \'g_ref\'"
    );
}
