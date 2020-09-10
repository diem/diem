module M {
    struct S { f: u64 }
    struct G {}
    fun foo() {
        let f = 0;
        let s = S 0;
        let s = S f;
        let g = G ();
        let g = G { {} };
    }
}
