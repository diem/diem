module M {
    resource struct R {}

    fun t0() acquires R {
        let _ : bool = exists<R>();
        let () = move_to_sender<R>();
        let _ : &R = borrow_global<R>();
        let _ : &mut R = borrow_global_mut<R>();
        let R {} = move_from<R>();
    }

    fun t1() acquires R {
        let _ : bool = exists<R>(0);
        let () = move_to_sender<R>(0);
        let () = move_to_sender(0);
        let _ : &R = borrow_global<R>(0);
        let _ : &mut R = borrow_global_mut<R>(0);
        let R {} = move_from<R>(0);
    }

    fun t2() acquires R {
        let _ : bool = exists<R>(0x0, 0);
        let () = move_to_sender<R>(R{}, 0);
        let () = move_to_sender(R{}, 0);
        let _ : &R = borrow_global<R>(0x0, false);
        let _ : &mut R = borrow_global_mut<R>(0x0, true);
        let R {} = move_from<R>(0x0, 0);
    }

}
