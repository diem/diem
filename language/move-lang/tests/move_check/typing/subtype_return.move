module M {
    struct S {}

    fun t0(u: &mut u64): &u64 {
        u
    }

    fun t1(s: &mut S): &S {
        s
    }

    fun t2(u1: &mut u64, u2: &mut u64): (&u64, &mut u64) {
        (u1, u2)
    }

    fun t3(u1: &mut u64, u2: &mut u64): (&mut u64, &u64) {
        (u1, u2)
    }

    fun t4(u1: &mut u64, u2: &mut u64): (&u64, &u64) {
        (u1, u2)
    }
}
