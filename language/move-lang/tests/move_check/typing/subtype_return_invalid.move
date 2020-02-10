module M {
    struct S {}

    fun t0(u: &u64): &mut u64 {
        u
    }

    fun t1(s: &S): &mut S {
        s
    }

    fun t2(u1: &u64, u2: &u64): (&u64, &mut u64) {
        (u1, u2)
    }

    fun t3(u1: &u64, u2: &u64): (&mut u64, &u64) {
        (u1, u2)
    }

    fun t4(u1: &u64, u2: &u64): (&mut u64, &mut u64) {
        (u1, u2)
    }
}
