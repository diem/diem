module Tester {
    struct X { f: u64 }

    fun bump_and_give(x_ref: &mut X): &u64 {
        x_ref.f = x_ref.f + 1;
        &x_ref.f
    }

    fun contrived_example(x_ref: &mut X): &u64 {
        let returned_ref = bump_and_give(x_ref);
        // imagine some more interesting check than this assert
        0x0::Transaction::assert(*returned_ref == *&x_ref.f, 42);
        returned_ref
    }
}
