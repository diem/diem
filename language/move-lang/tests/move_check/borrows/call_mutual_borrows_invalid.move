module M {
    struct S { f: u64, g: u64 }
    fun id<T>(r: &T): &T {
        r
    }
    fun id_mut<T>(r: &mut T): &mut T {
        r
    }
    fun imm_imm<T1, T2>(_x: &T1, _y: &T2) { }
    fun mut_imm<T1, T2>(_x: &mut T1, _y: &T2) { }
    fun mut_mut<T1, T2>(_x: &mut T1, _y: &mut T2) { }

    fun t0(s1: &mut S, _s2: &mut S) {
        let f = freeze(s1);
        mut_imm(s1, f);
        let f = &s1.f;
        mut_imm(s1, f);
        let f = &s1.f;
        mut_imm(&mut s1.f, f);
        let f = id(&s1.f);
        id_mut(&mut s1.f); f;

        mut_mut(s1, s1);
        let f = &mut s1.f;
        mut_mut(s1, f);
        mut_mut(&mut s1.f, s1);
        let s = id_mut(s1);
        id_mut(s1);
        s;
        let f = id_mut(&mut s1.f);
        mut_mut(s1, f);
        mut_mut(id_mut(&mut s1.f), s1);
    }
}
