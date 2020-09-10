module M {
    struct S { f: u64 }
    resource struct R { f: u64 }

    fun foo() {
        let _ : Self::S = S { f: 0 };
        let R { f: _ } : Self::R = R { f: 0 };
    }
}
