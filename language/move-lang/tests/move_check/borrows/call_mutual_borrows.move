module M {
    struct S { f: u64, g: u64 }
    id<T>(r: &T): &T {
        r
    }
    id_mut<T>(r: &mut T): &mut T {
        r
    }
    imm_imm<T1, T2>(x: &T1, y: &T2) { }
    mut_imm<T1, T2>(x: &mut T1, y: &T2) { }
    mut_mut<T1, T2>(x: &mut T1, y: &mut T2) { }

    t0(s1: &mut S, s2: &mut S) {
        imm_imm(freeze(s1), freeze(s1));
        imm_imm(freeze(s1), &s1.f);
        imm_imm(&s1.f, &s1.f);
        imm_imm(id(s1), s2);

        mut_imm(&mut s1.f, &s1.g);
        mut_imm(id_mut(&mut s1.f), id(&s1.g));

        mut_mut(&mut s1.f, &mut s2.g);
        mut_mut(id_mut(&mut s1.f), &mut s2.g);
        mut_mut(s1, s2);
    }
}
