address 0x1 {
module X {
    public fun foo() {}
}

module M {
    struct S {}
    // Use declarations can come after struct declarations.
    use 0x1::X;

    fun g() { X::foo() }
}
}
