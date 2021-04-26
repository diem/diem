module 0x8675309::M {
    struct R has key {}

    fun t(account: &signer) acquires R {
        let _ : bool = exists<R>(@0x0);
        let () = move_to<R>(account, R{});
        let _ : &R = borrow_global<R>(@0x0);
        let _ : &mut R = borrow_global_mut<R>(@0x0);
        let R {} = move_from<R>(@0x0);
    }
}
