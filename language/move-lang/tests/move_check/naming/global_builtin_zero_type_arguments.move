module 0x8675309::M {
    struct R1 {}
    fun foo(account: &signer) {
        borrow_global<>(@0x1);
        exists<>(@0x1);
        R1 {} = move_from<>(@0x1);
        move_to<>(account, R1{});
    }
}
