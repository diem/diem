module M {
    struct S { f: u64 }
    struct R { s1: S, s2: S }

    fun t0() {
        let f;
        let s2;
        R { s1: S { f }, s2 } = &R { s1: S{f: 0}, s2: S{f: 1} };
        f = 0;
        s2 = S { f: 0 }
    }

    fun t1() {
        let f;
        let s2;
        R { s1: S { f }, s2 } = &mut R { s1: S{f: 0}, s2: S{f: 1} };
        f = 0;
        s2 = S { f: 0 }
    }


    fun t2() {
        let f;
        let s2;
        R { s1: S { f }, s2 } = &mut R { s1: S{f: 0}, s2: S{f: 1} };
        f = &0;
        s2 = &S { f: 0 }
    }
}
