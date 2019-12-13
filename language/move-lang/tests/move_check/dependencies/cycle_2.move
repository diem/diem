address 0x1:

module A {
    public foo() {
        0x1::B::foo()
    }
}

module B {
    public foo() {
        0x1::A::foo()
    }
}
