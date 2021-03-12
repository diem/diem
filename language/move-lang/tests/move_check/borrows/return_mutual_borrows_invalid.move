module 0x8675309::M {
    struct S { f: u64, g: u64 }
    fun id<T>(r: &T): &T {
        r
    }
    fun id_mut<T>(r: &mut T): &mut T {
        r
    }
    fun imm_imm<T1, T2>(_x: &T1, _xy: &T2) { }
    fun mut_imm<T1, T2>(_x: &mut T1, _y: &T2) { }
    fun mut_mut<T1, T2>(_x: &mut T1, _y: &mut T2) { }

    fun mut_imm_0(s1: &mut S): (&mut S, &S) {
        let f = freeze(s1);
        (s1, f)
    }
    fun mut_imm_1(s1: &mut S): (&mut S, &u64) {
        let f = &s1.f;
        (s1, f)
    }
    fun mut_imm_2(s1: &mut S): (&mut u64, &u64) {
        let f = &s1.f;
        (&mut s1.f, f)
    }
    fun mut_imm_3(s1: &mut S): (&mut u64, &u64) {
        let f = id(&s1.f);
        (&mut s1.f, f)
    }

    fun mut_mut_0(s1: &mut S): (&mut S, &mut S) {
        (s1, s1)
    }
    fun mut_mut_1(s1: &mut S): (&mut S, &mut u64) {
        let f =  &mut s1.f;
        (s1, f)
    }
    fun mut_mut_2(s1: &mut S): (&mut u64, &mut S) {
        (&mut s1.f, s1)
    }
    fun mut_mut_3(s1: &mut S): (&mut S, &mut S) {
        (id_mut(s1), s1)
    }
    fun mut_mut_4(s1: &mut S): (&mut S, &mut u64) {
        let f = id_mut(&mut s1.f);
        (s1, f)
    }
    fun mut_mut_5(s1: &mut S): (&mut u64, &mut S) {
        (id_mut(&mut s1.f), s1)
    }
}
