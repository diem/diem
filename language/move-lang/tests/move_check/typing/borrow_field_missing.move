module M {
    struct S { f: u64 }

    fun t0(s: &S, s_mut: &mut S) {
        &s.g;
        &s_mut.h;
    }
}
