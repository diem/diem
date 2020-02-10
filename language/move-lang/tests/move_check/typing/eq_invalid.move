module M {
    struct S { u: u64 }
    resource struct R {
        f: u64
    }
    struct G0<T: copyable> {}
    struct G1<T: resource> {}
    struct G2<T> {}



    fun t0(r: &R, r_mut: &mut R, s: S, s_ref: &S, s_mut: &mut S) {
        0 == false;
        &0 == 1;
        1 == &0;
        s == s_ref;
        s_mut == s;
    }

    fun t1(r: R) {
        r == r;
    }

    fun t3() {
        G0{} == G0{};
        G1{} == G1{};
        G2{} == G2{};
    }

    fun t4() {
        () == ();
        (0, 1) == (0, 1);
        (1, 2, 3) == (0, 1);
        (0, 1) == (1, 2, 3);
    }
}
