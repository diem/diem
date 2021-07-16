module 0x42::Test {
    struct Inner { v: u64 }
    struct Outer { i: Inner }

    fun ret_mut(i: &mut Inner): &mut u64 {
        i.v = 0;
        &mut i.v
    }

    fun foo(o: &mut Outer, y: u64) {
        let x = ret_mut(&mut o.i);
        loop {
            spec {
                assert o.i.v <= y;
            };
            if (*x == y) {
                break
            };
            *x = *x + 1;
        }
    }
    spec foo {
        ensures o.i.v == y;
    }
}
