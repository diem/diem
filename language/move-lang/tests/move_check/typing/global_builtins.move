module M {
    resource struct R {}

    fun t0() acquires R {
        let _ : bool = exists<R>(0x0);
        let () = move_to_sender<R>(R{});
        let _ : &R = borrow_global<R>(0x0);
        let _ : &mut R = borrow_global_mut<R>(0x0);
        let R {} = move_from<R>(0x0);
    }

    fun t1() acquires R {
        let _ : bool = exists<R>(0x0);
        let () = move_to_sender(R{});
        let _ : &R = borrow_global(0x0);
        let _ : &mut R = borrow_global_mut(0x0);
        let R {} = move_from(0x0);
    }
}
