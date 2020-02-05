module Tester {
    resource struct Initializer { x: u64, y: u64 }
    struct Point { x: u64, y: u64 }

    // the resource struct is here to just give a feeling why the computation might not be reorderable
    fun set_and_pick(p: &mut Point): &mut u64 acquires Initializer {
        let init = borrow_global_mut<Initializer>(0x0::Transaction::sender());
        p.x = init.x;
        p.y = init.y;
        if (p.x >= p.y) &mut p.x else &mut p.y
    }

    fun bump_and_give(u: &mut u64): &u64 {
        *u = *u + 1;
        freeze(u)
    }

    fun larger_field(point_ref: &mut Point): &u64 acquires Initializer {
        0x0::Transaction::assert(point_ref.x == 0, 42);
        0x0::Transaction::assert(point_ref.y == 0, 42);

        let field_ref = set_and_pick(point_ref);
        let returned_ref = bump_and_give(field_ref);

        // imagine some more interesting check than this assert
        0x0::Transaction::assert(
            (*returned_ref == point_ref.x) &&
            (*returned_ref != point_ref.y),
            42
        );
        returned_ref
    }
}
