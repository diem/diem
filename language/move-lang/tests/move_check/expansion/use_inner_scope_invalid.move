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

    spec schema Foo<T> {
        ensures true;
    }

    fun t() {
        use 0x2::M as Self;
        use 0x2::M::{S1 as s1, Foo as foo};

    }
}
}
