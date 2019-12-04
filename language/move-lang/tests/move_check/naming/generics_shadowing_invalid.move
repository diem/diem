address 0x1:

module M {
    struct S {}

    foo<S>(s1: S, s2: S): S {
        (s1: Self::S);
        let s: S = S {}; // TODO error? should this try to construct the generic ?
        bar(s1);
        S {}
    }

    bar(s: S) {}
}
