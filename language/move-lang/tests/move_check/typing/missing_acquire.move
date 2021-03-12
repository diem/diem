module 0x8675309::M {
    struct R1 has key {}
    struct R2 has key {}

    fun t1(a: address) acquires R2 {
        borrow_global<R2>(a);

        r1(a);
        borrow_global<R1>(a);
        borrow_global_mut<R1>(a);
        R1{} = move_from<R1>(a);
    }

    fun r1(a: address) acquires R1 {
        borrow_global<R1>(a);
    }
}
