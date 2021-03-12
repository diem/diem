module 0x8675309::M {
    struct S has drop { f: u64 }
    struct R { s1: S, s2: S }

    fun t0() {
        let R { s1: S { f }, s2 }: R;
        f = 0; f;
        s2 = S { f: 0 }; s2;
    }

    fun t1() {
        let R { s1: S { f }, s2 }: &R;
        f = &0; f;
        s2 = &S { f: 0 }; s2;
    }

    fun t2() {
        let R { s1: S { f }, s2 }: &mut R;
        f = &mut 0; f;
        s2 = &mut S { f: 0 }; s2;
    }
}
