module 0x8675309::M {
    struct R1 has key, drop {}
    struct R2 has key {}
    struct R3<T> has key { f: T }

    fun foo1(a: address) acquires R1 {
        borrow_global<R1>(a);
    }

    fun foo2(a: address) acquires R2 {
        borrow_global<R2>(a);
    }

    fun t0(a: address) acquires R1, R2 {
        borrow_global<R1>(a);
        borrow_global<R2>(a);
    }

    fun t1(a: address) acquires R1, R2 {
        borrow_global_mut<R1>(a);
        borrow_global_mut<R2>(a);
    }

    fun t2(a: address) acquires R1, R2 {
        R1{} = move_from<R1>(a);
        R2{} = borrow_global_mut<R2>(a);
    }

    fun t3(a: address) acquires R1, R2 {
        foo1(a);
        foo2(a);
    }

    fun t4(account: &signer, a: address) {
        exists<R1>(a);
        exists<R2>(a);
        move_to<R1>(account, R1{});
        move_to<R2>(account, R2{});
    }

    fun t5(a: address) acquires R3 {
        R3{ f: _ } = move_from<R3<u64>>(a);
        R3{ f: _ } = move_from<R3<R1>>(a);
        borrow_global_mut<R3<bool>>(a);
        borrow_global_mut<R3<R2>>(a);
    }

    fun t6(account: &signer, a: address) {
        exists<R3<u64>>(a);
        exists<R3<R1>>(a);
        move_to<R3<bool>>(account, R3{ f: true });
        move_to<R3<R2>>(account, R3{ f: R2{} });
    }

}
