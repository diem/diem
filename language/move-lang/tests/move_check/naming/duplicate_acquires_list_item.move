module M {
    resource struct R {}
    resource struct X {}
    t0() acquires R, X, R {
        borrow_global_mut<R>(0x1);
        borrow_global_mut<X>(0x1);
    }
    t1() acquires R, X, R, R, R {
        borrow_global_mut<R>(0x1);
        borrow_global_mut<X>(0x1);
    }
}
