module M {
    resource struct R1 {}
    resource struct R2 {}

    t0() acquires R1 {

    }

    t1(a: address) acquires R1, R2 {
        borrow_global<R1>(a);
    }
}
