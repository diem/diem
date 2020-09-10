module M {
    struct S { f: u64, g: u64 }
    struct B { f: bool }
    struct P { b1: B, b2: B }

    fun t(r1: &mut u64, r2: &mut u64, s: &mut S) {
        r1 == r1;
        r1 == r2;
        r2 == r2;
        r2 == r2;

        r1 != r1;
        r1 != r2;
        r2 != r2;
        r2 != r2;

        (&mut s.f) == (&mut s.f);
        (&mut s.f) == (&mut s.g);
        (&mut s.g) == (&mut s.f);
        (&mut s.g) == (&mut s.g);

        (&mut s.f) != (&mut s.f);
        (&mut s.f) != (&mut s.g);
        (&mut s.g) != (&mut s.f);
        (&mut s.g) != (&mut s.g);
    }

    fun t1(p: &mut P) {
        let comp = (&mut p.b1) == (&mut p.b2);
        p.b1.f = comp
    }

    fun t2(p: &mut P) {
        let comp = (&mut p.b1) != (&mut p.b2);
        p.b1.f = comp
    }
}
