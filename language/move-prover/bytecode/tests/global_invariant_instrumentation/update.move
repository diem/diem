module 0x42::Test {
    struct R has key {
        x: u64,
    }

    spec module {
        invariant update [global] forall a: address: old(global<R>(a).x) < global<R>(a).x;
    }

    public fun incr(a: address) acquires R {
        let r = borrow_global_mut<R>(a);
        r.x = r.x + 1;
    }
}
