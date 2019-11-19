module M {
    struct S { f: u64 }

    t0(s: &S, s_mut: &mut S): (u64, u64) {
        (s.f, s_mut.f)
    }
}
