address 0x2 {
module X {
    spec module {
        define foo(): bool { true }
        define bar(): bool { true }
    }
}

module M {
    use 0x2::X::{foo, bar as baz};
    fun t() {
        foo();
        baz();
    }
}

}
