module 0x8675309::M {
    struct R has key { f: u64 }

    fun t0(addr: address) acquires R {
        let r1 = borrow_global_mut<R>(addr);
        let r2 = borrow_global<R>(addr);
        r1.f = r2.f
    }

    fun t1(addr: address) acquires R {
        let f = &mut borrow_global_mut<R>(addr).f;
        let r2 = borrow_global<R>(addr);
        *f = r2.f
    }

    fun t2(addr: address) acquires R {
        let r1 = borrow_global_mut<R>(addr);
        let f = &borrow_global<R>(addr).f;
        r1.f = *f
    }

    fun t3(addr: address) acquires R {
        let r2 = borrow_global<R>(addr);
        let r1 = borrow_global_mut<R>(addr);
        r1.f = r2.f
    }

    fun t4(addr: address) acquires R {
        let f = &mut borrow_global_mut<R>(addr).f;
        let r2 = borrow_global<R>(addr);
        *f = r2.f
    }

    fun t5(addr: address) acquires R {
        let r1 = borrow_global_mut<R>(addr);
        let f = &borrow_global<R>(addr).f;
        r1.f = *f
    }

    fun t6(cond: bool, addr: address) acquires R {
        let r = R { f: 0 };
        let r1; if (cond) r1 = borrow_global_mut<R>(addr) else r1 = &mut r;
        let f = &borrow_global<R>(addr).f;
        r1.f = *f;

        R { f: _ } = r
    }
}
