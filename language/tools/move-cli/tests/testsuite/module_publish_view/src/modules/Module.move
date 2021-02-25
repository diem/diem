address 0x42 {
module Module {
    struct S { i: u64 }

    public fun foo(i: u64): S {
        S { i }
    }
}
}
