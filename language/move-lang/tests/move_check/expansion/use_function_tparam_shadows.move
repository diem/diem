address 0x2 {
module X {
    fun foo() {}
}

module M {
    use 0x2::X::foo;

    fun bar<foo>() {
        foo()
    }
}
}
