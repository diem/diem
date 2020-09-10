address 0x2 {
module X {
    public fun foo() {}
}

module M {
    fun f() {}
    // Use declarations can come after function declarations.
    use 0x2::X;

    fun g() { X::foo() }
}
}
