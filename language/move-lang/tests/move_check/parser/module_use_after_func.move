address 0x1 {
module X {
    public fun foo() {}
}

module M {
    fun f() {}
    // Use declarations can come after function declarations.
    use 0x1::X;

    fun g() { X::foo() }
}
}
