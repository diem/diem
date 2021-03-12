module 0x42::Test {
    struct R has key {
        x: u64,
    }

    spec module {
        invariant [global] forall a: address: global<R>(a).x > 0;
    }

    public fun publish(s: &signer) {
        move_to<R>(s, R{x:1});
    }

    public fun remove(a: address): R acquires R {
        move_from<R>(a)
    }

}
