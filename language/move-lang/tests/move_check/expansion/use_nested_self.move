address 0x2 {
module X {
    struct S {}
    public fun foo() {}
}

module M {
    use 0x2::X::Self;

    struct S { f: X::S }
    fun foo() {
        X::foo()
    }
}
}
