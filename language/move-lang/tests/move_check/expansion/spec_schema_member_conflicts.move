address 0x2 {

module X1 {
    spec schema Foo<T> {
        ensures true;
    }
    struct Foo {}
}

module X2 {
    spec schema Foo<T> {
        ensures true;
    }
    spec schema Foo<T> {
        ensures true;
    }
}

module X3 {
    spec schema Foo<T> {
        ensures true;
    }
    fun Foo() {}
}

module X4 {
    spec schema Foo<T> {
        ensures true;
    }
    spec module {
        fun Foo(): bool { false }
    }
}

}
