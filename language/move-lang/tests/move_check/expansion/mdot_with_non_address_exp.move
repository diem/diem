address 0x1:

module X {
    bar() { }
}
module M {

    foo() {
        01::X::bar()
    }

    bar() {
        false::X::bar()
    }

    baz() {
        foo().bar().X::bar()
    }
}
