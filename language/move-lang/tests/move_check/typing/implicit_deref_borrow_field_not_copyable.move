module M {
    resource struct R {}
    struct S {}
    resource struct B { s: S, r: R }

    fun t1(b: B, bref: &B) {
        (b.s: S);
        R{} = b.r;

        (bref.s: S);
        R{} = bref.r;
    }

}
