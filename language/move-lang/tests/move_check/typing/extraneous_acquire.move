module 0x8675309::M {
    struct R1 has key {}
    struct R2 has key {}

    fun t0() acquires R1 {

    }

    fun t1(a: address) acquires R1, R2 {
        borrow_global<R1>(a);
    }
}
