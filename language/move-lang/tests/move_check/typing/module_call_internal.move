address 0x1:

module X {
    fun foo() {}
}

module M {
    use 0x1::X;
    fun foo() {
        X::foo()
    }
}
