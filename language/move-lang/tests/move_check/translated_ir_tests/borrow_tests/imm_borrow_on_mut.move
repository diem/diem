module Tester {
    use 0x1::Signer;

    resource struct Initializer { x: u64, y: u64 }
    struct Point { x: u64, y: u64 }

    // the resource struct is here to just give a feeling why the computation might not be reorderable
    fun set_and_pick(account: &signer, p: &mut Point): &mut u64 acquires Initializer {
        let sender = Signer::address_of(account);
        let init = borrow_global_mut<Initializer>(sender);
        p.x = init.x;
        p.y = init.y;
        if (p.x >= p.y) &mut p.x else &mut p.y
    }

    fun bump_and_give(u: &mut u64): &u64 {
        *u = *u + 1;
        freeze(u)
    }

    fun larger_field(account: &signer, point_ref: &mut Point): &u64 acquires Initializer {
        assert(point_ref.x == 0, 42);
        assert(point_ref.y == 0, 42);

        let field_ref = set_and_pick(account, point_ref);
        let returned_ref = bump_and_give(field_ref);

        // imagine some more interesting check than this assert
        assert(
            (*returned_ref == point_ref.x) &&
            (*returned_ref != point_ref.y),
            42
        );
        returned_ref
    }
}
