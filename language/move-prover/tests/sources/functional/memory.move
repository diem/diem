module TestMemory {

    spec module {
        pragma verify = true;
    }

    resource struct R {
        s: S
    }

    struct S {
        x: u64
    }

    public fun mutate(r: &mut R) {
        r.s.x = 1;
    }

    public fun mutate_at(addr: address) acquires R {
        let r = borrow_global_mut<R>(addr);
        r.s.x = 2;
    }
}
