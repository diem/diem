module 0x8675309::M {
    struct S { u: u64 }
    struct R has key {
        f: u64
    }
    struct G0<T> has drop {}
    struct G1<T: key> {}



    fun t0(r: &R, r_mut: &mut R, s: S,
    s_ref: &S, s_mut: &mut S) {
        (0: u8) == (1: u128);
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
        G0<R>{} == G0<R>{};
        // can be dropped, but cannot infer type
        G0{} == G0{};
        G1{} == G1{};
    }

    fun t4() {
        () == ();
        (0, 1) == (0, 1);
        (1, 2, 3) == (0, 1);
        (0, 1) == (1, 2, 3);
    }
}
