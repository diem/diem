address 0x2 {

module X {
    fun bar() { }
}
module M {

    fun foo() {
        01u8::X::bar()
    }

    fun bar() {
        false::X::bar()
    }

    fun baz() {
        foo().bar().X::bar()
    }
}

}
