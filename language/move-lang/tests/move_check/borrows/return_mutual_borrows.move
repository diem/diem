module M {
    struct S { f: u64, g: u64 }
    fun id<T>(r: &T): &T {
        r
    }
    fun id_mut<T>(r: &mut T): &mut T {
        r
    }

    fun imm_imm_0(s1: &mut S): (&S, &S) {
        (freeze(s1), freeze(s1))
    }
    fun imm_imm_1(s1: &mut S): (&S, &u64) {
        (freeze(s1), &s1.f)
    }
    fun imm_imm_2(s1: &mut S): (&u64, &u64) {
        (&s1.f, &s1.f)
    }
    fun imm_imm_3(s1: &mut S, s2: &mut S): (&S, &S) {
        (id(s1), s2)
    }

    fun mut_imm_0(s1: &mut S): (&mut u64, &u64) {
        (&mut s1.f, &s1.g)
    }
    fun mut_imm_1(s1: &mut S): (&mut u64, &u64) {
        (id_mut(&mut s1.f), id(&s1.g))
    }

    fun mut_mut_0(s1: &mut S, s2: &mut S): (&mut u64, &mut u64) {
        (&mut s1.f, &mut s2.g)
    }
    fun mut_mut_1(s1: &mut S, s2: &mut S): (&mut u64, &mut u64) {
        (id_mut(&mut s1.f), &mut s2.g)
    }
    fun mut_mut_2(s1: &mut S, s2: &mut S): (&mut S, &mut S) {
        (s1, s2)
    }
}
