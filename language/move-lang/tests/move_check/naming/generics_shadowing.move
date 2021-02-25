address 0x2 {

module M {
    struct S {}

    fun foo<S: copy + drop>(s: S): S {
        let s: S = (s: S);
        let s: S = s;
        s
    }

}

}
