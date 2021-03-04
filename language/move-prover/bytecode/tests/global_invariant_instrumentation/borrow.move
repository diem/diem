module Test {
    resource struct R {
        x: u64,
    }

    spec module {
        invariant [global] forall a: address: global<R>(a).x > 0;
    }

    public fun borrow(a: address) acquires R {
        let r = borrow_global_mut<R>(a);
        r.x = r.x + 1;
    }
}
