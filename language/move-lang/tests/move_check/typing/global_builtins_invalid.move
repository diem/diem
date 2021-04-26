module 0x8675309::M {
    struct R has key {}

    fun t0() acquires R {
        let _ : bool = exists<R>();
        let () = move_to<R>();
        let _ : &R = borrow_global<R>();
        let _ : &mut R = borrow_global_mut<R>();
        let R {} = move_from<R>();
    }

    fun t1(a: &signer) acquires R {
        let _ : bool = exists<R>(0);
        let () = move_to<R>(a, 0);
        let () = move_to(a, 0);
        let _ : &R = borrow_global<R>(0);
        let _ : &mut R = borrow_global_mut<R>(0);
        let R {} = move_from<R>(0);
    }

    fun t2(a: &signer) acquires R {
        let _ : bool = exists<R>(@0x0, 0);
        let () = move_to<R>(a, R{}, 0);
        let () = move_to(a, R{}, 0);
        let _ : &R = borrow_global<R>(@0x0, false);
        let _ : &mut R = borrow_global_mut<R>(@0x0, true);
        let R {} = move_from<R>(@0x0, 0);
    }

}
