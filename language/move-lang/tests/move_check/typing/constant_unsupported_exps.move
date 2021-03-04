address 0x42 {
module X {
    public fun f_public() {}
    public(script) fun f_script() {}
    public(friend) fun f_friend() {}
    fun f_private() {}
}

module M {
    resource struct R {}
    struct B { f: u64 }

    const FLAG: bool = false;
    const C: u64 = {
        let x = 0;
        let s: signer = abort 0;
        let b = B { f: 0 };
        spec { };
        &x;
        &mut x;
        f_public();
        f_script();
        f_friend();
        f_private();
        0x42::X::f_public();
        0x42::X::f_script();
        0x42::X::f_friend();
        0x42::X::f_private();
        borrow_global<R>(0x42);
        borrow_global_mut<R>(0x42);
        move_to(s, R{});
        R{} = move_from(0x42);
        freeze(&mut x);
        assert(true, 42);
        if (true) 0 else 1;
        loop ();
        loop { break; continue; };
        while (true) ();
        x = 1;
        return 0;
        abort 0;
        *(&mut 0) = 0;
        b.f = 0;
        b.f;
        *&b.f;
        (0, 1);
        FLAG;
        0
    };
    public fun f_public() {}
    public(script) fun f_script() {}
    public(friend) fun f_friend() {}
    fun f_private() {}
}
}
