module 0x8675309::M {
    struct R has key {}
    struct X has key {}
    fun t0() acquires R, X, R {
        borrow_global_mut<R>(@0x1);
        borrow_global_mut<X>(@0x1);
    }
    fun t1() acquires R, X, R, R, R {
        borrow_global_mut<R>(@0x1);
        borrow_global_mut<X>(@0x1);
    }
}
