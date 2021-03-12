module 0x8675309::M {
    struct S { f: u64 }

    fun t0(s: &S, s_mut: &mut S): (u64, u64) {
        (s.f, s_mut.f)
    }
}
