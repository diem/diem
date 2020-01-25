module M {
    struct S {f: u64}
    resource struct R {}

    public borrow_local(x: u64): u64 {
        // implicit freeze creates a temp local. If the temp is inlined, the copy won't fail
        let (boom, u): (&u64, u64) = (&mut x, x);
        *boom + u + x
    }

    public deref(x: u64): u64 {
        let r = &mut x;
        // implicit freeze creates a temp local. If the temp is inlined, the deref won't fail
        let (f, u): (&u64, u64) = (r, *r);
        *f + u + x
    }

    public borrow_field(s: &mut S): u64 {
        // implicit freeze creates a temp local. If the temp is inlined, the borrow field won't fail
        let (f, u): (&u64, u64) = (&mut s.f, s.f);
        *f + u
    }

    public bg(a: address) acquires R {
        // implicit freeze creates a temp local.
        // If the temp is inlined, the borrow global won't fail
        let (f, u): (&R, &R) = (borrow_global_mut<R>(a), borrow_global<R>(a));
        f;
        u;
    }
}
