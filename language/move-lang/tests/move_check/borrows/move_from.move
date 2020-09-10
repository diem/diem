module M {
    resource struct R { f: u64 }

    fun t0(addr: address) acquires R {
        R { f: _ } = move_from<R>(addr);
        R { f: _ } = move_from<R>(addr);
    }

    fun t1(addr: address) acquires R {
        R { f: _ } = move_from<R>(addr);
        borrow_global_mut<R>(addr);
    }

    fun t2(addr: address) acquires R {
        R { f: _ } = move_from<R>(addr);
        borrow_global<R>(addr);
    }

    fun t3(cond: bool, addr: address) acquires R {
        let r = R { f: 0 };
        R { f: _ } = move_from<R>(addr);
        let r1; if (cond) r1 = borrow_global_mut<R>(addr) else r1 = &mut r;
        r1.f = 0;
        R { f: _ } = r
    }
}
