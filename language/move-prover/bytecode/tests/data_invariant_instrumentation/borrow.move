module Test {
    resource struct R {
        x: u64,
        s: S
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

    public fun test_borrow_imm(): u64 acquires R {
        let r = borrow_global<R>(0x1);
        r.x
    }

    public fun test_borrow_mut(): u64 acquires R {
        let r = borrow_global_mut<R>(0x1);
        r.s.y = 2;
        r.x = 3;
        r.x
    }

    public fun test_borrow_mut_local(): R {
        let d = R{x: 2, s: S{y:1}};
        let r = &mut d;
        r.s.y = 2;
        r.x = 3;
        d
    }
}
