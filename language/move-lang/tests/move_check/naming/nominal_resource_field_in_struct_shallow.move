module M {
    resource struct R {}
    struct S {
        f: R
    }
    struct S2<T: resource> {
        f: R
    }
}
