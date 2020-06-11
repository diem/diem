address 0x2 {
module M {
    struct S1 { b: bool }
    struct S2 { u: u64 }
    fun check(): bool {
        false
    }
    fun num(): u64 {
        0
    }

    fun t() {
        use 0x2::M::{check as foo, num as foo};
        use 0x2::M as N;
        use 0x2::M as N;

    }
}
}
