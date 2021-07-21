module 0x8675309::M {
    struct S has drop { u: u64 }
    struct R {
        f: u64
    }
    struct G<T> has drop { f: T }

    fun t0(r: &R, r_mut: &mut R, s: S, s_ref: &S, s_mut: &mut S) {
        (0 != 1: bool);
        (0 != (1: u8): bool);
        ((0: u8) != 1: bool);
        (0 != (1: u128): bool);
        ((0: u128) != 1: bool);
        (&0 != &1: bool);
        (true != false: bool);
        (0x0 != 0x1: bool);
        (&s != s_ref: bool);
        (&mut s != s_ref: bool);
        (&mut s != s_mut: bool);
        (&s != s_mut: bool);
        (s_ref != s_mut: bool);
        (s_mut != s_mut: bool);
        (S{u: 0} != s: bool);
        (r != r: bool);
        (r_mut != r_mut: bool);
        (r != r_mut: bool);
        (r_mut != r: bool);
        (G { f: 1 } != G<u64> { f: 2 }: bool);
        (G<u64> { f: 1 } != G { f: 2 }: bool);
    }
}
