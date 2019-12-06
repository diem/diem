module M {
    struct S { f: u64 }

    t0(cond: bool, s: &S, s_mut: &mut S) {
        ((if (cond) s else s).f: u64);
        ((if (cond) s_mut else s).f: u64);
        ((if (cond) s else s_mut).f: u64);
        ((if (cond) s_mut else s_mut).f: u64);
        ({ let s = S{f: 0}; &s }.f: u64);
    }
}
