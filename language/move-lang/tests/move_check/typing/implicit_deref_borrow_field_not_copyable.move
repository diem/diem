module 0x8675309::M {
    struct R {}
    struct S has copy, drop {}
    struct B { s: S, r: R }

    fun t1(b: B, bref: &B) {
        (b.s: S);
        R{} = b.r;

        (bref.s: S);
        R{} = bref.r;
    }

}
