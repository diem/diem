module M {
    resource struct R {}
    resource struct B { r: R }

    fun t0(r: &R, b: &B) {
        R {} = *r;
        B { r: R{} } = *b;
        R{} = *&b.r;
    }

    fun t1(r: &mut R, b: &mut B) {
        R {} = *r;
        B { r: R{} } = *b;
        R{} = *&b.r;
        R{} = *&mut b.r;
    }

}
