address 0x2 {

module M {
    resource struct S {}

    fun foo<S: copyable>(s: S): S {
        let s: S = (s: S);
        let s: S = s;
        s
    }

}

}
