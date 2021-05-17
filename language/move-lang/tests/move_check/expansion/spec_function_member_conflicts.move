address 0x2 {

module X1 {
    spec module {
        fun Foo(): bool { true }
    }
    struct Foo {}
}

module X2 {
    spec module {
        fun Foo(): bool { true }
    }
    spec schema Foo<T> {
        ensures true;
    }
}

module X3 {
    spec module {
        fun Foo(): bool { true }
    }
    fun Foo() {}
}

module X4 {
    spec module {
        fun foo(): bool { true }
    }
    fun foo() {}
}

module X5 {
    spec module {
        fun foo(): bool { true }
    }
    spec module {
        fun foo(): bool { true }
    }
}
}
