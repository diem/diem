address 0x1 {
module X {
    struct S {}
    public fun foo() {}
}

module M {
    use 0x1::X;
    use 0x1::X::Self;

    struct S { f: X::S }
    fun foo() {
        X::foo()
    }
}
}
