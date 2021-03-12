module 0x8675309::M {
    struct B { f: bool }
    struct P { b1: B, b2: B }

    fun t0(p: &mut P) {
        let f = &mut p.b1.f;
        let comp = (&mut p.b1) == (&mut p.b2);
        *f = comp
    }

    fun t1(p: &mut P) {
        let f = &mut p.b1.f;
        let comp = (&mut p.b1) != (&mut p.b2);
        *f = comp
    }
}
