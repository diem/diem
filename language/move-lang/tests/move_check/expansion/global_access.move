module M {
    resource struct R {}

    exists(): u64 { 0 }
    move_to_sender(): u64 { 0 }
    borrow_global(): u64 { 0 }
    borrow_global_mut(): u64 { 0 }
    move_from(): u64 { 0 }
    freeze(): u64 { 0 }

    t() acquires R {
        let _ : u64 = exists();
        let _ : bool = .exists<R>(0x0);

        let _ : u64 = move_to_sender();
        let () = .move_to_sender<R>(R{});

        let _ : u64 = borrow_global();
        let _ : &R = .borrow_global<R>(0x0);

        let _ : u64 = move_from();
        let R {} = .move_from<R>(0x0);

        let _ : u64 = borrow_global();
        let r : &mut R = .borrow_global_mut<R>(0x0);

        let _ : u64 = freeze();
        let _ : &R = .freeze(r);
    }
}
