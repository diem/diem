address 0x1 {
module X {
    spec module {
        define foo(): bool { true }
        define bar(): bool { true }
    }
}

module M {
    use 0x1::X::{foo, bar as baz};
    fun t() {
    }

    spec fun t {
        ensures foo();
        ensures baz();
    }
}

}
