address 0x1:

module M {
    resource struct S {}

    foo<S: copyable>(s: S): S {
        let s: S = (s: S);
        let s: S = s;
        s
    }

}
