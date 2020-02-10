module M {
    resource struct R { f: u64 }

    fun t0(addr: address) acquires R {
        let f = borrow_global_mut<R>(addr).f;
        let r1 = borrow_global_mut<R>(addr);
        r1.f = f
    }

    fun t1(addr: address) acquires R {
        let f = borrow_global<R>(addr).f;
        let r1 = borrow_global_mut<R>(addr);
        r1.f = f
    }

    fun t2(addr: address) acquires R {
        borrow_global_mut<R>(addr).f = borrow_global_mut<R>(addr).f
    }

    fun t3(addr: address) acquires R {
        let R { f } = move_from<R>(addr);
        let r1 = borrow_global_mut<R>(addr);
        r1.f = f
    }

    fun t4(cond: bool, addr: address) acquires R {
        let r = R { f: 0 };
        let f = borrow_global<R>(addr).f;
        let r1; if (cond) r1 = borrow_global_mut<R>(addr) else r1 = &mut r;
        r1.f = f;

        R { f: _ } = r
    }
}
