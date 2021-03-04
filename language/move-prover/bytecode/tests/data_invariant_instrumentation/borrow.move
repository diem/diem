module Test {
    resource struct R<T> {
        x: u64,
        s: S,
        t: T,
    }

    struct S {
        y: u64
    }

    spec struct R {
        invariant x > s.y;
    }

    spec struct S {
        invariant y > 0;
    }

    public fun test_borrow_imm<T>(): u64 acquires R {
        let r = borrow_global<R<T>>(0x1);
        r.x
    }

    public fun test_borrow_mut<T>(): u64 acquires R {
        let r = borrow_global_mut<R<T>>(0x1);
        r.s.y = 2;
        r.x = 3;
        r.x
    }

    public fun test_borrow_mut_local(): R<u64> {
        let d = R{x: 2, s: S{y:1}, t: 0};
        let r = &mut d;
        r.s.y = 2;
        r.x = 3;
        d
    }
}
