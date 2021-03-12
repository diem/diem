module 0x8675309::M {
    struct S has drop { f: u64 }
    struct R { f: u64 }

    fun foo() {
        let _ : Self::S = S { f: 0 };
        let R { f: _ } : Self::R = R { f: 0 };
    }
}
