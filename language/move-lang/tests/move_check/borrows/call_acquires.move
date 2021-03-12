module 0x8675309::M {
    struct R has key { f: u64 }
    fun acq(addr: address): R acquires R {
        move_from(addr)
    }

    fun t0(addr: address) acquires R {
        R { f: _ } = acq(addr);
        R { f: _ } = acq(addr);
    }

    fun t1(addr: address) acquires R {
        R { f: _ } = acq(addr);
        borrow_global_mut<R>(addr);
    }

    fun t2(addr: address) acquires R {
        R { f: _ } = acq(addr);
        borrow_global<R>(addr);
    }

    fun t3(cond: bool, addr: address) acquires R {
        let r = R { f: 0 };
        R { f: _ } = acq(addr);
        let r1; if (cond) r1 = borrow_global_mut<R>(addr) else r1 = &mut r;
        r1.f = 0;
        R { f: _ } = r
    }
}
