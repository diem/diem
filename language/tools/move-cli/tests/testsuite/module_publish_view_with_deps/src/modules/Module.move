address 0x42 {
module Module {
    use 0x1::Debug;

    resource struct S { i: u64 }

    public fun foo(i: u64): S {
        Debug::print(&i);
        S { i }
    }
}
}
