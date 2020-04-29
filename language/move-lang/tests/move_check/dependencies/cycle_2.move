address 0x1 {

module A {
    public fun foo() {
        0x1::B::foo()
    }
}

module B {
    public fun foo() {
        0x1::A::foo()
    }
}

}
