module 0x8675309::M {
    struct R has key { f: u64 }

    fun t0(addr: address): bool acquires R {
        let f = &borrow_global_mut<R>(addr).f;
        let r1 = borrow_global<R>(addr);
        f == &r1.f
    }

    fun t1(addr: address): bool acquires R {
        let f = borrow_global<R>(addr).f;
        let r1 = borrow_global<R>(addr);
        f == r1.f
    }

    fun t2(addr: address):bool acquires R {
        borrow_global<R>(addr).f == borrow_global<R>(addr).f
    }

    fun t3(addr: address): bool acquires R {
        let R { f } = move_from<R>(addr);
        let r1 = borrow_global<R>(addr);
        r1.f == f
    }

    fun t4(cond: bool, addr: address): bool acquires R {
        let r = R { f: 0 };
        let f = &borrow_global<R>(addr).f;
        let r1; if (cond) r1 = borrow_global<R>(addr) else r1 = &mut r;
        let res = f == &r1.f;
        R { f: _ } = r;
        res
    }
}
