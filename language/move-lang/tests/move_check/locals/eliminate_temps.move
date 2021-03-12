module 0x8675309::M {
    struct S has drop {f: u64}
    struct R has key {}

    public fun borrow_local(x: u64): u64 {
        // implicit freeze creates a temp local. If the temp is inlined, the copy won't fail
        let (boom, u): (&u64, u64) = (&mut x, x);
        *boom + u + x
    }

    public fun deref(x: u64): u64 {
        let r = &mut x;
        // implicit freeze creates a temp local. If the temp is inlined, the deref won't fail
        let (f, u): (&u64, u64) = (r, *r);
        *f + u + x
    }

    public fun borrow_field(s: &mut S): u64 {
        // implicit freeze creates a temp local. If the temp is inlined, the borrow field won't fail
        let (f, u): (&u64, u64) = (&mut s.f, s.f);
        *f + u
    }

    public fun bg(a: address) acquires R {
        // implicit freeze creates a temp local.
        // If the temp is inlined, the borrow global won't fail
        let (f, u): (&R, &R) = (borrow_global_mut<R>(a), borrow_global<R>(a));
        f;
        u;
    }
}
