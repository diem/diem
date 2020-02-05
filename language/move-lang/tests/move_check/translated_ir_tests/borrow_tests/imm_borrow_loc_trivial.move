module Tester {
    resource struct X { f: u64 }

    fun bump_and_give(x_ref: &mut X, other: &u64): &u64 {
        x_ref.f = x_ref.f + 1;
        &x_ref.f
    }

    fun contrived_example(result: &mut u64) {
        let x = X { f: 0 };
        let other = 100;
        let returned_ref = bump_and_give(&mut x, &other);
        // imagine some more interesting check than this assert
        0x0::Transaction::assert(*returned_ref == x.f, 42);
        *result = *returned_ref;
        X { f: _ } = x;
    }
}
