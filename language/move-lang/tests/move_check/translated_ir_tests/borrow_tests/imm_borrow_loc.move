module Tester {
    resource struct Data { v1: u64, v2: u64 }
    resource struct Box { f: u64 }

    // the resource struct is here to just give a feeling why the computation might not be reorderable
    fun bump_and_pick(b1: &mut Box, b2: &mut Box): &u64 acquires Data {
        let data = borrow_global_mut<Data>(0x0::Transaction::sender());
        b1.f = data.v1;
        b2.f = data.v2;
        if (b1.f > b2.f) &b1.f else &b2.f
    }

    fun larger_field(drop: address, result: &mut u64) acquires Box, Data {
        let b1 = move_from<Box>(0x0::Transaction::sender());
        let b2 = move_from<Box>(drop);

        0x0::Transaction::assert(b1.f == 0, 42);
        0x0::Transaction::assert(b2.f == 0, 42);

        let returned_ref = bump_and_pick(&mut b1, &mut b2);

        // imagine some more interesting check than these asserts
        0x0::Transaction::assert(b1.f != 0, 42);
        0x0::Transaction::assert(b2.f != 0, 42);
        0x0::Transaction::assert((returned_ref == &b1.f) != (returned_ref == &b2.f), 42);

        *result = *returned_ref;
        move_to_sender<Box>(b1);
        Box { f: _ } = b2;
    }
}
