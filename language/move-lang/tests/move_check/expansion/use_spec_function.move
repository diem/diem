address 0x2 {
module X {
    spec module {
        fun foo(): bool { true }
        fun bar(): bool { true }
    }
}

module M {
    use 0x2::X::{foo, bar as baz};
    fun t() {
    }

    spec t {
        ensures foo();
        ensures baz();
    }
}

}
