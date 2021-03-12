module 0x8675309::M {
    struct X { f: u64 }
    struct S { v: u64, x: X }
    fun t() {
        let s = S { v: 0, x: X { f: 0 }};
        &mut (&s).v;
        &mut (&s.x).f;
        let sref = &s;
        let xref = &s.x;
        &mut sref.v;
        &mut xref.v;
    }

    fun t2(s: &S, x: &X) {
        x.f = x.f + 1;
        s.x.f = s.x.f + 1
    }
}
