module M {
    resource struct R1 {}
    resource struct R2 {}

    fun t0() acquires R1 {

    }

    fun t1(a: address) acquires R1, R2 {
        borrow_global<R1>(a);
    }
}
