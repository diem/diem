module 0x8675309::M {
    struct R has key {}

    fun t0(a: &signer) acquires R {
        let _ : bool = exists<R>(@0x0);
        let () = move_to<R>(a, R{});
        let _ : &R = borrow_global<R>(@0x0);
        let _ : &mut R = borrow_global_mut<R>(@0x0);
        let R {} = move_from<R>(@0x0);
    }

    fun t1(a: &signer) acquires R {
        let _ : bool = exists<R>(@0x0);
        let () = move_to(a, R{});
        let _ : &R = borrow_global(@0x0);
        let _ : &mut R = borrow_global_mut(@0x0);
        let R {} = move_from(@0x0);
    }
}
