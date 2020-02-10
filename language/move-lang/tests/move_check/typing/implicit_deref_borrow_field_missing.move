module M {
    struct S { f: u64 }

    fun t0(s: S, sref: &S, s_mut: &mut S) {
        s.g;
        sref.g;
        s_mut.h;
    }
}
