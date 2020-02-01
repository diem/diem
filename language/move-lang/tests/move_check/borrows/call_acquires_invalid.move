module M {
    resource struct R { f: u64 }
    id<T>(r: &T): &T {
        r
    }
    id_mut<T>(r: &mut T): &mut T {
        r
    }

    acq(addr: address): R acquires R {
        move_from(addr)
    }

    t0(addr: address) acquires R {
        let r1 = borrow_global_mut<R>(addr);
        let R { f } = acq(addr);
        r1.f = f
    }

    t1(addr: address) acquires R {
        let f_ref = &mut borrow_global_mut<R>(addr).f;
        let R { f } = acq(addr);
        *f_ref = f
    }

    t2(addr: address) acquires R {
        let r1 = id_mut(borrow_global_mut<R>(addr));
        let R { f } = acq(addr);
        r1.f = f
    }

    t3(addr: address) acquires R {
        let f_ref = id_mut(&mut borrow_global_mut<R>(addr).f);
        let R { f } = acq(addr);
        *f_ref = f
    }

    t4(addr: address): u64 acquires R {
        let r1 = borrow_global<R>(addr);
        let R { f } = acq(addr);
        r1.f + f
    }

    t5(addr: address): u64 acquires R {
        let f_ref = &borrow_global<R>(addr).f;
        let R { f } = acq(addr);
        *f_ref + f
    }

    t6(addr: address): u64 acquires R {
        let r1 = id(borrow_global<R>(addr));
        let R { f } = acq(addr);
        r1.f + f
    }

    t7(addr: address): u64 acquires R {
        let f_ref = id(&borrow_global<R>(addr).f);
        let R { f } = acq(addr);
        *f_ref + f
    }


    t8(cond: bool, addr: address) acquires R {
        let r = R { f: 0 };
        let r1; if (cond) r1 = borrow_global_mut<R>(addr) else r1 = &mut r;
        let R { f } = acq(addr);
        r1.f = f;

        R { f: _ } = r
    }
}
