module M {
    resource struct R {}

    fun exists(): u64 { 0 }
    fun move_to_sender(): u64 { 0 }
    fun borrow_global(): u64 { 0 }
    fun borrow_global_mut(): u64 { 0 }
    fun move_from(): u64 { 0 }
    fun freeze(): u64 { 0 }

    fun t() acquires Self::R {
        let _ : u64 = exists();
        let _ : bool = ::exists<Self::R>(0x0);

        let _ : u64 = move_to_sender();
        let () = ::move_to_sender<Self::R>(Self::R{});

        let _ : u64 = borrow_global();
        let _ : &Self::R = ::borrow_global<Self::R>(0x0);

        let _ : u64 = move_from();
        let Self::R {} = ::move_from<Self::R>(0x0);

        let _ : u64 = borrow_global();
        let r : &mut Self::R = ::borrow_global_mut<Self::R>(0x0);

        let _ : u64 = freeze();
        let _ : &Self::R = ::freeze<Self::R>(r);
    }
}
