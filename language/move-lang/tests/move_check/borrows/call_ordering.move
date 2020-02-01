module M {
    struct S { f: u64 }

    foo(_s: &S, _u: u64) {}
    t0(s: &mut S) {
        let f = &mut s.f;
        foo(freeze(s), { *f = 0; 1 })
    }

    bar(_s: &mut u64, _u: u64) {}
    t1(s: &mut S) {
        bar(&mut s.f, { s.f = 0; 1 })
    }
}
