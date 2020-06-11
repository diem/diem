address 0x2 {

module A {
    public fun foo() {
        0x2::B::foo()
    }
}

module B {
    public fun foo() {
        0x2::A::foo()
    }
}

}
