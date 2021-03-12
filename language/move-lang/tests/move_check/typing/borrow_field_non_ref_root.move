module 0x8675309::M {
    struct S has drop { f: u64 }

    fun t0(cond: bool, s: S) {
        (&s.f: &u64);
        (&mut s.f: &mut u64);
        (&(if (cond) S { f: 0 } else S { f: 1 }).f : &u64);
        (&mut (if (cond) S { f: 0 } else S { f: 1 }).f : &mut u64);
    }
}
