module M {
    struct S { f: u64 }

    t0(s: &S, s_mut: &mut S, s_mut2: &mut S): (&u64, &u64, &mut u64) {
        (&s.f, &s_mut.f, &mut s_mut2.f)
    }
}
