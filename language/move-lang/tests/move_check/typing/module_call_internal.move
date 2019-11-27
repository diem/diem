address 0x1:

module X {
    foo() {}
}

module M {
    use 0x1::X;
    foo() {
        X::foo()
    }
}
