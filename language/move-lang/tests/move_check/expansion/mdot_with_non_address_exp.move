address 0x2 {

module X {
    fun bar() { }
}
module M {

    fun foo() {
        01::X::bar()
    }

    fun bar() {
        false::X::bar()
    }

    fun baz() {
        foo().bar().X::bar()
    }
}

}
