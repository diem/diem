module 0x8675309::M {
    struct R1 has key {}
    struct R2 has key {}
    struct R3 has key {}
    struct R4 has key {}
    struct R5 has key {}
    fun foo(account: &signer) acquires R1 {
        borrow_global<R1, R2>(@0x1);
        exists<R1, R2, R3>(@0x1);
        R1 {} = move_from<R1, R2, R3, R4>(@0x1);
        move_to<R1, R2, R3, R4, R5>(account, R1{});

    }
}
