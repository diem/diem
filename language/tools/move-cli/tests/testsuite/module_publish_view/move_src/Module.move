address 0x42 {
module Module {
    resource struct S{ i: u64}

    public fun foo(i: u64): S {
        S { i }
    }
}
}
