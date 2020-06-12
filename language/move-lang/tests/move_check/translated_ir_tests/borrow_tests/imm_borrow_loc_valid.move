module Tester {
    use 0x0::Signer;

    resource struct Data { v1: u64, v2: u64 }
    resource struct Box { f: u64 }

    // the resource struct is here to just give a feeling why the computation might not be reorderable
    fun bump_and_pick(account: &signer, b1: &mut Box, b2: &mut Box): &u64 acquires Data {
        let sender = Signer::address_of(account);
        let data = borrow_global_mut<Data>(sender);
        b1.f = data.v1;
        b2.f = data.v2;
        if (b1.f > b2.f) &b1.f else &b2.f
    }

    fun larger_field(account: &signer, drop: address, result: &mut u64) acquires Box, Data {
        let sender = Signer::address_of(account);
        let b1 = move_from<Box>(sender);
        let b2 = move_from<Box>(drop);

        assert(b1.f == 0, 42);
        assert(b2.f == 0, 42);

        let returned_ref = bump_and_pick(account, &mut b1, &mut b2);

        // imagine some more interesting check than these asserts
        assert(b1.f != 0, 42);
        assert(b2.f != 0, 42);
        assert(
            (returned_ref == &(&mut b1).f) != (returned_ref == &(&mut b2).f),
            42
        );

        *result = *returned_ref;
        move_to<Box>(account, b1);
        Box { f: _ } = b2;
    }
}
