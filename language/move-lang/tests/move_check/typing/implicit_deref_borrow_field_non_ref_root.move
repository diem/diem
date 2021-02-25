module M {
    struct S has drop { f: u64 }

    fun t0(cond: bool, s: S) {
        (s.f: u64);
        ((if (cond) S { f: 0 } else S { f: 1 }).f : u64);
    }
}
