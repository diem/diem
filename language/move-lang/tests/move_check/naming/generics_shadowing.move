address 0x1:

module M {
    use 0x1.X;

    resource struct S {}

    foo<S: copyable>(s: S): S {
        let s: S = (s: S);
        let s: S = copy s;
        s
    }

}
