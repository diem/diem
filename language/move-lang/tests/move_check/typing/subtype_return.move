module M {
    struct S {}

    t0(u: &mut u64): &u64 {
        u
    }

    t1(s: &mut S): &S {
        s
    }

    t2(u1: &mut u64, u2: &mut u64): (&u64, &mut u64) {
        (u1, u2)
    }

    t3(u1: &mut u64, u2: &mut u64): (&mut u64, &u64) {
        (u1, u2)
    }

    t4(u1: &mut u64, u2: &mut u64): (&u64, &u64) {
        (u1, u2)
    }
}
