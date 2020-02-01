module M {
    resource struct R { f: u64 }

    t0(addr: address): bool acquires R {
        let f = &borrow_global_mut<R>(addr).f;
        let r1 = borrow_global<R>(addr);
        f == &r1.f
    }

    t1(addr: address): bool acquires R {
        let f = borrow_global<R>(addr).f;
        let r1 = borrow_global<R>(addr);
        f == r1.f
    }

    t2(addr: address):bool acquires R {
        borrow_global<R>(addr).f == borrow_global<R>(addr).f
    }

    t3(addr: address): bool acquires R {
        let R { f } = move_from<R>(addr);
        let r1 = borrow_global<R>(addr);
        r1.f == f
    }

    t4(cond: bool, addr: address): bool acquires R {
        let r = R { f: 0 };
        let f = &borrow_global<R>(addr).f;
        let r1; if (cond) r1 = borrow_global<R>(addr) else r1 = &mut r;
        let res = f == &r1.f;
        R { f: _ } = r;
        res
    }
}
